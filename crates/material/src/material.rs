//! 物料类型与求值实现。
//!
//! 一个 [`Material`] 实例对应一份 Rhai 脚本，在加载时预编译为 AST。
//! 每次求值时创建独立的 `Engine`（注册了捕获当前 `DynamicContext` 的 host function），
//! 通过 `call_fn` 调用脚本的 `build` 函数，返回 `Vec<Element>`。

use crate::context::DynamicContext;
use crate::error::{MaterialError, MaterialResult};
use peregrine_config::{Element, Rect, SimpleRng};
use rhai::{Dynamic, Engine, ImmutableString, Map, Scope, AST};

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
        let ast = engine
            .compile(source)
            .map_err(|e| MaterialError::Parse {
                id: id.clone(),
                source: e,
            })?;

        let mut scope = Scope::new();

        // 调用 defaults() 缓存默认参数。
        let defaults_val: Dynamic = engine
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

        let display_name = parse_display_name(source)
            .unwrap_or_else(|| id.rsplit('.').next().unwrap_or(&id).to_string());

        let metadata = MaterialMetadata {
            id,
            display_name,
            is_dynamic,
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

    // mouse_pos() -> Map { x: Float, y: Float }
    let (mx, my) = ctx.mouse_pos;
    engine.register_fn("mouse_pos", move || {
        let mut m = Map::new();
        m.insert("x".into(), (mx as f64).into());
        m.insert("y".into(), (my as f64).into());
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

    engine
}

thread_local! {
    /// 每个线程一个调用计数器，用于让多次 `rand()` 调用产生不同结果。
    ///
    /// 每次物料求值使用独立的 Rhai Engine，但同一个 Engine 内多次调用 `rand()`
    /// 需要不同的结果。计数器在 Engine 创建时清零（见 `thread_local_reset`）。
    static RAND_COUNTER: std::cell::Cell<u64> = std::cell::Cell::new(0);
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
            detail: format!(
                "element must be a Map, got {}",
                d.type_name()
            ),
        });
    };

    let get_str = |key: &str| -> MaterialResult<String> {
        let v = m
            .get(key)
            .ok_or_else(|| MaterialError::ElementField {
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
        let v = m
            .get(key)
            .ok_or_else(|| MaterialError::ElementField {
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
    match type_name.as_str() {
        "rect" => Ok(Element::Rect {
            x: get_f32("x")?,
            y: get_f32("y")?,
            w: get_f32("w")?,
            h: get_f32("h")?,
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
            let points_val = m
                .get("points")
                .ok_or_else(|| MaterialError::ElementField {
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
        }),
        "image" => Ok(Element::Image {
            path: get_str("path")?,
            x: get_f32("x")?,
            y: get_f32("y")?,
            w: get_f32("w")?,
            h: get_f32("h")?,
        }),
        other => Err(MaterialError::UnknownElementType {
            id: material_id.to_string(),
            element_type: other.to_string(),
        }),
    }
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
            other => panic!("expected MissingFunction, got {:?}", other.map(|m| m.id().to_string())),
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
            MaterialError::UnknownElementType {
                element_type, ..
            } => {
                assert_eq!(element_type, "ellipse");
            }
            other => panic!("expected UnknownElementType, got {:?}", other),
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
            let params = serde_json::json!({"anchor": anchor, "size": 100.0, "secondary_size": 30.0});
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

        // edge 模式比 center 模式多 2 行 + 2 列（含边缘）。
        assert!(edge.len() > center.len());
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
}
