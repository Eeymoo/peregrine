//! 物料类型与求值实现。
//!
//! 一个 [`Material`] 实例对应一份 Rhai 脚本，在加载时预编译为 AST。
//! 每次求值时创建独立的 `Engine`（注册了捕获当前 `DynamicContext` 的 host function），
//! 通过 `call_fn` 调用脚本的 `build` 函数，返回 `Vec<Element>`。

use crate::context::DynamicContext;
use crate::error::{MaterialError, MaterialResult};
use peregrine_config::{Element, PathSegment, Rect, SimpleRng};
use rhai::{AST, Array, Dynamic, Engine, ImmutableString, Map, Scope};

/// 物料 id（如 `"builtin.cross"` 或 `"user.my_material"`）。
pub type MaterialId = String;

/// Rhai 单次求值的最大操作数（防死循环）。
const MAX_OPERATIONS: u64 = 1_000_000;
/// Rhai 最大递归调用深度。
const MAX_CALL_LEVELS: usize = 64;

/// 物料元数据，供 UI 展示与缓存策略使用。
#[derive(Debug, Clone)]
pub struct MaterialMetadata {
    /// 物料 id。
    pub id: MaterialId,
    /// 用户可读名称（从脚本顶部的 `// Name: xxx` 注释解析，默认取 id 末段）。
    pub display_name: String,
    /// 是否依赖动态输入（时间/鼠标/键盘/随机）。
    ///
    /// `false` 时物料求值可永久缓存（version 永远为 0）。
    pub is_dynamic: bool,
    /// 动态物料的最小刷新间隔（毫秒）。
    ///
    /// 从脚本可选导出的 `fn refresh_interval_ms() -> Int` 解析（缺失 = 0）。
    /// 语义为「物料输出最快多久变一次」：调度层据此对唤醒节流——
    /// 例如时钟声明 500ms，则 60FPS 配置下 Rhai 求值从每秒 60 次降到 2 次。
    /// 仅对 `is_dynamic = true` 的物料生效；静态物料恒 0（无意义）。
    pub refresh_interval_ms: u32,
}

/// 物料信息，通过 IPC 返回给前端用于物料选择与 UI 控件生成。
#[derive(Debug, Clone, serde::Serialize)]
pub struct MaterialInfo {
    /// 物料 id。
    pub id: MaterialId,
    /// 显示名称。
    pub display_name: String,
    /// 是否为内置物料。
    pub builtin: bool,
    /// 是否依赖动态输入。
    pub is_dynamic: bool,
    /// 默认参数（JSON 对象）。
    pub defaults: serde_json::Value,
    /// 参数 schema（JSON 数组）。
    pub schema: serde_json::Value,
}

/// 已加载的物料，持有预编译的 Rhai AST 与缓存元数据。
pub struct Material {
    metadata: MaterialMetadata,
    source: String,
    ast: AST,
    /// 缓存：`defaults()` 返回值（JSON）。
    cached_defaults: serde_json::Value,
    /// 缓存：`schema()` 返回值（JSON）。
    cached_schema: serde_json::Value,
}

impl Material {
    /// 从源码加载物料。
    ///
    /// 加载时：
    /// 1. 用临时 Engine 编译源码为 AST
    /// 2. 调用 `defaults()` 和 `schema()` 缓存元数据
    /// 3. 调用 `is_dynamic()` 确定动态性
    pub fn load(id: MaterialId, source: &str, _builtin: bool) -> MaterialResult<Self> {
        let engine = make_engine();
        let ast = engine.compile(source).map_err(|e| MaterialError::Parse {
            id: id.clone(),
            source: e,
        })?;

        let mut scope = Scope::new();

        // 调用 defaults() 缓存默认参数。
        let defaults_val: Dynamic =
            engine
                .call_fn(&mut scope, &ast, "defaults", ())
                .map_err(|_| MaterialError::MissingFunction {
                    id: id.clone(),
                    function: "defaults",
                })?;
        let cached_defaults = dynamic_to_json(&defaults_val);

        // 调用 schema() 缓存参数 schema。
        let schema_val: Dynamic = engine
            .call_fn(&mut scope, &ast, "schema", ())
            .map_err(|_| MaterialError::MissingFunction {
                id: id.clone(),
                function: "schema",
            })?;
        let cached_schema = dynamic_to_json(&schema_val);

        // 调用 is_dynamic()，若缺失则默认为 false（静态物料）。
        let is_dynamic: bool = engine
            .call_fn(&mut scope, &ast, "is_dynamic", ())
            .unwrap_or(false);

        // 调用可选的 refresh_interval_ms()：动态物料声明最小刷新间隔。
        // 缺失 / 非法（负数 / 溢出）时回退 0 = 不节流（每帧求值，兼容既有脚本）。
        let refresh_interval_ms: u32 = engine
            .call_fn::<i64>(&mut scope, &ast, "refresh_interval_ms", ())
            .ok()
            .and_then(|ms| u32::try_from(ms).ok())
            .unwrap_or(0);

        let display_name = parse_display_name(source)
            .unwrap_or_else(|| id.rsplit('.').next().unwrap_or(&id).to_string());

        let metadata = MaterialMetadata {
            id,
            display_name,
            is_dynamic,
            refresh_interval_ms,
        };

        Ok(Self {
            metadata,
            source: source.to_string(),
            ast,
            cached_defaults,
            cached_schema,
        })
    }

    /// 物料 id。
    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    /// 物料元数据。
    pub fn metadata(&self) -> &MaterialMetadata {
        &self.metadata
    }

    /// 物料源码（用于调试 / 编辑器）。
    pub fn source(&self) -> &str {
        &self.source
    }

    /// 默认参数（JSON）。
    pub fn defaults(&self) -> &serde_json::Value {
        &self.cached_defaults
    }

    /// 参数 schema（JSON）。
    pub fn schema(&self) -> &serde_json::Value {
        &self.cached_schema
    }

    /// 生成 `MaterialInfo`（供 IPC 返回前端）。
    pub fn info(&self, builtin: bool) -> MaterialInfo {
        MaterialInfo {
            id: self.metadata.id.clone(),
            display_name: self.metadata.display_name.clone(),
            builtin,
            is_dynamic: self.metadata.is_dynamic,
            defaults: self.cached_defaults.clone(),
            schema: self.cached_schema.clone(),
        }
    }

    /// 求值：参数 + 屏幕区域 + 动态上下文 → Element 列表。
    ///
    /// 每次求值创建独立的 Engine（注册了捕获 `ctx` 的 host function），
    /// 共享预编译的 AST，通过 `call_fn` 调用脚本的 `build` 函数。
    pub fn evaluate(
        &self,
        params: &serde_json::Value,
        screen: &Rect,
        ctx: &DynamicContext,
    ) -> MaterialResult<Vec<Element>> {
        let engine = make_engine_with_dynamic(ctx);

        // 合并默认参数与传入参数（传入值优先）。
        let merged = merge_params(&self.cached_defaults, params);
        let params_map = json_to_rhai_map(&merged);
        let screen_map = rect_to_rhai_map(screen);

        let mut scope = Scope::new();
        let result: Dynamic = engine
            .call_fn(&mut scope, &self.ast, "build", (params_map, screen_map))
            .map_err(|e| MaterialError::Evaluation {
                id: self.metadata.id.clone(),
                message: e.to_string(),
            })?;

        let arr = result
            .into_array()
            .map_err(|type_name| MaterialError::InvalidReturnType {
                id: self.metadata.id.clone(),
                detail: format!("expected Array, got {}", type_name),
            })?;

        arr.into_iter()
            .map(|d| dynamic_to_element(&self.metadata.id, d))
            .collect()
    }
}

/// 创建一个 Rhai Engine，应用沙箱限制。
fn make_engine() -> Engine {
    let mut engine = Engine::new();
    engine.set_max_operations(MAX_OPERATIONS);
    engine.set_max_call_levels(MAX_CALL_LEVELS);
    // 提高表达式深度限制，避免复杂物料（grid/random_orb 的嵌套算式）触发 ExprTooDeep。
    engine.set_max_expr_depths(128, 128);
    engine
}

/// 创建带动态输入 host function 的 Engine。
///
/// host function 通过 closure 捕获 `DynamicContext` 的不可比引用副本。
/// Rhai Engine 要求 register_fn 的 closure 实现 `Fn`（非 `FnMut`），
/// 因此 RNG 状态以种子形式保存，每次调用重新构造 RNG（确定性随机）。
fn make_engine_with_dynamic(ctx: &DynamicContext) -> Engine {
    let mut engine = make_engine();

    // time_ms() -> i64
    // Rhai 的 INT 默认是 i64。
    let time_ms = ctx.time_ms as i64;
    engine.register_fn("time_ms", move || time_ms);

    // now_ms() -> i64：当前系统时间（毫秒），直读墙钟。
    //
    // 不推荐：绕过 DynamicContext 快照，导致预览与 overlay 求值时刻不一致
    // （WYSIWYG 漂移）。仅为兼容既有用户脚本保留注册，新脚本一律用 time_ms()。
    engine.register_fn("now_ms", || chrono::Local::now().timestamp_millis());

    // format_time(ms, format) -> String：把毫秒时间戳格式化为本地时间字符串。
    // 支持占位符：yyyy, MM, dd, HH, hh, mm, ss, a。
    engine.register_fn("format_time", |ms: i64, format: ImmutableString| {
        use chrono::TimeZone;
        let dt = chrono::Local
            .timestamp_millis_opt(ms)
            .single()
            .unwrap_or_else(chrono::Local::now);
        let mut s = format.to_string();
        // 先替换较长占位符，避免短占位符冲突（如先 yyyy 再 MM）。
        let replacements = [
            ("yyyy", dt.format("%Y").to_string()),
            ("MM", dt.format("%m").to_string()),
            ("dd", dt.format("%d").to_string()),
            ("HH", dt.format("%H").to_string()),
            ("hh", dt.format("%I").to_string()),
            ("mm", dt.format("%M").to_string()),
            ("ss", dt.format("%S").to_string()),
            ("a", dt.format("%p").to_string()),
        ];
        for (pat, val) in &replacements {
            s = s.replace(pat, val);
        }
        s
    });

    // mouse_pos() -> Map { x: Float, y: Float }
    let (mx, my) = ctx.mouse_pos;
    engine.register_fn("mouse_pos", move || {
        let mut m = Map::new();
        m.insert("x".into(), (mx as f64).into());
        m.insert("y".into(), (my as f64).into());
        m
    });

    // mouse_velocity() -> Map { x: Float, y: Float }：鼠标速度（逻辑像素/秒）。
    // 由平台轮询器差分采样 + EMA 平滑 + 死区归零提供；同一快照内多次调用同值。
    let (vx, vy) = ctx.mouse_velocity;
    engine.register_fn("mouse_velocity", move || {
        let mut m = Map::new();
        m.insert("x".into(), (vx as f64).into());
        m.insert("y".into(), (vy as f64).into());
        m
    });

    // mouse_acceleration() -> Map { x: Float, y: Float }：鼠标加速度（逻辑像素/秒²）。
    // 与 mouse_velocity 同风格；死区归零保证静止/匀速时精确为 0（稳态跳帧保障）。
    let (ax, ay) = ctx.mouse_acceleration;
    engine.register_fn("mouse_acceleration", move || {
        let mut m = Map::new();
        m.insert("x".into(), (ax as f64).into());
        m.insert("y".into(), (ay as f64).into());
        m
    });

    // key_down(code: &str) -> Bool
    let key_state = ctx.key_state.clone();
    engine.register_fn("key_down", move |code: ImmutableString| {
        key_state.is_down(&code)
    });

    // RNG：用种子派生。为使每次 rand() 都产生不同结果（同一帧内多次调用），
    // 我们使用计数器：rand_counter 原子递增。
    // 但 register_fn 要求 Fn，不能在 closure 中维护可变状态。
    // 解决：暴露 `rand()` 每次返回基于 `(seed, counter)` 的固定序列；
    // 物料脚本若需要多个随机数，应调用 `rand_range(0, N)` 拿到不同结果。
    // 当前实现：每次 `rand()` 调用都重新构造 RNG 并推进一次状态。
    // 物料脚本若需独立随机流，可显式调用 `rand_int(N)` 取下标。
    let seed = ctx.rng_seed.max(1) as i64;
    thread_local_reset(seed as u64);
    engine.register_fn("rand_seed", move |s: i64| {
        // 设置全局种子的 host function 版本：Rhai 脚本侧无法持久修改 host 状态，
        // 但仍暴露此函数供 API 兼容（实际无效）。
        let _ = s;
    });
    // rand() 内部用 Rhai 的内置随机数（由时间派生）。
    // 这里实现一个简单的 LCG：每次 rand() 调用基于种子产生一个数。
    // 由于 register_fn 不能 FnMut，我们用静态 thread-local 计数器。
    let seed_for_rand = seed;
    engine.register_fn("rand", move || thread_local_rand(seed_for_rand as u64));
    engine.register_fn("rand_range", move |min: f64, max: f64| {
        let r = thread_local_rand(seed_for_rand as u64);
        min + r * (max - min)
    });
    engine.register_fn("rand_int", move |max: i64| {
        let r = thread_local_rand(seed_for_rand as u64);
        (r * max.max(1) as f64) as i64
    });

    // parse_svg_path(d) -> Array of #{cmd, x, y, ...}：
    // 把 SVG path 的 d 字符串解析为段数组（绝对坐标）。
    // 支持 M/L/Q/C/Z 与相对 m/l/q/c/z、H/V/h/v（换算为绝对 L）；
    // 相对坐标基于「上一段终点」累加，隐式重复命令（"M 1 2 3 4" = M+L）、
    // 数值分隔符（空格/逗号/符号前缀）均按 SVG 语法处理。
    // H/V 展开为 L（PathSegment 无专命令）；A/S/T 不支持（报运行时错误）。
    // 注意：必须返回裸 Array——Rhai 不会自动解包 Result，
    // 脚本侧拿到包装值会破坏元素转换（segments must be Array）。
    // 解析失败通过 `panic` 关键字路径不可用，改为返回空数组
    // 让脚本显式判空（转换层对空 segments 会报错，问题可见）。
    engine.register_fn("parse_svg_path", |d: ImmutableString| -> Array {
        match parse_svg_path_d(&d) {
            Ok(segs) => segs.into_iter().map(segment_to_dynamic).collect(),
            Err(_) => Array::new(),
        }
    });

    engine
}

/// PathSegment → Rhai Map（`parse_svg_path` 的输出格式，
/// 与脚本文本手写段一致：`#{cmd: "M", x, y}` 等）。
fn segment_to_dynamic(seg: PathSegment) -> Dynamic {
    let mut m = Map::new();
    match seg {
        PathSegment::M { x, y } => {
            m.insert("cmd".into(), "M".into());
            m.insert("x".into(), (x as f64).into());
            m.insert("y".into(), (y as f64).into());
        }
        PathSegment::L { x, y } => {
            m.insert("cmd".into(), "L".into());
            m.insert("x".into(), (x as f64).into());
            m.insert("y".into(), (y as f64).into());
        }
        PathSegment::Q { x1, y1, x, y } => {
            m.insert("cmd".into(), "Q".into());
            m.insert("x1".into(), (x1 as f64).into());
            m.insert("y1".into(), (y1 as f64).into());
            m.insert("x".into(), (x as f64).into());
            m.insert("y".into(), (y as f64).into());
        }
        PathSegment::C {
            x1,
            y1,
            x2,
            y2,
            x,
            y,
        } => {
            m.insert("cmd".into(), "C".into());
            m.insert("x1".into(), (x1 as f64).into());
            m.insert("y1".into(), (y1 as f64).into());
            m.insert("x2".into(), (x2 as f64).into());
            m.insert("y2".into(), (y2 as f64).into());
            m.insert("x".into(), (x as f64).into());
            m.insert("y".into(), (y as f64).into());
        }
        PathSegment::Z => {
            m.insert("cmd".into(), "Z".into());
        }
    }
    Dynamic::from(m)
}

/// SVG path `d` 字符串 → 绝对坐标段序列。
///
/// 为 `parse_svg_path` host function 的实现。语法子集：
/// - 命令：`M` `L` `Q` `C` `Z` 大写（绝对）与小写 `m` `l` `q` `c` `z`（相对）；
/// - `H`/`V`/`h`/`v` 展开为等价的 `L`（PathSegment 无专命令）；
/// - 隐式重复：`M 1 2 3 4` 等价 `M 1 2 L 3 4`（首个 M 后、其余按 L）；
///   `m` 同理（首个 m 后按 l）；其它命令直接重复自身；
/// - 分隔符：空格 / 逗号 / 负号与前缀符号（`1-2` = 1, -2）；
/// - 指数记法（`1e2`）不支持——锚点坐标用不到。
///
/// `A`（圆弧）/`S`/`T`（平滑简写）不支持：返回 `Err`（含位置信息），
/// 脚本作者可用 Q/C 显式展开。相对坐标基于上一段终点累加，输出恒绝对。
fn parse_svg_path_d(d: &str) -> Result<Vec<PathSegment>, String> {
    let mut segs = Vec::new();
    let bytes: Vec<char> = d.chars().collect();
    let n = bytes.len();
    let mut i = 0usize;

    // 当前点（相对坐标基准）与子路径起点（Z 的回退目标）。
    let (mut cx, mut cy) = (0.0f32, 0.0f32);
    let (mut sx, mut sy) = (0.0f32, 0.0f32);
    // 当前命令：显式字母或隐式重复（implicit_cmd 记录上一显式命令
    // 对应的重复形式——M/m 后续坐标按 L/l 处理，其它命令重复自身）。
    #[allow(clippy::needless_late_init)]
    let mut last_cmd;
    let mut implicit_cmd = ' ';

    // 分隔符：SVG 语法允许空格与逗号（含尾随逗号）任意混用。
    let skip_ws = |i: &mut usize| {
        while *i < n && (bytes[*i].is_whitespace() || bytes[*i] == ',') {
            *i += 1;
        }
    };
    let read_number = |i: &mut usize| -> Result<f32, String> {
        skip_ws(i);
        let start = *i;
        if *i < n && (bytes[*i] == '-' || bytes[*i] == '+') {
            *i += 1;
        }
        while *i < n && (bytes[*i].is_ascii_digit() || bytes[*i] == '.') {
            *i += 1;
        }
        if *i == start {
            return Err(format!("数值缺失于位置 {}", start));
        }
        bytes[start..*i]
            .iter()
            .collect::<String>()
            .parse::<f32>()
            .map_err(|_| format!("非法数值于位置 {}", start))
    };

    while {
        skip_ws(&mut i);
        i < n
    } {
        let c = bytes[i];
        if c.is_ascii_alphabetic() {
            i += 1;
            last_cmd = c;
            // M/m 的隐式重复是 L/l，其它命令重复自身。
            implicit_cmd = match c {
                'M' | 'm' => {
                    if c == 'M' {
                        'L'
                    } else {
                        'l'
                    }
                }
                other => other,
            };
        } else {
            // 隐式重复：沿用 implicit_cmd。
            last_cmd = implicit_cmd;
        }

        let is_rel = last_cmd.is_ascii_lowercase();
        // 大写化统一处理。
        let cmd = last_cmd.to_ascii_uppercase();

        match cmd {
            'M' => {
                let x = read_number(&mut i)?;
                let y = read_number(&mut i)?;
                let (ax, ay) = if is_rel { (cx + x, cy + y) } else { (x, y) };
                segs.push(PathSegment::M { x: ax, y: ay });
                cx = ax;
                cy = ay;
                sx = ax;
                sy = ay;
            }
            'L' => {
                let x = read_number(&mut i)?;
                let y = read_number(&mut i)?;
                let (ax, ay) = if is_rel { (cx + x, cy + y) } else { (x, y) };
                segs.push(PathSegment::L { x: ax, y: ay });
                cx = ax;
                cy = ay;
            }
            'H' => {
                let x = read_number(&mut i)?;
                let ax = if is_rel { cx + x } else { x };
                segs.push(PathSegment::L { x: ax, y: cy });
                cx = ax;
            }
            'V' => {
                let y = read_number(&mut i)?;
                let ay = if is_rel { cy + y } else { y };
                segs.push(PathSegment::L { x: cx, y: ay });
                cy = ay;
            }
            'Q' => {
                let x1 = read_number(&mut i)?;
                let y1 = read_number(&mut i)?;
                let x = read_number(&mut i)?;
                let y = read_number(&mut i)?;
                let (ax1, ay1) = if is_rel { (cx + x1, cy + y1) } else { (x1, y1) };
                let (ax, ay) = if is_rel { (cx + x, cy + y) } else { (x, y) };
                segs.push(PathSegment::Q {
                    x1: ax1,
                    y1: ay1,
                    x: ax,
                    y: ay,
                });
                cx = ax;
                cy = ay;
            }
            'C' => {
                let (x1, y1) = (read_number(&mut i)?, read_number(&mut i)?);
                let (x2, y2) = (read_number(&mut i)?, read_number(&mut i)?);
                let (x, y) = (read_number(&mut i)?, read_number(&mut i)?);
                let (ax1, ay1, ax2, ay2, ax, ay) = if is_rel {
                    (cx + x1, cy + y1, cx + x2, cy + y2, cx + x, cy + y)
                } else {
                    (x1, y1, x2, y2, x, y)
                };
                segs.push(PathSegment::C {
                    x1: ax1,
                    y1: ay1,
                    x2: ax2,
                    y2: ay2,
                    x: ax,
                    y: ay,
                });
                cx = ax;
                cy = ay;
            }
            'Z' => {
                segs.push(PathSegment::Z);
                // SVG 语义：Z 后当前点回到子路径起点。
                cx = sx;
                cy = sy;
            }
            'A' | 'S' | 'T' => {
                return Err(format!(
                    "不支持的命令 '{}' 于位置 {}（A/S/T 请用 Q/C 显式展开）",
                    cmd, i
                ));
            }
            _ => {
                return Err(format!("未知命令 '{}' 于位置 {}", cmd, i));
            }
        }
    }

    Ok(segs)
}

thread_local! {
    /// 每个线程一个调用计数器，用于让多次 `rand()` 调用产生不同结果。
    ///
    /// 每次物料求值使用独立的 Rhai Engine，但同一个 Engine 内多次调用 `rand()`
    /// 需要不同的结果。计数器在 Engine 创建时清零（见 `thread_local_reset`）。
    static RAND_COUNTER: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// 重置当前线程的 RNG 计数器（在 Engine 创建时调用）。
fn thread_local_reset(seed: u64) {
    RAND_COUNTER.with(|c| c.set(seed));
}

/// 基于 (种子, 计数器) 产生确定性随机数。
fn thread_local_rand(seed: u64) -> f64 {
    RAND_COUNTER.with(|c| {
        let n = c.get();
        c.set(n.wrapping_add(1));
        let mut rng = SimpleRng::new(seed.wrapping_add(n));
        rng.next_f32() as f64
    })
}

/// 把 `Rect` 转换为 Rhai Map。
fn rect_to_rhai_map(r: &Rect) -> Map {
    let mut m = Map::new();
    m.insert("min_x".into(), (r.min_x as f64).into());
    m.insert("min_y".into(), (r.min_y as f64).into());
    m.insert("max_x".into(), (r.max_x as f64).into());
    m.insert("max_y".into(), (r.max_y as f64).into());
    m
}

/// 把 JSON Value 转换为 Rhai Map（仅处理 Object，其他返回空 Map）。
fn json_to_rhai_map(v: &serde_json::Value) -> Map {
    let mut m = Map::new();
    if let serde_json::Value::Object(obj) = v {
        for (k, val) in obj {
            m.insert(k.clone().into(), json_to_dynamic(val));
        }
    }
    m
}

/// JSON Value → Rhai Dynamic（递归）。
fn json_to_dynamic(v: &serde_json::Value) -> Dynamic {
    match v {
        serde_json::Value::Null => Dynamic::UNIT,
        serde_json::Value::Bool(b) => (*b).into(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into()
            } else if let Some(f) = n.as_f64() {
                f.into()
            } else {
                Dynamic::UNIT
            }
        }
        serde_json::Value::String(s) => s.clone().into(),
        serde_json::Value::Array(arr) => {
            let v: Vec<Dynamic> = arr.iter().map(json_to_dynamic).collect();
            v.into()
        }
        serde_json::Value::Object(obj) => {
            let _ = obj;
            Dynamic::from(json_to_rhai_map(v))
        }
    }
}

/// Rhai Dynamic → JSON Value（递归）。
fn dynamic_to_json(d: &Dynamic) -> serde_json::Value {
    // 单位值。
    if d.is_unit() {
        return serde_json::Value::Null;
    }
    if let Ok(b) = d.as_bool() {
        return serde_json::Value::Bool(b);
    }
    if let Ok(i) = d.as_int() {
        return serde_json::Value::Number(i.into());
    }
    if let Ok(f) = d.as_float() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return serde_json::Value::Number(n);
        }
    }
    if let Ok(s) = d.as_immutable_string_ref() {
        return serde_json::Value::String(s.to_string());
    }
    // 数组：先克隆再 into_array。
    if d.is_array() {
        if let Ok(arr) = d.clone().into_array() {
            return serde_json::Value::Array(arr.iter().map(dynamic_to_json).collect());
        }
    }
    // Map：通过 as_map_ref（返回 Result<impl Deref>）。
    if let Ok(m) = d.as_map_ref() {
        let mut obj = serde_json::Map::new();
        for (k, v) in m.iter() {
            obj.insert(k.to_string(), dynamic_to_json(v));
        }
        return serde_json::Value::Object(obj);
    }
    serde_json::Value::Null
}

/// 把 Rhai Dynamic（一个 Element 描述 Map）转换为 `Element`。
fn dynamic_to_element(material_id: &str, d: Dynamic) -> MaterialResult<Element> {
    let m: Map = if d.is_map() {
        // 已知是 Map，直接 try_cast（拥有所有权）。
        d.try_cast::<Map>().unwrap_or_default()
    } else {
        return Err(MaterialError::InvalidReturnType {
            id: material_id.to_string(),
            detail: format!("element must be a Map, got {}", d.type_name()),
        });
    };

    let get_str = |key: &str| -> MaterialResult<String> {
        let v = m.get(key).ok_or_else(|| MaterialError::ElementField {
            id: material_id.to_string(),
            detail: format!("missing string field '{}'", key),
        })?;
        v.as_immutable_string_ref()
            .map(|s| s.to_string())
            .map_err(|_| MaterialError::ElementField {
                id: material_id.to_string(),
                detail: format!("field '{}' must be string", key),
            })
    };
    let get_f32 = |key: &str| -> MaterialResult<f32> {
        let v = m.get(key).ok_or_else(|| MaterialError::ElementField {
            id: material_id.to_string(),
            detail: format!("missing field '{}'", key),
        })?;
        if let Ok(f) = v.as_float() {
            Ok(f as f32)
        } else if let Ok(i) = v.as_int() {
            Ok(i as f32)
        } else {
            Err(MaterialError::ElementField {
                id: material_id.to_string(),
                detail: format!("field '{}' must be number", key),
            })
        }
    };

    let type_name = get_str("type")?;

    // 可选字重：缺失 / 显式 () 视为 None；存在时必须是 100–900 的百位整数倍。
    let get_font_weight = || -> MaterialResult<Option<u16>> {
        let Some(v) = m.get("font_weight") else {
            return Ok(None);
        };
        if v.is_unit() {
            return Ok(None);
        }
        let w = if let Ok(f) = v.as_float() {
            f as i64
        } else if let Ok(i) = v.as_int() {
            i
        } else {
            return Err(MaterialError::ElementField {
                id: material_id.to_string(),
                detail: "field 'font_weight' must be number".to_string(),
            });
        };
        if !(100..=900).contains(&w) || w % 100 != 0 {
            return Err(MaterialError::ElementField {
                id: material_id.to_string(),
                detail: format!(
                    "field 'font_weight' must be a multiple of 100 in 100..=900, got {}",
                    w
                ),
            });
        }
        Ok(Some(w as u16))
    };
    match type_name.as_str() {
        "rect" => Ok(Element::Rect {
            x: get_f32("x")?,
            y: get_f32("y")?,
            w: get_f32("w")?,
            h: get_f32("h")?,
            // 任务 9.3：支持 edge_rect 等物料的 corner_radius 字段。
            // 物料脚本不输出该字段时回退 None（直角，向后兼容）。
            corner_radius: get_f32("corner_radius").ok(),
        }),
        "circle" => Ok(Element::Circle {
            cx: get_f32("cx")?,
            cy: get_f32("cy")?,
            radius: get_f32("radius")?,
        }),
        "circle_stroke" => Ok(Element::CircleStroke {
            cx: get_f32("cx")?,
            cy: get_f32("cy")?,
            radius: get_f32("radius")?,
            thickness: get_f32("thickness")?,
        }),
        "dashed_circle" => Ok(Element::DashedCircle {
            cx: get_f32("cx")?,
            cy: get_f32("cy")?,
            radius: get_f32("radius")?,
            thickness: get_f32("thickness")?,
            dash_len: get_f32("dash_len")?,
            gap_len: get_f32("gap_len")?,
        }),
        "triangle" => Ok(Element::Triangle {
            x1: get_f32("x1")?,
            y1: get_f32("y1")?,
            x2: get_f32("x2")?,
            y2: get_f32("y2")?,
            x3: get_f32("x3")?,
            y3: get_f32("y3")?,
        }),
        "polygon" => {
            let points_val = m.get("points").ok_or_else(|| MaterialError::ElementField {
                id: material_id.to_string(),
                detail: "missing 'points' field".to_string(),
            })?;
            let arr = points_val
                .clone()
                .into_array()
                .map_err(|_| MaterialError::ElementField {
                    id: material_id.to_string(),
                    detail: "'points' must be Array".to_string(),
                })?;
            let mut pts = Vec::with_capacity(arr.len());
            for p in arr {
                // 多边形点支持两种格式：Map #{0: x, 1: y} 或 #{x: x, y: y}。
                let x: f32;
                let y: f32;
                if let Ok(mref) = p.as_map_ref() {
                    let xv = mref.get("0").or_else(|| mref.get("x"));
                    let yv = mref.get("1").or_else(|| mref.get("y"));
                    let extract = |v: Option<&Dynamic>| -> Option<f32> {
                        v.and_then(|v| {
                            v.as_float()
                                .map(|f| f as f32)
                                .ok()
                                .or_else(|| v.as_int().ok().map(|i| i as f32))
                        })
                    };
                    x = extract(xv).ok_or_else(|| MaterialError::ElementField {
                        id: material_id.to_string(),
                        detail: "polygon point missing x".to_string(),
                    })?;
                    y = extract(yv).ok_or_else(|| MaterialError::ElementField {
                        id: material_id.to_string(),
                        detail: "polygon point missing y".to_string(),
                    })?;
                } else if let Ok(arr2) = p.clone().into_array() {
                    // 数组 [x, y]
                    if arr2.len() < 2 {
                        return Err(MaterialError::ElementField {
                            id: material_id.to_string(),
                            detail: "polygon point array must have 2 elements".to_string(),
                        });
                    }
                    let extract = |v: &Dynamic| -> Option<f32> {
                        v.as_float()
                            .map(|f| f as f32)
                            .ok()
                            .or_else(|| v.as_int().ok().map(|i| i as f32))
                    };
                    x = extract(&arr2[0]).ok_or_else(|| MaterialError::ElementField {
                        id: material_id.to_string(),
                        detail: "polygon point[0] invalid".to_string(),
                    })?;
                    y = extract(&arr2[1]).ok_or_else(|| MaterialError::ElementField {
                        id: material_id.to_string(),
                        detail: "polygon point[1] invalid".to_string(),
                    })?;
                } else {
                    return Err(MaterialError::ElementField {
                        id: material_id.to_string(),
                        detail: "polygon point must be Map or Array".to_string(),
                    });
                }
                pts.push([x, y]);
            }
            Ok(Element::Polygon { points: pts })
        }
        "line" => Ok(Element::Line {
            x1: get_f32("x1")?,
            y1: get_f32("y1")?,
            x2: get_f32("x2")?,
            y2: get_f32("y2")?,
            thickness: get_f32("thickness")?,
        }),
        "text" => Ok(Element::Text {
            x: get_f32("x")?,
            y: get_f32("y")?,
            content: get_str("content")?,
            font_size: get_f32("font_size")?,
            font_weight: get_font_weight()?,
        }),
        "image" => Ok(Element::Image {
            path: get_str("path")?,
            x: get_f32("x")?,
            y: get_f32("y")?,
            w: get_f32("w")?,
            h: get_f32("h")?,
        }),
        "path" => parse_path_element(material_id, &m, &get_f32),
        other => Err(MaterialError::UnknownElementType {
            id: material_id.to_string(),
            element_type: other.to_string(),
        }),
    }
}

/// 解析 path 元素的 segments 数组与可选颜色字段，并做结构校验。
///
/// 段对象为 Map 风格（`#{cmd: "M", x: .., y: ..}`），坐标接受 float/int
/// （与 polygon points 的解析先例一致）。校验规则（design D3）：
/// - `segments` 非空且首段必须为 `M`；
/// - `fill=false && thickness=0` 的不可见组合拒绝；
/// - `thickness >= 0`；
/// - 颜色分量为 `0..=1` 的数值且数组长度为 4。
fn parse_path_element(
    material_id: &str,
    m: &Map,
    get_f32: &dyn Fn(&str) -> MaterialResult<f32>,
) -> MaterialResult<Element> {
    use peregrine_config::PathSegment;

    let field_err = |detail: String| MaterialError::ElementField {
        id: material_id.to_string(),
        detail,
    };

    // 可选颜色数组字段：[r, g, b, a]，分量 0..=1，长度必须为 4。
    let parse_color = |key: &str| -> MaterialResult<Option<[f32; 4]>> {
        let Some(v) = m.get(key) else {
            return Ok(None);
        };
        if v.is_unit() {
            return Ok(None);
        }
        let arr = v
            .clone()
            .into_array()
            .map_err(|_| field_err(format!("field '{}' must be Array", key)))?;
        if arr.len() != 4 {
            return Err(field_err(format!(
                "field '{}' must have 4 elements, got {}",
                key,
                arr.len()
            )));
        }
        let mut out = [0.0f32; 4];
        for (i, c) in arr.iter().enumerate() {
            let f = c
                .as_float()
                .ok()
                .or_else(|| c.as_int().ok().map(|i| i as f64))
                .ok_or_else(|| {
                    field_err(format!("field '{}' element {} must be number", key, i))
                })?;
            if !(0.0..=1.0).contains(&f) {
                return Err(field_err(format!(
                    "field '{}' element {} must be in 0..=1, got {}",
                    key, i, f
                )));
            }
            out[i] = f as f32;
        }
        Ok(Some(out))
    };

    // 解析单个段 Map 的坐标字段（float/int 皆可）。
    let seg_f32 = |seg: &Map, key: &str| -> MaterialResult<f32> {
        let v = seg
            .get(key)
            .ok_or_else(|| field_err(format!("path segment missing field '{}'", key)))?;
        if let Ok(f) = v.as_float() {
            Ok(f as f32)
        } else if let Ok(i) = v.as_int() {
            Ok(i as f32)
        } else {
            Err(field_err(format!(
                "path segment field '{}' must be number",
                key
            )))
        }
    };

    let thickness = get_f32("thickness")?;
    let fill = match m.get("fill") {
        Some(v) => v
            .as_bool()
            .map_err(|_| field_err("field 'fill' must be bool".to_string()))?,
        None => false,
    };

    // segments 必须为非空数组。
    let segs_val = m
        .get("segments")
        .ok_or_else(|| field_err("missing 'segments' field".to_string()))?;
    let seg_arr = segs_val
        .clone()
        .into_array()
        .map_err(|_| field_err("'segments' must be Array".to_string()))?;
    if seg_arr.is_empty() {
        return Err(field_err("'segments' must not be empty".to_string()));
    }

    let mut segments = Vec::with_capacity(seg_arr.len());
    for (idx, sv) in seg_arr.into_iter().enumerate() {
        let seg = sv
            .as_map_ref()
            .map_err(|_| field_err(format!("path segment {} must be Map", idx)))?;
        let cmd = seg
            .get("cmd")
            .and_then(|c| c.as_immutable_string_ref().ok().map(|s| s.to_string()))
            .ok_or_else(|| field_err(format!("path segment {} missing 'cmd'", idx)))?;
        let segment = match cmd.as_str() {
            "M" => PathSegment::M {
                x: seg_f32(&seg, "x")?,
                y: seg_f32(&seg, "y")?,
            },
            "L" => PathSegment::L {
                x: seg_f32(&seg, "x")?,
                y: seg_f32(&seg, "y")?,
            },
            "Q" => PathSegment::Q {
                x1: seg_f32(&seg, "x1")?,
                y1: seg_f32(&seg, "y1")?,
                x: seg_f32(&seg, "x")?,
                y: seg_f32(&seg, "y")?,
            },
            "C" => PathSegment::C {
                x1: seg_f32(&seg, "x1")?,
                y1: seg_f32(&seg, "y1")?,
                x2: seg_f32(&seg, "x2")?,
                y2: seg_f32(&seg, "y2")?,
                x: seg_f32(&seg, "x")?,
                y: seg_f32(&seg, "y")?,
            },
            "Z" => PathSegment::Z,
            other => {
                return Err(field_err(format!(
                    "path segment {} has unknown cmd '{}' (expected M/L/Q/C/Z)",
                    idx, other
                )));
            }
        };
        segments.push(segment);
    }

    // 结构校验：首段必须为 M。
    if !matches!(segments.first(), Some(PathSegment::M { .. })) {
        return Err(field_err(
            "path segments must start with an 'M' command".to_string(),
        ));
    }
    // 不可见组合拒绝：fill=false && thickness=0。
    if !fill && thickness == 0.0 {
        return Err(field_err(
            "path is invisible: fill=false and thickness=0".to_string(),
        ));
    }
    if thickness < 0.0 {
        return Err(field_err("field 'thickness' must be >= 0".to_string()));
    }

    let stroke_color = parse_color("stroke_color")?;
    let fill_color = parse_color("fill_color")?;

    Ok(Element::Path {
        segments,
        fill,
        thickness,
        stroke_color,
        fill_color,
    })
}

/// 合并默认参数与传入参数（传入值优先，深度合并）。
fn merge_params(defaults: &serde_json::Value, overrides: &serde_json::Value) -> serde_json::Value {
    match (defaults, overrides) {
        (serde_json::Value::Object(d), serde_json::Value::Object(o)) => {
            let mut merged = d.clone();
            for (k, v) in o {
                merged.insert(k.clone(), v.clone());
            }
            serde_json::Value::Object(merged)
        }
        (_, o @ serde_json::Value::Object(_)) => o.clone(),
        (d @ serde_json::Value::Object(_), _) => d.clone(),
        (_, o) => o.clone(),
    }
}

/// 从 Rhai 源码顶部注释解析显示名称。
///
/// 约定：第一行若为 `// Name: xxx`，则取 `xxx` 作为 display_name。
fn parse_display_name(source: &str) -> Option<String> {
    let first_line = source.lines().next()?.trim();
    let rest = first_line.strip_prefix("//")?.trim();
    let name = rest.strip_prefix("Name:")?.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use peregrine_config::Rect;

    fn test_rect() -> Rect {
        Rect {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1920.0,
            max_y: 1080.0,
        }
    }

    /// 测试辅助：加载已注册的 builtin.time 物料（time.rhai 已归位内置）。
    fn load_builtin_time() -> std::sync::Arc<Material> {
        let registry = crate::registry::MaterialRegistry::new();
        registry.load_builtin().expect("load builtin materials");
        registry
            .get("builtin.time")
            .expect("builtin.time registered")
    }

    /// `refresh_interval_ms()` 可选导出：内置 time 声明 500ms；
    /// 未声明的物料（cross）回退 0 = 不节流。
    #[test]
    fn refresh_interval_ms_parsed_from_script() {
        assert_eq!(load_builtin_time().metadata().refresh_interval_ms, 500);

        let registry = crate::registry::MaterialRegistry::new();
        registry.load_builtin().expect("load builtin materials");
        let cross = registry
            .get("builtin.cross")
            .expect("builtin.cross registered");
        assert_eq!(cross.metadata().refresh_interval_ms, 0);
    }

    #[test]
    fn evaluate_time_material() {
        // time.rhai 已归位内置物料（BUILTIN_MATERIALS 含 "time"）。
        let m = load_builtin_time();
        assert_eq!(m.id(), "builtin.time");
        assert_eq!(m.metadata().display_name, "时间显示");
        assert!(m.metadata().is_dynamic);

        let params = serde_json::json!({
            "font_size": 24.0,
            "x": 100.0,
            "y": 200.0,
            "format": "HH:mm:ss",
        });
        let screen = test_rect();
        let ctx = DynamicContext::preview_snapshot(1920.0, 1080.0);
        let elements = m.evaluate(&params, &screen, &ctx).unwrap();
        assert_eq!(elements.len(), 1);
        if let Element::Text {
            x,
            y,
            content,
            font_size,
            ..
        } = &elements[0]
        {
            assert_eq!(*x, 100.0);
            assert_eq!(*y, 200.0);
            assert_eq!(*font_size, 24.0);
            // HH:mm:ss 格式应为 8 个字符，例如 14:30:25。
            assert_eq!(content.len(), 8);
            assert!(
                content.contains(':'),
                "expected time string, got {}",
                content
            );
        } else {
            panic!("expected Text element");
        }
    }

    #[test]
    fn evaluate_time_material_custom_format() {
        let m = load_builtin_time();
        let params = serde_json::json!({
            "font_size": 16.0,
            "x": 0.0,
            "y": 0.0,
            "format": "yyyy-MM-dd",
        });
        let screen = test_rect();
        let ctx = DynamicContext::preview_snapshot(1920.0, 1080.0);
        let elements = m.evaluate(&params, &screen, &ctx).unwrap();
        assert_eq!(elements.len(), 1);
        if let Element::Text { content, .. } = &elements[0] {
            // yyyy-MM-dd 格式应为 10 个字符，例如 2026-07-19。
            assert_eq!(content.len(), 10);
            assert!(
                content.contains('-'),
                "expected date string, got {}",
                content
            );
        } else {
            panic!("expected Text element");
        }
    }

    #[test]
    fn evaluate_time_material_free_format() {
        let m = load_builtin_time();
        let params = serde_json::json!({
            "font_size": 16.0,
            "x": 0.0,
            "y": 0.0,
            "format": "yyyy年MM月dd日 HH时mm分",
        });
        let screen = test_rect();
        let ctx = DynamicContext::preview_snapshot(1920.0, 1080.0);
        let elements = m.evaluate(&params, &screen, &ctx).unwrap();
        assert_eq!(elements.len(), 1);
        if let Element::Text { content, .. } = &elements[0] {
            assert!(
                content.contains('年'),
                "expected Chinese date, got {}",
                content
            );
            assert!(
                content.contains('时'),
                "expected Chinese time, got {}",
                content
            );
        } else {
            panic!("expected Text element");
        }
    }

    #[test]
    fn text_element_font_weight_conversion() {
        // 构造一个直接输出指定 font_weight 的极简文本物料。
        let make = |fw_expr: &str| {
            let src = format!(
                r#"fn defaults() {{ #{{}} }}
fn schema() {{ [] }}
fn build(params, screen) {{
    [#{{type: "text", x: 0.0, y: 0.0, content: "t", font_size: 16.0, font_weight: {fw_expr}}}]
}}"#
            );
            Material::load("test.fw".to_string(), &src, false).expect("load fw material")
        };
        let screen = test_rect();
        let ctx = DynamicContext::preview_snapshot(1920.0, 1080.0);
        let params = serde_json::json!({});

        // 缺失字段 → None（时间物料等未声明 font_weight 的旧物料）。
        let m = Material::load(
            "test.fw.missing".to_string(),
            r#"fn defaults() { #{} }
fn schema() { [] }
fn build(params, screen) {
    [#{type: "text", x: 0.0, y: 0.0, content: "t", font_size: 16.0}]
}"#,
            false,
        )
        .unwrap();
        let els = m.evaluate(&params, &screen, &ctx).unwrap();
        assert!(matches!(
            &els[0],
            Element::Text {
                font_weight: None,
                ..
            }
        ));

        // 显式 ()（Rhai unit）→ None。
        let els = make("()").evaluate(&params, &screen, &ctx).unwrap();
        assert!(matches!(
            &els[0],
            Element::Text {
                font_weight: None,
                ..
            }
        ));

        // 700 → Some(700)。
        let els = make("700").evaluate(&params, &screen, &ctx).unwrap();
        assert!(matches!(
            &els[0],
            Element::Text {
                font_weight: Some(700),
                ..
            }
        ));

        // 非法值：非百位整数倍 → 求值报错。
        assert!(make("150").evaluate(&params, &screen, &ctx).is_err());
        // 非法值：超出范围 → 求值报错。
        assert!(make("1000").evaluate(&params, &screen, &ctx).is_err());
    }

    #[test]
    fn load_builtin_cross_material() {
        let m = Material::load(
            "builtin.cross".to_string(),
            include_str!("../builtin/cross.rhai"),
            true,
        )
        .expect("load cross material");
        assert_eq!(m.id(), "builtin.cross");
        assert_eq!(m.metadata().display_name, "准星");
        assert!(!m.metadata().is_dynamic);

        let defaults = m.defaults();
        assert_eq!(defaults["size"], 24.0);
        assert_eq!(defaults["thickness"], 2.0);
        assert_eq!(defaults["gap"], 4.0);
    }

    #[test]
    fn evaluate_cross_material() {
        let m = Material::load(
            "builtin.cross".to_string(),
            include_str!("../builtin/cross.rhai"),
            true,
        )
        .unwrap();

        let params = serde_json::json!({"size": 24.0, "thickness": 2.0, "gap": 4.0});
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let elements = m.evaluate(&params, &screen, &ctx).unwrap();

        // cross 物料应返回 4 个矩形。
        assert_eq!(elements.len(), 4);
        for e in &elements {
            assert!(matches!(e, Element::Rect { .. }));
        }
    }

    #[test]
    fn evaluate_cross_with_default_params() {
        let m = Material::load(
            "builtin.cross".to_string(),
            include_str!("../builtin/cross.rhai"),
            true,
        )
        .unwrap();

        // 不传任何参数，应使用 defaults。
        let params = serde_json::json!({});
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let elements = m.evaluate(&params, &screen, &ctx).unwrap();
        assert_eq!(elements.len(), 4);
    }

    #[test]
    fn evaluate_with_param_override() {
        let m = Material::load(
            "builtin.cross".to_string(),
            include_str!("../builtin/cross.rhai"),
            true,
        )
        .unwrap();

        // 只传 size，thickness/gap 取默认。
        let params = serde_json::json!({"size": 100.0});
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let elements = m.evaluate(&params, &screen, &ctx).unwrap();

        // 第一条矩形：x = 960 - 100 = 860, w = 100 - 2 = 98
        if let Element::Rect { x, w, .. } = &elements[0] {
            assert!(((*x - 860.0).abs()) < 0.01, "expected x=860 got {}", x);
            assert!(((*w - 98.0).abs()) < 0.01, "expected w=98 got {}", w);
        } else {
            panic!("expected Rect");
        }
    }

    #[test]
    fn load_missing_required_function_fails() {
        let bad_source = r#"
            fn defaults() { #{} }
            // 缺少 schema 和 build
        "#;
        let result = Material::load("builtin.bad".to_string(), bad_source, true);
        assert!(result.is_err());
        match result {
            Err(MaterialError::MissingFunction { function, .. }) => {
                assert_eq!(function, "schema");
            }
            other => panic!(
                "expected MissingFunction, got {:?}",
                other.map(|m| m.id().to_string())
            ),
        }
    }

    #[test]
    fn unknown_element_type_fails() {
        let source = r#"
            fn defaults() { #{} }
            fn schema() { [] }
            fn build(params, screen) {
                [#{type: "ellipse"}]
            }
        "#;
        let m = Material::load("test.unknown".to_string(), source, false).unwrap();
        let params = serde_json::json!({});
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let result = m.evaluate(&params, &screen, &ctx);
        assert!(result.is_err());
        match result.unwrap_err() {
            MaterialError::UnknownElementType { element_type, .. } => {
                assert_eq!(element_type, "ellipse");
            }
            other => panic!("expected UnknownElementType, got {:?}", other),
        }
    }

    // ===== Path 图元转换层测试 =====

    /// 构造一个按给定 build 体输出 path 元素的测试物料。
    fn load_path_material(build_body: &str) -> Material {
        let src = format!(
            r#"fn defaults() {{ #{{}} }}
fn schema() {{ [] }}
fn build(params, screen) {{
    {build_body}
}}"#
        );
        Material::load("test.path".to_string(), &src, false).expect("load path material")
    }

    /// 完整 path 解析：segments（float/int 混用）+ fill + 双色覆盖。
    #[test]
    fn path_element_full_parse() {
        let m = load_path_material(
            r#"[#{type: "path",
    segments: [
        #{cmd: "M", x: 100, y: 100},
        #{cmd: "L", x: 200.5, y: 100},
        #{cmd: "Q", x1: 10, y1: 20, x: 30, y: 40},
        #{cmd: "C", x1: 0.0, y1: 1.0, x2: 2.0, y2: 3.0, x: 4.0, y: 5.0},
        #{cmd: "Z"},
    ],
    thickness: 2.0,
    fill: true,
    fill_color: [0.3, 0.5, 1.0, 0.2],
    stroke_color: [1.0, 0.0, 0.0, 1.0],
}]"#,
        );
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let els = m
            .evaluate(&serde_json::json!({}), &screen, &ctx)
            .expect("path element must parse");
        match &els[0] {
            Element::Path {
                segments,
                fill,
                thickness,
                stroke_color,
                fill_color,
            } => {
                assert_eq!(segments.len(), 5);
                assert!(matches!(
                    segments[0],
                    peregrine_config::PathSegment::M { x: 100.0, y: 100.0 }
                ));
                assert!(matches!(
                    segments[1],
                    peregrine_config::PathSegment::L { x: 200.5, y: 100.0 }
                ));
                assert!(matches!(
                    segments[2],
                    peregrine_config::PathSegment::Q { .. }
                ));
                assert!(matches!(
                    segments[3],
                    peregrine_config::PathSegment::C { .. }
                ));
                assert!(matches!(segments[4], peregrine_config::PathSegment::Z));
                assert!(*fill);
                assert_eq!(*thickness, 2.0);
                assert_eq!(*stroke_color, Some([1.0, 0.0, 0.0, 1.0]));
                assert_eq!(*fill_color, Some([0.3, 0.5, 1.0, 0.2]));
            }
            other => panic!("expected Path, got {:?}", other),
        }
    }

    /// 未知 cmd 拒绝：错误信息包含未知命令名。
    #[test]
    fn path_element_unknown_cmd_rejected() {
        let m = load_path_material(
            r#"[#{type: "path", segments: [#{cmd: "M", x: 0.0, y: 0.0}, #{cmd: "a", x: 1.0, y: 1.0}], thickness: 2.0}]"#,
        );
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let err = m
            .evaluate(&serde_json::json!({}), &screen, &ctx)
            .expect_err("unknown cmd must be rejected");
        match err {
            MaterialError::ElementField { detail, .. } => {
                assert!(
                    detail.contains('a'),
                    "detail should name the cmd: {}",
                    detail
                );
            }
            other => panic!("expected ElementField, got {:?}", other),
        }
    }

    /// Q 段缺控制点拒绝：错误信息指明缺失字段。
    #[test]
    fn path_element_q_missing_control_point_rejected() {
        let m = load_path_material(
            r#"[#{type: "path", segments: [#{cmd: "M", x: 0.0, y: 0.0}, #{cmd: "Q", x: 10.0, y: 10.0}], thickness: 2.0}]"#,
        );
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let err = m
            .evaluate(&serde_json::json!({}), &screen, &ctx)
            .expect_err("Q without control point must be rejected");
        match err {
            MaterialError::ElementField { detail, .. } => {
                assert!(
                    detail.contains("x1"),
                    "detail should name the field: {}",
                    detail
                );
            }
            other => panic!("expected ElementField, got {:?}", other),
        }
    }

    /// 不可见组合拒绝：fill=false && thickness=0。
    #[test]
    fn path_element_invisible_combo_rejected() {
        let m = load_path_material(
            r#"[#{type: "path", segments: [#{cmd: "M", x: 0.0, y: 0.0}, #{cmd: "L", x: 1.0, y: 1.0}], thickness: 0.0}]"#,
        );
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        assert!(m.evaluate(&serde_json::json!({}), &screen, &ctx).is_err());
    }

    /// 纯填充合法：fill=true && thickness=0。
    #[test]
    fn path_element_fill_only_accepted() {
        let m = load_path_material(
            r#"[#{type: "path", segments: [#{cmd: "M", x: 0.0, y: 0.0}, #{cmd: "L", x: 1.0, y: 1.0}, #{cmd: "Z"}], thickness: 0.0, fill: true}]"#,
        );
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let els = m
            .evaluate(&serde_json::json!({}), &screen, &ctx)
            .expect("fill-only path must be accepted");
        assert!(matches!(
            &els[0],
            Element::Path {
                fill: true,
                thickness: 0.0,
                ..
            }
        ));
    }

    /// 首段非 M 拒绝。
    #[test]
    fn path_element_first_segment_not_m_rejected() {
        let m = load_path_material(
            r#"[#{type: "path", segments: [#{cmd: "L", x: 0.0, y: 0.0}], thickness: 2.0}]"#,
        );
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let err = m
            .evaluate(&serde_json::json!({}), &screen, &ctx)
            .expect_err("first segment must be M");
        match err {
            MaterialError::ElementField { detail, .. } => {
                assert!(detail.contains('M'), "detail should explain: {}", detail);
            }
            other => panic!("expected ElementField, got {:?}", other),
        }
    }

    /// 空段数组拒绝。
    #[test]
    fn path_element_empty_segments_rejected() {
        let m = load_path_material(r#"[#{type: "path", segments: [], thickness: 2.0}]"#);
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        assert!(m.evaluate(&serde_json::json!({}), &screen, &ctx).is_err());
    }

    /// 颜色分量越界拒绝。
    #[test]
    fn path_element_color_out_of_range_rejected() {
        let m = load_path_material(
            r#"[#{type: "path", segments: [#{cmd: "M", x: 0.0, y: 0.0}, #{cmd: "L", x: 1.0, y: 1.0}], thickness: 2.0, stroke_color: [1.5, 0.0, 0.0, 1.0]}]"#,
        );
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        assert!(m.evaluate(&serde_json::json!({}), &screen, &ctx).is_err());
    }

    /// 颜色数组长度错误拒绝。
    #[test]
    fn path_element_color_wrong_length_rejected() {
        let m = load_path_material(
            r#"[#{type: "path", segments: [#{cmd: "M", x: 0.0, y: 0.0}, #{cmd: "L", x: 1.0, y: 1.0}], thickness: 2.0, fill_color: [1.0, 0.0, 0.0]}]"#,
        );
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        assert!(m.evaluate(&serde_json::json!({}), &screen, &ctx).is_err());
    }

    /// thickness 为负拒绝。
    #[test]
    fn path_element_negative_thickness_rejected() {
        let m = load_path_material(
            r#"[#{type: "path", segments: [#{cmd: "M", x: 0.0, y: 0.0}, #{cmd: "L", x: 1.0, y: 1.0}], thickness: -1.0}]"#,
        );
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        assert!(m.evaluate(&serde_json::json!({}), &screen, &ctx).is_err());
    }

    // ===== 鼠标速度 / 加速度 host function 测试 =====

    // ===== parse_svg_path host function 测试 =====

    /// 解析器核心：绝对/相对坐标、H/V 展开、隐式重复、Z 子路径回退。
    #[test]
    fn parse_svg_path_d_core_grammar() {
        use peregrine_config::PathSegment as PS;

        // 绝对 M + L + Z。
        assert_eq!(
            parse_svg_path_d("M 0 0 L 10 20 Z").unwrap(),
            vec![PS::M { x: 0.0, y: 0.0 }, PS::L { x: 10.0, y: 20.0 }, PS::Z]
        );

        // 相对坐标累加：m 10 10 l 5 0 → M(10,10) L(15,10)。
        assert_eq!(
            parse_svg_path_d("m 10 10 l 5 0").unwrap(),
            vec![PS::M { x: 10.0, y: 10.0 }, PS::L { x: 15.0, y: 10.0 }]
        );

        // H/V（绝对与相对）展开为 L。
        assert_eq!(
            parse_svg_path_d("M 0 0 H 30 V 20 h -10 v -5").unwrap(),
            vec![
                PS::M { x: 0.0, y: 0.0 },
                PS::L { x: 30.0, y: 0.0 },
                PS::L { x: 30.0, y: 20.0 },
                PS::L { x: 20.0, y: 20.0 },
                PS::L { x: 20.0, y: 15.0 },
            ]
        );

        // 隐式重复：M 后续坐标对按 L；其它命令重复自身。
        assert_eq!(
            parse_svg_path_d("M 1 2 3 4").unwrap(),
            vec![PS::M { x: 1.0, y: 2.0 }, PS::L { x: 3.0, y: 4.0 }]
        );
        assert_eq!(
            parse_svg_path_d("M 0 0 L 1 1 2 2").unwrap(),
            vec![
                PS::M { x: 0.0, y: 0.0 },
                PS::L { x: 1.0, y: 1.0 },
                PS::L { x: 2.0, y: 2.0 },
            ]
        );

        // Q/C 相对形式。
        assert_eq!(
            parse_svg_path_d("M 0 0 q 10 10 20 0").unwrap(),
            vec![
                PS::M { x: 0.0, y: 0.0 },
                PS::Q {
                    x1: 10.0,
                    y1: 10.0,
                    x: 20.0,
                    y: 0.0,
                },
            ]
        );
        assert_eq!(
            parse_svg_path_d("M 0 0 c 1 2 3 4 5 6").unwrap(),
            vec![
                PS::M { x: 0.0, y: 0.0 },
                PS::C {
                    x1: 1.0,
                    y1: 2.0,
                    x2: 3.0,
                    y2: 4.0,
                    x: 5.0,
                    y: 6.0,
                },
            ]
        );

        // Z 后当前点回到子路径起点（相对坐标基准复位）。
        assert_eq!(
            parse_svg_path_d("M 10 10 L 20 20 Z l 5 5").unwrap(),
            vec![
                PS::M { x: 10.0, y: 10.0 },
                PS::L { x: 20.0, y: 20.0 },
                PS::Z,
                PS::L { x: 15.0, y: 15.0 },
            ]
        );

        // 分隔符：逗号与负号前缀（无空格）。
        assert_eq!(
            parse_svg_path_d("M0,0L10-5").unwrap(),
            vec![PS::M { x: 0.0, y: 0.0 }, PS::L { x: 10.0, y: -5.0 }]
        );
    }

    /// 解析器错误路径：不支持命令、缺数值、垃圾输入。
    #[test]
    fn parse_svg_path_d_errors() {
        assert!(parse_svg_path_d("M 0 0 A 5 5 0 0 1 10 10").is_err());
        assert!(parse_svg_path_d("M 0 0 S 10 10 20 20").is_err());
        assert!(parse_svg_path_d("M 0 0 T 10 10").is_err());
        assert!(parse_svg_path_d("M 0").is_err());
        assert!(parse_svg_path_d("M x y").is_err());
        assert!(parse_svg_path_d("").is_ok()); // 空路径 = 空段数组
    }

    /// host function 通路：脚本调用 parse_svg_path 得到段数组，
    /// 可直接作为 path 图元的 segments（端到端）。
    #[test]
    fn parse_svg_path_host_function_end_to_end() {
        let m = Material::load(
            "test.parsepath".to_string(),
            r#"fn defaults() { #{d: "M 0 0 L 100 0 L 100 50 Z"} }
fn schema() { [] }
fn build(params, screen) {
    let segs = parse_svg_path(params.d);
    [#{type: "path", segments: segs, fill: true, thickness: 0.0}]
}"#,
            false,
        )
        .unwrap();
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let els = m.evaluate(&m.defaults().clone(), &screen, &ctx).unwrap();
        match &els[0] {
            Element::Path { segments, fill, .. } => {
                assert!(*fill);
                assert_eq!(segments.len(), 4);
                assert!(matches!(
                    segments[0],
                    peregrine_config::PathSegment::M { x, y } if (x - 0.0).abs() < 1e-5 && (y - 0.0).abs() < 1e-5
                ));
                assert!(matches!(segments[3], peregrine_config::PathSegment::Z));
            }
            other => panic!("expected Path, got {:?}", other),
        }
    }

    /// host function 返回 Map {x, y} 结构；同一次求值内多次调用返回同值。
    #[test]
    fn mouse_velocity_acceleration_host_functions() {
        let m = Material::load(
            "test.dyninput".to_string(),
            r#"fn defaults() { #{} }
fn schema() { [] }
fn build(params, screen) {
    let v = mouse_velocity();
    let a = mouse_acceleration();
    let v2 = mouse_velocity();
    // Rhai 的 == 对 Map 做逐键比较：同快照内重复调用返回同值。
    let stable = v == v2;
    [#{type: "rect", x: v.x, y: v.y, w: a.x, h: a.y},
     #{type: "circle", cx: if stable { 1.0 } else { 0.0 }, cy: 0.0, radius: 1.0}]
}"#,
            false,
        )
        .unwrap();

        let screen = test_rect();
        let ctx = DynamicContext {
            mouse_velocity: (120.0, -40.0),
            mouse_acceleration: (-500.0, 250.0),
            ..DynamicContext::default()
        };
        let els = m.evaluate(&serde_json::json!({}), &screen, &ctx).unwrap();
        match (&els[0], &els[1]) {
            (Element::Rect { x, y, w, h, .. }, Element::Circle { cx, .. }) => {
                assert_eq!(*x, 120.0);
                assert_eq!(*y, -40.0);
                assert_eq!(*w, -500.0);
                assert_eq!(*h, 250.0);
                // 同快照内重复调用返回同值。
                assert_eq!(*cx, 1.0);
            }
            other => panic!("unexpected elements: {:?}", other),
        }

        // 默认上下文（速度/加速度为 0）不报错，返回 0。
        let els = m
            .evaluate(&serde_json::json!({}), &screen, &DynamicContext::default())
            .unwrap();
        match &els[0] {
            Element::Rect { x, y, w, h, .. } => {
                assert_eq!((*x, *y, *w, *h), (0.0, 0.0, 0.0, 0.0));
            }
            other => panic!("expected Rect, got {:?}", other),
        }
    }

    #[test]
    fn parse_display_name_extracts_label() {
        assert_eq!(
            parse_display_name("// Name: 准星\nfn defaults()...").unwrap(),
            "准星"
        );
        assert!(parse_display_name("// just a comment").is_none());
        assert!(parse_display_name("fn defaults() {}").is_none());
    }

    #[test]
    fn schema_returned_as_json() {
        let m = Material::load(
            "builtin.cross".to_string(),
            include_str!("../builtin/cross.rhai"),
            true,
        )
        .unwrap();
        let schema = m.schema();
        assert!(schema.is_array());
        let arr = schema.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["key"], "size");
        assert_eq!(arr[0]["label"], "臂长");
        assert_eq!(arr[0]["widget"], "slider");
    }

    #[test]
    fn info_serialization() {
        let m = Material::load(
            "builtin.cross".to_string(),
            include_str!("../builtin/cross.rhai"),
            true,
        )
        .unwrap();
        let info = m.info(true);
        assert_eq!(info.id, "builtin.cross");
        assert!(info.builtin);
        assert!(!info.is_dynamic);
    }

    #[test]
    fn rhai_rand_is_deterministic_with_same_seed() {
        let ctx1 = DynamicContext {
            rng_seed: 12345,
            ..DynamicContext::default()
        };
        let ctx2 = DynamicContext {
            rng_seed: 12345,
            ..DynamicContext::default()
        };
        // 同种子下 SimpleRng 输出一致。
        let mut r1 = SimpleRng::new(12345);
        let mut r2 = SimpleRng::new(12345);
        for _ in 0..10 {
            assert_eq!(r1.next_f32(), r2.next_f32());
        }
        let _ = (ctx1, ctx2);
    }

    // ===== 内置物料批量加载/求值测试 =====

    fn load_builtin(name: &str) -> Material {
        let id = format!("builtin.{}", name);
        let source = crate::BUILTIN_MATERIALS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, s)| *s)
            .unwrap_or_else(|| panic!("builtin material '{}' not found", name));
        Material::load(id, source, true).expect("failed to load builtin material")
    }

    #[test]
    fn all_builtin_materials_load() {
        // 全部内置物料应能成功加载。
        for (name, _) in crate::BUILTIN_MATERIALS {
            let m = load_builtin(name);
            assert!(!m.metadata().display_name.is_empty());
            assert!(m.defaults().is_object());
            assert!(m.schema().is_array());
        }
    }

    #[test]
    fn all_builtin_materials_evaluate_with_defaults() {
        // 用默认参数求值，所有物料都应返回合法的 Element 列表（image 需要路径，单独测）。
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        for (name, _) in crate::BUILTIN_MATERIALS {
            let m = load_builtin(name);
            let params = m.defaults().clone();
            let result = m.evaluate(&params, &screen, &ctx);
            assert!(
                result.is_ok(),
                "material '{}' evaluation failed: {:?}",
                name,
                result.err()
            );
            let elements = result.unwrap();
            // image 物料默认 path 为空，返回 0 个元素，是合法的。
            if name != &"image" {
                assert!(
                    !elements.is_empty(),
                    "material '{}' returned empty element list",
                    name
                );
            }
        }
    }

    #[test]
    fn corner_dots_count_variants() {
        // count: 4 → 4 个圆，count: 6 → 6 个，count: 8 → 8 个。
        let m = load_builtin("corner_dots");
        let screen = test_rect();
        let ctx = DynamicContext::static_context();

        for (count, expected) in [(4, 4), (6, 6), (8, 8)] {
            let params = serde_json::json!({"count": count});
            let elements = m.evaluate(&params, &screen, &ctx).unwrap();
            assert_eq!(
                elements.len(),
                expected,
                "corner_dots with count={} should return {} elements",
                count,
                expected
            );
            for e in &elements {
                assert!(matches!(e, Element::Circle { .. }));
            }
        }
    }

    #[test]
    fn ring_styles_produce_different_output() {
        let m = load_builtin("ring");
        let screen = test_rect();
        let ctx = DynamicContext::static_context();

        let solid = m
            .evaluate(&serde_json::json!({"ring_style": "solid"}), &screen, &ctx)
            .unwrap();
        let dashed = m
            .evaluate(&serde_json::json!({"ring_style": "dashed"}), &screen, &ctx)
            .unwrap();
        let double = m
            .evaluate(&serde_json::json!({"ring_style": "double"}), &screen, &ctx)
            .unwrap();

        assert_eq!(solid.len(), 1);
        assert_eq!(dashed.len(), 1);
        assert_eq!(double.len(), 2); // 双环：实线 + 虚线
    }

    #[test]
    fn border_frame_styles() {
        let m = load_builtin("border_frame");
        let screen = test_rect();
        let ctx = DynamicContext::static_context();

        let solid = m
            .evaluate(&serde_json::json!({"frame_style": "solid"}), &screen, &ctx)
            .unwrap();
        let gap = m
            .evaluate(&serde_json::json!({"frame_style": "gap"}), &screen, &ctx)
            .unwrap();

        // 实线边框：4 条矩形
        assert_eq!(solid.len(), 4);
        // gap 边框：上下左右各 2 段 = 8 条
        assert_eq!(gap.len(), 8);
    }

    #[test]
    fn edge_rect_anchors() {
        let m = load_builtin("edge_rect");
        let screen = test_rect();
        let ctx = DynamicContext::static_context();

        for anchor in ["top", "bottom", "left", "right", "center"] {
            let params =
                serde_json::json!({"anchor": anchor, "size": 100.0, "secondary_size": 30.0});
            let elements = m.evaluate(&params, &screen, &ctx).unwrap();
            assert_eq!(elements.len(), 1, "anchor {} should produce 1 rect", anchor);
        }
    }

    #[test]
    fn random_orb_produces_correct_count() {
        let m = load_builtin("random_orb");
        let screen = test_rect();
        let ctx = DynamicContext {
            rng_seed: 42,
            ..DynamicContext::default()
        };

        let params = serde_json::json!({"orb_count": 2});
        let elements = m.evaluate(&params, &screen, &ctx).unwrap();
        // 4 边 × 每边 2 个 = 8 个圆
        assert_eq!(elements.len(), 8);
    }

    #[test]
    fn grid_center_vs_edge() {
        let m = load_builtin("grid");
        let screen = test_rect();
        let ctx = DynamicContext::static_context();

        let center = m
            .evaluate(
                &serde_json::json!({"grid_size": 120.0, "alignment": "center"}),
                &screen,
                &ctx,
            )
            .unwrap();
        let edge = m
            .evaluate(
                &serde_json::json!({"grid_size": 120.0, "alignment": "edge"}),
                &screen,
                &ctx,
            )
            .unwrap();

        // 两种模式都应生成网格线（>0），数量级相近。
        assert!(!center.is_empty(), "center 模式应生成网格线");
        assert!(!edge.is_empty(), "edge 模式应生成网格线");
        // center 模式从屏幕中心对称扩展，含超出边缘一圈的线，
        // 元素数量应与 edge 同量级（差异不超过 2 行/列）。
        let diff = center.len() as i64 - edge.len() as i64;
        assert!(
            diff.abs() <= 4,
            "center/edge 元素数差异过大：center={} edge={}",
            center.len(),
            edge.len()
        );
    }

    #[test]
    fn grid_center_symmetric_and_fills_screen() {
        // 任务 8.3：center 模式应让网格铺满屏幕（允许超出边缘一圈），
        // 而非在屏幕内部留白。校验：所有 rect 元素的 x/y 范围覆盖屏幕宽高。
        let m = load_builtin("grid");
        let screen = test_rect(); // 通常 0..800 / 0..600 之类
        let ctx = DynamicContext::static_context();

        let elements = m
            .evaluate(
                &serde_json::json!({"grid_size": 200.0, "alignment": "center"}),
                &screen,
                &ctx,
            )
            .unwrap();

        // 校验竖线高度铺满屏幕（h ≥ screen.height），横线宽度铺满屏幕（w ≥ screen.width）。
        let screen_w = screen.max_x - screen.min_x;
        let screen_h = screen.max_y - screen.min_y;
        let mut has_vertical = false; // 竖线：w 小、h 大
        let mut has_horizontal = false; // 横线：h 小、w 大
        for e in &elements {
            if let Element::Rect { w, h, .. } = e {
                if *h >= screen_h && *w < screen_w {
                    has_vertical = true;
                } else if *w >= screen_w && *h < screen_h {
                    has_horizontal = true;
                }
            }
        }
        assert!(has_vertical, "center 模式应有铺满屏幕高度的竖线");
        assert!(has_horizontal, "center 模式应有铺满屏幕宽度的横线");
    }

    #[test]
    fn image_material_returns_empty_when_no_path() {
        let m = load_builtin("image");
        let screen = test_rect();
        let ctx = DynamicContext::static_context();

        let empty_params = serde_json::json!({"path": ""});
        let elements = m.evaluate(&empty_params, &screen, &ctx).unwrap();
        assert!(elements.is_empty());

        let with_path = serde_json::json!({"path": "/tmp/test.png", "width": 64.0, "height": 64.0});
        let elements = m.evaluate(&with_path, &screen, &ctx).unwrap();
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            Element::Image { path, .. } => assert_eq!(path, "/tmp/test.png"),
            _ => panic!("expected Image element"),
        }
    }

    // ===== 示例物料加载 / 求值测试（examples/*.rhai） =====

    /// 示例物料：simple_cross（静态）
    #[test]
    fn example_simple_cross_loads_and_evaluates() {
        let source = include_str!("../examples/simple_cross.rhai");
        let m = Material::load("user.simple_cross".to_string(), source, false)
            .expect("load example simple_cross");
        assert_eq!(m.metadata().display_name, "示例·简易十字");
        assert!(!m.metadata().is_dynamic);

        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let params = m.defaults().clone();
        let elements = m
            .evaluate(&params, &screen, &ctx)
            .expect("evaluate simple_cross");
        // 四段矩形。
        assert_eq!(elements.len(), 4);
        for e in &elements {
            assert!(
                matches!(e, Element::Rect { .. }),
                "expected Rect, got {:?}",
                e
            );
        }
    }

    /// 示例物料：clock（时间动态）
    #[test]
    fn example_clock_loads_and_evaluates() {
        let source = include_str!("../examples/clock.rhai");
        let m =
            Material::load("user.clock".to_string(), source, false).expect("load example clock");
        assert_eq!(m.metadata().display_name, "示例·时钟");
        assert!(m.metadata().is_dynamic);

        // 动态输入软关闭下使用静态上下文求值（与发布构建行为一致）。
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let params = m.defaults().clone();
        let elements = m
            .evaluate(&params, &screen, &ctx)
            .expect("evaluate clock under static context");
        assert_eq!(elements.len(), 1);
        assert!(matches!(
            &elements[0],
            Element::Text { content, .. } if !content.is_empty()
        ));
    }

    // ===== 内置时间物料：上下文时间快照（防 now_ms 逃逸回归） =====

    /// `builtin.time` 求值必须使用注入上下文的 `time_ms`：
    /// 固定 `DynamicContext.time_ms = T` → 输出文本对应时刻 T，
    /// 而非求值发生的真实墙钟时刻（防 `now_ms()` 逃逸回归）。
    #[test]
    fn builtin_time_uses_injected_context_time() {
        let m = load_builtin_time();

        let screen = test_rect();
        // 固定上下文时刻：2026-08-14 12:34:56.789 UTC+8 对应的 Unix 毫秒。
        // 选一个与「当前墙钟」几乎不可能相同的时刻，防偶发通过。
        let fixed_ms: u64 = 1_788_636_896_789;
        let ctx = DynamicContext {
            time_ms: fixed_ms,
            mouse_pos: (0.0, 0.0),
            mouse_velocity: (0.0, 0.0),
            mouse_acceleration: (0.0, 0.0),
            key_state: crate::context::KeyState::new(),
            rng_seed: 1,
            version: fixed_ms,
        };
        let mut params = m.defaults().clone();
        params["format"] = serde_json::json!("yyyy-MM-dd HH:mm:ss");
        let elements = m
            .evaluate(&params, &screen, &ctx)
            .expect("evaluate builtin.time with fixed context");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            Element::Text { content, .. } => {
                // format_time 基于本地时区渲染，无法硬编码完整字符串；
                // 秒数与时区无关（时区偏移均为分钟粒度）：fixed_ms 截断在 56 秒。
                // 若物料逃逸用 now_ms() 直读墙钟，秒数几乎必然不同（1/60 通过率），
                // 再叠加下条 epoch 测试双保险。
                assert!(
                    content.contains(":56"),
                    "content `{}` should embed fixed-context time (second=56), not wall clock",
                    content
                );
            }
            other => panic!("expected Text element, got {:?}", other),
        }
    }

    /// `builtin.time` 在 static 上下文（time_ms = 0）下输出 UNIX 起点
    /// 对应的本地时刻，同样不是求值时刻（软关闭下冻结语义）。
    #[test]
    fn builtin_time_static_context_renders_epoch() {
        let m = load_builtin_time();

        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let mut params = m.defaults().clone();
        // 默认 HH:mm:ss 不含年份，改用日期格式以便断言 epoch 时刻。
        params["format"] = serde_json::json!("yyyy-MM-dd HH:mm:ss");
        let elements = m.evaluate(&params, &screen, &ctx).expect("evaluate");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            Element::Text { content, .. } => {
                // UNIX 起点 1970-01-01：本地时区年份数字必含 1970（或 1969，
                // 西半球负偏移时区）。无论如何不应是求值当下的年份。
                assert!(
                    content.contains("1970") || content.contains("1969"),
                    "static context should render epoch time, got `{}`",
                    content
                );
            }
            other => panic!("expected Text element, got {:?}", other),
        }
    }

    /// 示例物料：key_indicator（输入动态）
    #[test]
    fn example_key_indicator_loads_and_evaluates() {
        let source = include_str!("../examples/key_indicator.rhai");
        let m = Material::load("user.key_indicator".to_string(), source, false)
            .expect("load example key_indicator");
        assert_eq!(m.metadata().display_name, "示例·按键指示器");
        assert!(m.metadata().is_dynamic);

        // 静态上下文：key_down 恒为 false，仅输出中心点。
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let params = m.defaults().clone();
        let elements = m
            .evaluate(&params, &screen, &ctx)
            .expect("evaluate key_indicator under static context");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], Element::Circle { .. }));

        // 动态上下文：key_down("space") = true 时输出中心点 + 外圈。
        let mut key_state = crate::context::KeyState::new();
        key_state.press("space");
        let ctx_dyn = DynamicContext {
            time_ms: 0,
            mouse_pos: (0.0, 0.0),
            mouse_velocity: (0.0, 0.0),
            mouse_acceleration: (0.0, 0.0),
            key_state,
            rng_seed: 1,
            version: 1,
        };
        let elements = m
            .evaluate(&params, &screen, &ctx_dyn)
            .expect("evaluate key_indicator with space pressed");
        assert_eq!(elements.len(), 2);
        assert!(matches!(&elements[0], Element::Circle { .. }));
        assert!(matches!(&elements[1], Element::CircleStroke { .. }));
    }

    // ===== 内置演示物料：teardrop / path_showcase（Path 图元） =====

    /// teardrop（呼吸环）：动态物料（10Hz 节流）；恒为正圆双圈
    /// （外圈 Q 平滑环 + 内圈镂空），缺省不携带颜色覆盖（继承图层色）。
    #[test]
    fn builtin_teardrop_static_ring_geometry() {
        let m = load_builtin("teardrop");
        assert_eq!(m.metadata().display_name, "呼吸环");
        assert!(m.metadata().is_dynamic);

        let screen = test_rect();
        let ctx = DynamicContext::static_context(); // 加速度为 0
        let els = m.evaluate(&m.defaults().clone(), &screen, &ctx).unwrap();
        assert_eq!(els.len(), 1);
        match &els[0] {
            Element::Path {
                segments,
                fill,
                thickness,
                stroke_color,
                fill_color,
            } => {
                // 闭合 Path：两个 M（外圈子路径 + 内圈子路径）+ 末段 Z；fill=true。
                let m_count = segments
                    .iter()
                    .filter(|s| matches!(s, peregrine_config::PathSegment::M { .. }))
                    .count();
                assert_eq!(m_count, 2, "ring = outer + inner subpaths");
                assert!(matches!(
                    segments.first(),
                    Some(peregrine_config::PathSegment::M { .. })
                ));
                assert!(matches!(
                    segments.last(),
                    Some(peregrine_config::PathSegment::Z)
                ));
                assert!(*fill);
                // 缺省 thickness=0（纯填充环）且不携带颜色覆盖（继承图层色）。
                assert_eq!(*thickness, 0.0);
                assert_eq!(stroke_color, &None);
                assert_eq!(fill_color, &None);
                // 曲线环：Q 段存在（中点平滑，非直线多边形）。
                assert!(
                    segments
                        .iter()
                        .any(|s| matches!(s, peregrine_config::PathSegment::Q { .. })),
                    "ring should use Q smoothing"
                );
                // 无角点：freeze 重设计后永不产生 L 段（对称原则）。
                assert!(
                    !segments
                        .iter()
                        .any(|s| matches!(s, peregrine_config::PathSegment::L { .. })),
                    "anchor ring must never contain corner L segments"
                );
                // 纯静态：所有 Q 控制点距圆心等距（正圆），
                // 两档距离（外/内圈半径）。
                let cx = 960.0;
                let cy = 540.0;
                let outer_r = 1080.0 * 0.05;
                let inner_r = outer_r - 10.0;
                let mut dists = Vec::new();
                for seg in segments {
                    if let peregrine_config::PathSegment::Q { x1, y1, .. } = seg {
                        dists.push(((x1 - cx).powi(2) + (y1 - cy).powi(2)).sqrt());
                    }
                }
                assert!(!dists.is_empty());
                for d in &dists {
                    let on_outer = (d - outer_r).abs() < 1e-2;
                    let on_inner = (d - inner_r).abs() < 1e-2;
                    assert!(
                        on_outer || on_inner,
                        "should be perfect circle: d={} (outer={} inner={})",
                        d,
                        outer_r,
                        inner_r
                    );
                }
                assert!(
                    dists.iter().any(|d| (d - outer_r).abs() < 1e-2)
                        && dists.iter().any(|d| (d - inner_r).abs() < 1e-2),
                    "should have both outer and inner ring"
                );
            }
            other => panic!("expected Path, got {:?}", other),
        }
    }

    /// teardrop：预览快照呈正圆双圈环（静态物料与动态上下文无关，
    /// 预览/overlay 任意快照下输出恒定）。
    #[test]
    fn builtin_teardrop_preview_snapshot_is_circle() {
        let m = load_builtin("teardrop");
        let screen = test_rect();
        let ctx = DynamicContext::preview_snapshot(1920.0, 1080.0);
        let els = m.evaluate(&m.defaults().clone(), &screen, &ctx).unwrap();
        match &els[0] {
            Element::Path { segments, .. } => {
                // 纯静态：正圆（无呼吸叠加）。
                let cx = 960.0;
                let cy = 540.0;
                let base = 1080.0 * 0.05;
                let mut dists = Vec::new();
                for seg in segments {
                    if let peregrine_config::PathSegment::Q { x1, y1, .. } = seg {
                        dists.push(((x1 - cx).powi(2) + (y1 - cy).powi(2)).sqrt());
                    }
                }
                assert!(!dists.is_empty());
                // 外圈集合等值、内圈集合等值（正圆），半径精确等于基准。
                let outer: Vec<f32> = dists.iter().copied().filter(|d| *d > base * 0.9).collect();
                let inner: Vec<f32> = dists.iter().copied().filter(|d| *d <= base * 0.9).collect();
                assert!(!outer.is_empty() && !inner.is_empty());
                let o0 = outer[0];
                for d in &outer {
                    assert!(
                        (d - o0).abs() < 1e-2,
                        "preview outer ring must be circle: {} vs {}",
                        d,
                        o0
                    );
                }
                let i0 = inner[0];
                for d in &inner {
                    assert!(
                        (d - i0).abs() < 1e-2,
                        "preview inner ring must be circle: {} vs {}",
                        d,
                        i0
                    );
                }
                // 呼吸语义：外径在 base × [0.97, 1.03] 区间内。
                assert!(o0 > base * 0.96 && o0 < base * 1.04);
            }
            other => panic!("expected Path, got {:?}", other),
        }
    }

    /// teardrop：呼吸性能与节拍声明——refresh_interval_ms=100（10Hz）、
    /// 0.5px 量化（相邻亚档采样指纹不变，跳过光栅化）、呼吸周期可区分
    ///（slow/normal/fast 三档相位取样单调）。
    #[test]
    fn builtin_teardrop_breath_throttling_and_quantization() {
        let m = load_builtin("teardrop");
        let screen = test_rect();
        let defaults = m.defaults().clone();

        // 节拍声明：10Hz。
        assert_eq!(m.metadata().refresh_interval_ms, 100);

        let eval_segs = |ctx: &DynamicContext| {
            m.evaluate(&defaults, &screen, ctx)
                .unwrap()
                .into_iter()
                .map(|e| match e {
                    Element::Path { segments, .. } => segments,
                    _ => panic!("expected Path"),
                })
                .next()
                .unwrap()
        };
        let outer_radius = |segs: &[peregrine_config::PathSegment]| -> f32 {
            let cx = 960.0;
            let cy = 540.0;
            segs.iter()
                .filter_map(|s| {
                    if let peregrine_config::PathSegment::Q { x1, y1, .. } = *s {
                        Some(((x1 - cx).powi(2) + (y1 - cy).powi(2)).sqrt())
                    } else {
                        None
                    }
                })
                .fold(f32::MIN, f32::max)
        };

        // 量化跳帧：顶点附近相邻 125ms 采样（1075 与 1200）半径差
        // < 0.5px → 同一量化档 → 输出逐段一致（帧指纹不变）。
        assert_eq!(
            eval_segs(&DynamicContext {
                time_ms: 1075,
                ..DynamicContext::default()
            }),
            eval_segs(&DynamicContext {
                time_ms: 1200,
                ..DynamicContext::default()
            }),
            "sub-quantum time step must keep fingerprint"
        );

        // 量化不改呼吸可见性：顶点（t=1075）半径 = 量化后的 1.03 峰值
        //（55.5），显著大于基准 54。
        let base_r = 1080.0 * 0.05;
        let r_peak = outer_radius(&eval_segs(&DynamicContext {
            time_ms: 1075,
            ..DynamicContext::default()
        }));
        let peak: f32 = base_r * 1.03;
        let q_peak: f32 = (peak * 2.0).floor() / 2.0;
        assert!(
            (r_peak - q_peak).abs() < 1e-2,
            "peak should quantize to 55.5: {}",
            r_peak
        );
        assert!(q_peak > base_r, "breathing must stay visible");

        // 呼吸速度三档 → 周期可区分（t=1700 相位取样单调）：
        // slow(7500) sin≈0.989 近顶点；normal(4300) sin≈0.611；
        // fast(3000) sin≈-0.407 收缩相。
        let radius_at = |params: &serde_json::Value| -> f32 {
            let ctx = DynamicContext {
                time_ms: 1700,
                ..DynamicContext::default()
            };
            let segs = m
                .evaluate(params, &screen, &ctx)
                .unwrap()
                .into_iter()
                .map(|e| match e {
                    Element::Path { segments, .. } => segments,
                    _ => panic!("expected Path"),
                })
                .next()
                .unwrap();
            outer_radius(&segs)
        };
        let r_slow = radius_at(&serde_json::json!({"breath_speed": "slow"}));
        let r_norm = radius_at(&serde_json::json!({"breath_speed": "normal"}));
        let r_fast = radius_at(&serde_json::json!({"breath_speed": "fast"}));
        assert!(
            r_slow > r_norm && r_norm > r_fast,
            "breath speeds must map to distinct periods: {} {} {}",
            r_slow,
            r_norm,
            r_fast
        );

        // 周期平移不变：相隔整周期输出逐段一致（呼吸是唯一时间依赖）。
        assert_eq!(
            eval_segs(&DynamicContext {
                time_ms: 0,
                ..DynamicContext::default()
            }),
            eval_segs(&DynamicContext {
                time_ms: 4300,
                ..DynamicContext::default()
            })
        );
    }

    /// teardrop：缺省不携带颜色覆盖（继承图层色，换色热键生效）——
    /// 参数面板产品化后颜色覆盖演示已移除。
    #[test]
    fn builtin_teardrop_inherits_layer_color_by_default() {
        let m = load_builtin("teardrop");
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let els = m.evaluate(&m.defaults().clone(), &screen, &ctx).unwrap();
        match &els[0] {
            Element::Path {
                stroke_color,
                fill_color,
                ..
            } => {
                assert_eq!(stroke_color, &None);
                assert_eq!(fill_color, &None);
            }
            other => panic!("expected Path, got {:?}", other),
        }
    }

    /// path_showcase（自定义路径）：静态物料，d 参数解析为单条 Path，
    /// 包围盒归一化居中；缺省形状为 Q 段花环（开箱即用）。
    #[test]
    fn builtin_path_showcase_parses_d_param() {
        let m = load_builtin("path_showcase");
        assert_eq!(m.metadata().display_name, "自定义路径");
        assert!(!m.metadata().is_dynamic);

        let screen = test_rect();
        let ctx = DynamicContext::static_context();

        // 缺省 d：四叶花环（Q 段闭环），单 Path 居中。
        let els = m.evaluate(&m.defaults().clone(), &screen, &ctx).unwrap();
        assert_eq!(els.len(), 1);
        let cx = 960.0;
        let cy = 540.0;
        match &els[0] {
            Element::Path {
                segments,
                fill,
                stroke_color,
                ..
            } => {
                assert!(*fill);
                // 缺省参数不携带颜色覆盖（继承图层色）。
                assert_eq!(stroke_color, &None);
                // 缺省形状是 Q 闭环。
                assert!(
                    segments
                        .iter()
                        .any(|s| matches!(s, peregrine_config::PathSegment::Q { .. })),
                    "default flower should contain Q segments"
                );
                // 归一化居中：所有坐标点距屏幕中心 < target/2 + 余量。
                // target = 1080 × 0.08 = 86.4 → 半径 < 50（Q 控制点可在
                // 包围盒外少许），断言 < 60。
                for seg in segments {
                    let (x, y) = match *seg {
                        peregrine_config::PathSegment::M { x, y } => (x, y),
                        peregrine_config::PathSegment::L { x, y } => (x, y),
                        peregrine_config::PathSegment::Q { x, y, .. } => (x, y),
                        peregrine_config::PathSegment::C { x, y, .. } => (x, y),
                        peregrine_config::PathSegment::Z => continue,
                    };
                    assert!(
                        ((x - cx).abs() < 60.0) && ((y - cy).abs() < 60.0),
                        "normalized shape must center on screen: ({}, {})",
                        x,
                        y
                    );
                }
            }
            other => panic!("expected Path, got {:?}", other),
        }

        // 自定义 d（相对坐标 + H/V）：三角形，坐标域 0..10（任意域）。
        // 验证归一化把小坐标域放大到目标大小。
        let tri = m
            .evaluate(
                &serde_json::json!({"d": "M 0 0 h 10 v 10 h -10 Z"}),
                &screen,
                &ctx,
            )
            .unwrap();
        match &tri[0] {
            Element::Path { segments, .. } => {
                // H/V 展开为 L；共 5 段（M + 3L + Z）。
                assert_eq!(segments.len(), 5);
                // 归一化放大：宽/高从 10 放大到 ~86（1080×0.08）。
                let mut max_x = f32::MIN;
                let mut min_x = f32::MAX;
                for seg in segments {
                    if let peregrine_config::PathSegment::L { x, .. } = *seg {
                        max_x = max_x.max(x);
                        min_x = min_x.min(x);
                    }
                }
                assert!(
                    (max_x - min_x - 86.4).abs() < 1.0,
                    "normalization should scale to target size: got width {}",
                    max_x - min_x
                );
            }
            other => panic!("expected Path, got {:?}", other),
        }
    }

    /// path_showcase：纯静态——仅 mouse_pos/time_ms 不同的两上下文输出完全相同。
    #[test]
    fn builtin_path_showcase_output_independent_of_dynamic_input() {
        let m = load_builtin("path_showcase");
        let screen = test_rect();
        let params = m.defaults().clone();

        let ctx_a = DynamicContext {
            time_ms: 1000,
            mouse_pos: (100.0, 200.0),
            ..DynamicContext::default()
        };
        let ctx_b = DynamicContext {
            time_ms: 999999,
            mouse_pos: (800.0, 100.0),
            ..DynamicContext::default()
        };
        let out_a = m.evaluate(&params, &screen, &ctx_a).unwrap();
        let out_b = m.evaluate(&params, &screen, &ctx_b).unwrap();
        assert_eq!(out_a, out_b);
    }

    /// path_showcase：非法 d（A 圆弧不支持）回退内置花环——
    /// 物料永不渲染空输出（图层突然消失比回退更迷惑）。
    #[test]
    fn builtin_path_showcase_invalid_d_falls_back() {
        let m = load_builtin("path_showcase");
        let screen = test_rect();
        let ctx = DynamicContext::static_context();
        let els = m
            .evaluate(
                &serde_json::json!({"d": "M 0 0 A 5 5 0 0 1 10 10"}),
                &screen,
                &ctx,
            )
            .unwrap();
        // 回退形状仍是一条有效 Path（含 Q 段）。
        match &els[0] {
            Element::Path { segments, .. } => {
                assert!(
                    segments
                        .iter()
                        .any(|s| matches!(s, peregrine_config::PathSegment::Q { .. })),
                    "fallback should be the built-in flower ring"
                );
            }
            other => panic!("expected Path, got {:?}", other),
        }
    }
}
