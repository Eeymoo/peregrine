# 任务：Path 路径图元

## 1. 数据模型（crates/config）

- [x] 1.1 `schema.rs` 新增 `PathSegment` 枚举（`M`/`L`/`Q`/`C`/`Z`，serde `tag = "cmd"` + snake_case），所有变体带中文文档注释
- [x] 1.2 `schema.rs` 的 `Element` 枚举新增 `Path { segments, fill, thickness, stroke_color, fill_color }` 变体，可选字段用 `#[serde(default)]`
- [x] 1.3 `schema.rs` 单测：Path 序列化往返（含 `"cmd": "m"` tag 断言）、缺省字段反序列化（`fill=false`、颜色 `None`）

## 2. Rhai 转换层（crates/material）

- [x] 2.1 `material.rs` 转换 match 新增 `"path"` arm：解析 `segments`（Map 风格段对象，坐标接受 float/int）、`thickness`、可选 `fill` / `stroke_color` / `fill_color`
- [x] 2.2 转换层校验：首段必须 `M`、segments 非空、`fill=false && thickness=0` 拒绝、`thickness >= 0`、颜色分量 `0..=1` 且长度 4；全部走 `ElementField` 错误
- [x] 2.3 `material.rs` 单测：完整 path 解析、未知 cmd 拒绝、Q 缺控制点拒绝、不可见组合拒绝、颜色越界拒绝

## 3. 鼠标速度/加速度输入（crates/material + crates/peregrine）

- [x] 3.1 `context.rs` 的 `DynamicContext` 新增 `mouse_velocity` / `mouse_acceleration` 字段（`(f32, f32)`，默认 0）；`preview_snapshot` / `static_context` 语义注释更新
- [x] 3.2 `material.rs` 注册 `mouse_velocity()` / `mouse_acceleration()` host function（返回 `Map {x, y}`，与 `mouse_pos` 同风格）
- [x] 3.3 `platform/windows.rs` 的 `poll_dynamic_context` 差分采样：`static` 上一采样（位置/速度/EMA/Instant），实测 `dt` 差分，EMA 平滑（α 常量），死区归零（速度 ~5 px/s、加速度 ~50 px/s² 常量集中定义）；非 Windows 占位返回 0
- [x] 3.4 `context.rs` / `material.rs` 单测：默认上下文速度/加速度为 0；host function 返回 Map 结构正确

## 4. 内置物料（crates/material/builtin）

- [x] 4.1 编写 `teardrop.rhai`：圆周采样 + `mouse_acceleration()` 方向/幅度驱动的径向位移 + 尖角合成，输出闭合 Path（`fill=true`、双色覆盖、`is_dynamic` 声明）；零加速度退化为正圆
- [x] 4.2 编写 `path_showcase.rhai`：星形（M/L/Z 直线段）+ 贝塞尔花（C 曲线段）两组静态 Path，参数暴露花瓣数/内外径比/厚度，缺省继承图层色
- [x] 4.3 `lib.rs` 的 `BUILTIN_MATERIALS` 追加 teardrop + path_showcase 条目
- [x] 4.4 单测：`load_builtin()` 后两物料可求值；teardrop 零加速度 = 正圆 / 非零加速度 = 偏移且强度成正比；path_showcase 含直线段与曲线段 Path、双上下文输出一致

## 5. 几何变换与包围盒（crates/peregrine）

- [x] 5.1 `shapes.rs` 新增自适应展平工具（de Casteljau 细分至中点偏离弦 < 0.5 逻辑像素，返回折点）
- [x] 5.2 `apply_transform` 新增 Path arm：仿射直接作用于 M/L/Q/C 全部坐标（锚点+控制点），`Z` 透传，颜色字段透传；单测覆盖旋转闭合路径与缩放
- [x] 5.3 `elements_center` 新增 Path 分支：展平折点取 min/max，描边外扩 `thickness/2`；单测断言同曲线两次计算结果一致、凸曲线包围盒 ≤ 控制点壳

## 6. 渲染双端（crates/peregrine + src）

- [x] 6.1 `overlay_renderer.rs` 的 `hash_element` 新增 Path arm（判别值 + fill + thickness + 逐段 cmd/坐标位模式 + 覆盖色分量位模式）
- [x] 6.2 `svg_renderer.rs` 的 `build_elements_svg` 新增 Path arm：拼接 `d` 属性（坐标 × scale）、`fill/stroke` 按覆盖色语义取值（纯描边 `fill="none"`、纯填充 `stroke="none"`）、`stroke-linecap/linejoin="round"`
- [x] 6.3 `src/types/config.ts` 的 `Element` union 新增 path 分支 + `PathSegment` 类型定义
- [x] 6.4 `Preview.tsx` 的 `drawElement` 新增 path case：`Path2D` 构造同构路径，先 fill 后 stroke，覆盖色 × 图层 opacity 分色
- [x] 6.5 `svg_renderer.rs` 单测：纯描边 / 双色覆盖 / 纯填充三种 Path 生成的 SVG 字符串断言（`d`、`fill`、`stroke`、`stroke-width` 属性）

## 7. 端到端验证

- [x] 7.1 `cargo test -p peregrine_config -p peregrine_material -p peregrine` 全绿；`cargo fmt --check` + `cargo clippy -- -D warnings` 通过
- [x] 7.2 `npm run build`（tsc 类型检查 + Vite 构建）通过
- [ ] 7.3 手动验证：teardrop 图层在 overlay 中随鼠标急转变形、静止回正且回到跳帧；path_showcase 图层预览与 overlay 双端一致（WYSIWYG）、静止帧不重光栅化
