# 设计：Overlay 动态物料刷新修复与文本加粗

## Context

Windows 实机走查发现两个 overlay 渲染问题（详见 proposal）：

1. **动态物料不刷新**：`src-tauri/src/overlay.rs::about_to_wait` 中，新格式（layers 非空）的动态性判定分支写死返回 `None`（代码内注释"首期简化，避免 registry 借用"），导致 `is_animated` 恒为 `false`，事件循环进入 `ControlFlow::Wait` 永久挂起，只有窗口事件（拖拽、resize）触发 `needs_redraw` 才重绘一帧。
2. **文本字重过细**：`Element::Text`（schema.rs:1053）只有 `x / y / content / font_size`，SVG 后端（svg_renderer.rs:255）不输出 `font-weight`。

关键现状：

- `OverlayApp` 已持有 `material_registry: Arc<MaterialRegistry>`（overlay.rs:49,124），不存在借用障碍，原注释的顾虑已被后来的字段引入消除。
- 物料元数据 `Material::metadata().is_dynamic` 在加载时已解析缓存（material.rs:97），查询成本极低。
- 配置变更通过 `OverlayCommand::UpdateConfig` 推送（overlay.rs:155）；物料热重载通过 `peregrine:materials-changed` 事件（src-tauri/src/lib.rs:269）广播，但当前未向 overlay 线程发命令。
- `time.rhai` 已声明 `is_dynamic() == true`，是本修复的首个受益者；`defaults()` / `schema()` / `build()` 三函数结构完整，新增参数只需机械扩展。

## Goals / Non-Goals

**Goals:**

- 含任一 `is_dynamic` 物料图层的 profile，overlay 以 60FPS 持续重绘；时钟每秒更新、鼠标跟随 < 50ms、键盘响应即时（对齐 `material-e2e-validation` §6 验收）。
- 动态性判定开销可忽略（不每帧查询 registry）。
- `Element::Text` 支持字重，overlay 与前端预览 WYSIWYG 一致；`time.rhai` 提供「加粗」开关。
- 序列化向后兼容：旧配置无 `font_weight` 字段可正常加载。

**Non-Goals:**

- 不引入字体族（font-family）选择、斜体、描边等更多排版能力。
- 不改动旧格式（crosshair）RandomOrb 的既有重绘路径。
- 不做动态物料的"按需降频"（如时钟物料 1Hz 即可）——首期统一 60FPS，性能问题由 `material-e2e-validation` §3 验收把关。

## Decisions

### 决策 1：动态性判定放在 `about_to_wait`，结果随配置快照缓存失效

`about_to_wait` 新格式分支改为：遍历 active profile 的 `layers`，跳过 `visible == false` 的图层，经 `material_registry` 查得 `Material::metadata().is_dynamic`，任一为 `true` 即按动画路径持续重绘。

缓存策略：`OverlayApp` 新增 `dynamic_dirty: bool`（初始 `true`）与缓存的判定结果。判定结果在以下时机重新计算：

- `dynamic_dirty == true`（启动 / UpdateConfig / 物料热重载后置位）
- 每次判定后清 `dynamic_dirty`，之后直接复用缓存值

**备选方案**：每帧查询 registry —— 否决，`about_to_wait` 每帧执行，虽单次查询便宜（HashMap 查找 + 布尔读取），但需要对 registry 加读锁或遍历，无必要。
**备选方案**：在 `UpdateConfig` 时预计算并存入配置快照 —— 否决，配置快照类型定义在 `peregrine_config`（纯数据 crate），不应感知物料运行时元数据，破坏分层边界。

### 决策 2：物料热重载通过新增 `OverlayCommand::RefreshMaterials` 通知 overlay 线程

物料热重载（`peregrine:materials-changed`）可能改变某物料的 `is_dynamic`（静态改动态或反之）。src-tauri 的 watcher 回调在重建 registry 后，向 overlay 线程发送新增的 `OverlayCommand::RefreshMaterials`，overlay 侧更新 `material_registry` 句柄并置 `dynamic_dirty = true` + `needs_redraw = true`。

**备选方案**：复用 `UpdateConfig` —— 否决，物料重载不伴随配置变化，语义不符且会触发不必要的配置比较逻辑。

### 决策 3：`font_weight` 用 `Option<u16>`（100–900），`None` 语义为 400

`Element::Text` 新增：

```rust
/// 字重（100–900，`None` 等价于 400 常规）。
#[serde(default)]
pub font_weight: Option<u16>,
```

- `Option<u16>` 而非 `u16` + 默认值函数：让"未设置"与"显式 400"在 JSON 层面可区分，旧配置文件反序列化零成本（`#[serde(default)]` 得 `None`）。
- 校验：非 `None` 时必须在 100–900 且为 100 的整数倍（CSS 规范）。（**实现调整**：`Element` 是物料求值输出、从不持久化到配置文件，`AppConfig::validate()` 无法触及；校验实际落在 `material.rs` 的 Rhai→Element 转换层，非法值返回 `MaterialError::ElementField`，该图层求值失败被跳过，其余图层不受影响。）
- SVG 输出：`font_weight.map(|w| format!("font-weight=\"{w}\"")).unwrap_or_default()`。
- 前端预览：`ctx.font = `${bold ? "bold " : ""}${size}px sans-serif``。

**备选方案**：`bold: bool` —— 否决，字重是数值轴，bool 未来扩展（如 600 semibold）需再加字段。

### 决策 4：`time.rhai` 的 bold 参数在 build 中映射为 font_weight 700

- `defaults()` 加 `bold: false`；`schema()` 加 `#{key: "bold", label: "加粗", widget: "toggle"}`。
- `build()`：`font_weight: if params.bold { 700 } else { () }`（Rhai 侧 `()` 序列化为 `null` → Rust 侧 `None`）。需在 `material.rs` 的 Rhai→Element 转换中确认 `font_weight` 缺失 / null 均映射为 `None`（serde default 已保证缺失兼容，null 需确认 `Option` 反序列化路径，预期天然支持）。

## Risks / Trade-offs

- [60FPS 持续重绘增加 CPU 占用] → 动态物料本就需要持续刷新；`material-e2e-validation` §3.3/3.4 会以单帧 < 8ms、1 小时稳定性验收兜底。静态物料 profile 仍走 `ControlFlow::Wait`，零额外开销。
- [`is_dynamic` 判定缓存失效时机遗漏导致行为陈旧] → 失效入口只有两个（UpdateConfig / RefreshMaterials），均在命令处理单点置位；新增 overlay 命令时 review 检查是否影响 layers。
- [系统无粗体字形导致 font-weight 无效] → `load_system_fonts` 加载系统字体库，Windows 常规字体均含 Bold 字族；降级行为为渲染常规字重，不崩溃。
- [Rhai `()` → JSON null → `Option<u16>` 反序列化路径与预期不符] → 在 `material.rs` 转换层补单元测试覆盖 `font_weight` 缺失 / null / 700 三种输入。

## Migration Plan

纯代码修复 + 向后兼容的字段新增，无配置迁移。发布随四层分支的下一个 alpha 版本。实机复测通过 `docs/manual-test-checklist.md` B4 区 + 字重目测后，可勾掉 `multi-profile-config` 之外受阻于本 bug 的相关验收项。

## Open Questions

- 键盘响应 / 鼠标跟随示例物料尚不存在（`material-docs-examples` §4 交付），本 change 仅用 `time.rhai` 验证时间动态性；输入动态性的实机验收依赖示例物料就位后复测。
