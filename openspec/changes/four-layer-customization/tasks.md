# 四层可定制化架构 - 实施任务清单

## 1. 基础设施搭建

- [x] 1.1 在根 `Cargo.toml` 的 `[workspace.dependencies]` 新增 `rhai = "1.19"`
- [x] 1.2 新建 crate `crates/material`，配置 `Cargo.toml`（依赖 `peregrine_config` + `rhai` + `ahash` + `tracing` + `thiserror`）
- [x] 1.3 把 `crates/material` 加入 workspace `members`
- [x] 1.4 在 `crates/material/src/lib.rs` 编写模块顶部中文 doc comment，声明 crate 职责（物料运行时，不依赖 winit / softbuffer）
- [x] 1.5 验证 `cargo build -p peregrine_material` 通过

## 2. 配置层 schema 扩展（crates/config）

- [x] 2.1 在 `crates/config/src/schema.rs` 新增 `Element` 枚举（`Rect`/`Circle`/`CircleStroke`/`DashedCircle`/`Triangle`/`Polygon`/`Line`/`Text`/`Image` 变体，每个 variant 用 `#[serde(tag = "type", rename_all = "snake_case")]`）
- [x] 2.2 为 `Element` 编写完整的中文 doc comment 和字段范围说明
- [x] 2.3 新增 `Transform2D { offset_x, offset_y, scale, rotation_deg }` 结构 + 默认值（identity）
- [x] 2.4 新增 `LayerStyle { color: [f32;4], opacity: f32, blend_mode: BlendMode }` + `BlendMode` 枚举（首期仅 `Normal`）
- [x] 2.5 新增 `MaterialRef` 枚举（`Builtin { id }` / `User { name }`），`#[serde(tag = "kind", rename_all = "snake_case")]`
- [x] 2.6 新增 `Layer { id, name, material, params: serde_json::Value, style, transform, visible, locked }`
- [x] 2.7 修改 `Profile`：保留 `trigger`、`settings_hotkey`、`target_window`；将 `crosshair` 字段替换为 `layers: Vec<Layer>`（**BREAKING**）
- [x] 2.8 新增 `Profile::active_layer(&self, id) -> Option<&Layer>` 和 `active_layer_mut`
- [x] 2.9 为新 schema 编写 `validate()` 扩展：layer id 不重复、material 引用非空、color 在 [0,1]、opacity 在 [0,1]、scale > 0 等
- [x] 2.10 编写新 schema 的单元测试：默认值、序列化往返、字段缺失 serde 兼容、校验失败用例

## 3. 保留旧 Crosshair 作为迁移源类型（crates/config）

- [x] 3.1 把现 `Crosshair` / `CrosshairStyle` / `Anchor` / `RingStyle` / `OrbPosition` / `RandomOrbMode` / `BorderFrameStyle` / `GridAlignment` 等类型移动到 `crates/config/src/legacy.rs`，标 `#[deprecated]`，保留序列化能力
- [x] 3.2 在 `lib.rs` 重新导出：新 schema 类型走 `pub use schema::*`；旧类型走 `pub use legacy::*`
- [x] 3.3 验证旧 `config.json`（含 `crosshair` 字段）能通过 `serde_json::from_str::<LegacyAppConfig>` 反序列化

## 4. 物料运行时实现（crates/material）

- [x] 4.1 在 `crates/material/src/lib.rs` 定义 `Material` struct（持有 `AST` + 元数据）、`MaterialRegistry`（`HashMap<MaterialId, Arc<Material>>`）
- [x] 4.2 定义 `DynamicContext { time_ms, mouse_pos, key_state, rng_seed, version }` + `KeyState` 类型（按键状态表，基于 `HashSet<String>`）
- [x] 4.3 定义 `MaterialError`（thiserror）：脚本加载失败、求值失败、超时、未知图元类型
- [x] 4.4 实现 `Material::load(source: &str, id: &str) -> Result<Material>`：创建 `Engine`、注册 host function、解析 AST、调用 `defaults()` 和 `schema()` 缓存元数据
- [x] 4.5 实现 `Material::evaluate(&self, params: &serde_json::Value, screen: &Rect, ctx: &DynamicContext) -> Result<Vec<Element>>`：执行 `build(params, screen)`，转换 Rhai 返回值为 `Vec<Element>`
- [x] 4.6 注册 host function：`time_ms`、`mouse_pos`、`key_down`、`rand`、`rand_range`、`rand_seed`（通过 closure 捕获 `DynamicContext`）
- [x] 4.7 实现物料求值结果缓存：`MaterialCache` LRU（`HashMap<(MaterialId, params_hash, screen_hash, ctx_version), Arc<Vec<Element>>>`），上限 256 项
- [x] 4.8 实现 `MaterialRegistry::load_builtin()`：通过 `include_str!` 加载 12 份内置物料
- [x] 4.9 实现 `MaterialRegistry::load_user(dir: &Path)`：扫描 `%APPDATA%/Peregrine/materials/*.rhai`，合并到注册表（用户优先）
- [x] 4.10 实现 `is_dynamic` 元数据声明：物料脚本可导出 `fn is_dynamic() -> Bool`，影响缓存策略

## 5. 内置物料脚本（crates/material/builtin/*.rhai）

- [x] 5.1 编写 `cross.rhai`（对应旧 `Cross`），对照 `crates/peregrine/src/shapes.rs:85-114`
- [x] 5.2 编写 `large_cross.rhai`（对照 `shapes.rs:115-129`）
- [x] 5.3 编写 `edge_rect.rhai`（对照 `shapes.rs:130-147`）
- [x] 5.4 编写 `corner_dots.rhai`（参数 `count: 4/6/8`，对照 `shapes.rs:148-202`）
- [x] 5.5 编写 `ring.rhai`（对照 `shapes.rs:203-242`），包含 `ring_style` 参数（solid/dashed/double）
- [x] 5.6 编写 `custom_orb.rhai`（对照 `shapes.rs:243-290`）
- [x] 5.7 编写 `random_orb.rhai`（对照 `shapes.rs:291-333`），**重点**：种子计算 MUST 与旧实现一致（`shapes.rs:291-296` 的 `wrapping_add` 序列）
- [x] 5.8 编写 `border_frame.rhai`（对照 `shapes.rs:334-365`），含 `border_frame_style`（solid/gap）
- [x] 5.9 编写 `edge_arrows.rhai`（对照 `shapes.rs:366-490`）
- [x] 5.10 编写 `grid.rhai`（对照 `shapes.rs:494-575`），含 `grid_alignment`
- [x] 5.11 编写 `image.rhai`（对应旧 `CustomImage`，返回单 `Element::Image`）
- [x] 5.12 为每份物料编写 `defaults()` 和 `schema()` 函数（参数 label / widget / min / max / step / options）
- [x] 5.13 为每份物料编写中文 doc comment（脚本顶部 `//` 注释说明对应旧 style 和参数含义）

## 6. SimpleRng / 几何工具跨 crate 共享

- [x] 6.1 把 `crates/peregrine/src/shapes.rs:727-745` 的 `SimpleRng` 移到 `crates/config/src/rng.rs`（或新建 `crates/util`），保持纯数据无依赖
- [x] 6.2 在 `crates/material` 中暴露 `SimpleRng` 给 Rhai host function（`rand()` 内部调用同一份实现）
- [x] 6.3 单元测试：`SimpleRng` 在 Rust 与 Rhai 两侧产生完全一致的随机序列

## 7. 配置迁移逻辑（crates/config/src/migration.rs）

- [x] 7.1 新建 `migration.rs` 模块，在 `lib.rs` 中 `pub mod migration;`
- [x] 7.2 实现 `migrate_legacy_crosshair(crosshair: &LegacyCrosshair) -> Result<Layer>`：按样式映射表生成单图层
- [x] 7.3 实现 `migrate_app_config(legacy_value: serde_json::Value) -> Result<AppConfig>`：反序列化旧格式、迁移每个 profile、生成新 schema
- [x] 7.4 实现 `is_legacy_config(value: &serde_json::Value) -> bool`：检测是否含 `crosshair` 且无 `layers`
- [x] 7.5 修改 `storage.rs::load_or_create_default`：加载时检测旧格式 → 备份 `.legacy.bak` → 迁移 → 校验 → 保存新格式
- [x] 7.6 异常分支：迁移失败时备份 `.legacy.bak.error`，回退默认配置，记录 warn 日志
- [x] 7.7 单元测试（13 个）：每种旧 style 的迁移用例 + `toilet_paper` alias
- [x] 7.8 单元测试：字段值完整保留（数值、字符串、枚举值逐字段断言）
- [x] 7.9 单元测试：异常输入降级到默认配置
- [x] 7.10 集成测试 `crates/config/tests/migration_regression.rs`：迁移后调用真实物料求值，与旧 `build_shapes` 输出逐元素对比（13 个 style × 典型参数组合）

## 8. 共享库 shapes.rs 重构（crates/peregrine）

- [x] 8.1 修改 `crates/peregrine/src/shapes.rs::build_shapes` 签名：从 `(screen, &Crosshair)` 改为 `(screen, &Profile, &MaterialRegistry, &DynamicContext)`
- [x] 8.2 实现"遍历图层 → 物料求值 → 变换 → 样式 → Element 列表"三段管线（参照 design.md 决策 5 伪代码）
- [x] 8.3 实现 `apply_transform(elements: Vec<Element>, transform: &Transform2D, screen: &Rect) -> Vec<Element>`
- [x] 8.4 实现 `apply_style(elements: Vec<Element>, style: &LayerStyle) -> Vec<Element>`（染色 + opacity 相乘）
- [x] 8.5 单个图层求值失败时记录 warn 并跳过，不阻塞整体渲染
- [x] 8.6 把旧的 12 分支 match 块全部删除（已被物料脚本替代）
- [x] 8.7 保留 `Shape` 类型定义，但内部改为 `pub use peregrine_config::Element`（统一类型，不再两份）
- [x] 8.8 更新 `edge_orb_shapes` / `solid_frame_shapes` / `gap_frame_shapes` 等辅助函数：迁移到对应物料的 Rhai 脚本中（或保留为 Rust 工具函数供物料脚本通过 host function 调用）

## 9. 渲染器改造（crates/peregrine/src/overlay_renderer.rs）

- [x] 9.1 `OverlayRenderer` 新增字段：`material_registry: Arc<MaterialRegistry>`
- [x] 9.2 渲染循环改为：每帧构建 `DynamicContext`（从平台 API 读取鼠标 / 键盘 / 时间）→ 调用新 `build_shapes` → 光栅化 Element 列表
- [x] 9.3 实现 `Element::Text` 光栅化（使用 `-font kit` 或内嵌位图字体；首期可只支持 ASCII）
- [x] 9.4 实现 `Element::Polygon` 光栅化（参照现有 `draw_triangle` 算法推广到 N 边形）
- [x] 9.5 实现 `Element::Line` 光栅化（粗线段，可用旋转矩形近似）
- [x] 9.6 `Element::Image` 光栅化复用现有 `draw_image` + `ensure_image_loaded` 逻辑
- [x] 9.7 多图层混合：按图层顺序绘制，每层按 `LayerStyle.opacity` 相乘 alpha
- [x] 9.8 性能基准：1080p / 3 图层 / 60fps 渲染单帧 < 8ms（debug 可放宽到 < 16ms）

## 10. 动态输入采集（crates/peregrine/src/platform）

- [x] 10.1 在 `platform/windows.rs` 新增 `poll_dynamic_context() -> DynamicContext`：使用 `GetCursorPos` + `GetAsyncKeyState` 读取鼠标键盘状态
- [x] 10.2 非 Windows 平台返回安全默认值（屏幕中心、无按键、零时间）
- [x] 10.3 在 `overlay.rs` 渲染线程每帧调用 `poll_dynamic_context` 并传给 `build_shapes`
- [x] 10.4 键盘状态查询仅记录 Bool，MUST NOT 在日志中打印具体按键代码

## 11. Tauri commands（src-tauri/src/lib.rs）

- [x] 11.1 `AppState` 新增字段 `material_registry: Arc<MaterialRegistry>`，启动时 `MaterialRegistry::load_builtin()` + `load_user()`
- [x] 11.2 新增 command `build_shapes(profile_name: String, screen: RectF) -> Result<Vec<Element>, String>`：调用新 `build_shapes`，返回 JSON
- [x] 11.3 新增 command `list_materials() -> Vec<MaterialInfo>`（id / name / schema / defaults / is_dynamic）
- [x] 11.4 新增 command `add_layer(material_id: String, name: String) -> Result<Layer, String>`
- [x] 11.5 新增 command `remove_layer(layer_id: String) -> Result<(), String>`
- [x] 11.6 新增 command `move_layer(layer_id: String, new_index: usize) -> Result<(), String>`
- [x] 11.7 新增 command `duplicate_layer(layer_id: String) -> Result<Layer, String>`
- [x] 11.8 新增 command `update_layer(layer_id: String, patch: LayerPatch) -> Result<(), String>`
- [x] 11.9 新增 command `list_layers() -> Vec<Layer>`
- [x] 11.10 修改 `save_config` / `get_config` 适配新 schema（无 `crosshair`，有 `layers`）
- [x] 11.11 修改 `set_crosshair_color` → `set_layer_color(layer_id, color)`，针对图层而非 crosshair
- [x] 11.12 删除 `execute_hotkey_action` 中所有对 `active_profile().crosshair.color` 的引用，改为操作图层
- [x] 11.13 所有图层操作 commands 触发 `app.emit("peregrine:layers-changed", &layers)` 事件
- [x] 11.14 在 `invoke_handler!` 注册所有新 commands

## 12. 配置热重载适配

- [x] 12.1 `ConfigWatcher` 检测到 `config.json` 变化时，自动调用迁移逻辑（若用户手动放回旧格式）
- [x] 12.2 用户物料目录 `%APPDATA%/Peregrine/materials/` 新增 watcher，物料文件变化时重新加载 `MaterialRegistry`
- [x] 12.3 物料重载后广播 `peregrine:materials-changed` 事件，前端刷新物料列表

## 13. 前端类型同步（src/types/config.ts）

- [x] 13.1 删除 `Crosshair`、`CrosshairStyle`、`Anchor`、`RingStyle` 等旧类型（或迁移到 `types/legacy.ts` 仅供迁移期使用）
- [x] 13.2 新增 `Element` discriminated union（type 字段区分 variant）
- [x] 13.3 新增 `Layer`、`MaterialRef`、`Transform2D`、`LayerStyle`、`BlendMode` 接口
- [x] 13.4 修改 `Profile`：`crosshair` → `layers: Layer[]`
- [x] 13.5 新增 `MaterialInfo` / `MaterialSchema` / `LayerPatch` 等接口

## 14. 前端 Preview 改造（src/components/Preview.tsx）

- [x] 14.1 删除 `src/lib/shapes.ts` 全部内容（约 280 行）
- [x] 14.2 Preview 组件 state 新增 `elements: Element[]`
- [x] 14.3 实现 `useEffect` 监听 profile / layers 变化，调用 `build_shapes` IPC，节流 16ms
- [x] 14.4 Canvas 渲染逻辑改为遍历 `elements` 数组，按 `type` 字段 switch 绘制
- [x] 14.5 新增 Canvas 渲染分支：`text`（用 Canvas 2D `fillText`）、`polygon`（`Path2D`）、`line`（`lineWidth` + `moveTo/lineTo`）、`image`（`drawImage`）
- [x] 14.6 处理 IPC 错误：物料求值失败时显示错误提示
- [x] 14.7 动态物料标签显示"动态物料 - 预览为快照"提示

## 15. 前端图层管理 UI

- [x] 15.1 新建 `src/components/LayerPanel.tsx`：左侧或右侧侧边栏显示图层列表
- [x] 15.2 图层项组件：显示 name / material / visible toggle / locked icon / delete button
- [x] 15.3 拖拽排序（使用 `@dnd-kit/core` 或原生 HTML5 drag）
- [x] 15.4 添加图层按钮：弹出物料选择对话框（`list_materials` IPC 返回的列表）
- [x] 15.5 选中图层后高亮显示，下方显示该图层的参数面板
- [x] 15.6 顶部工具栏：图层操作（复制 / 删除 / 上移 / 下移）

## 16. 前端动态参数控件（src/components/StyleFields.tsx 重写）

- [x] 16.1 删除旧的按 `CrosshairStyle` switch 的固定控件渲染
- [x] 16.2 新建 `src/components/MaterialParamControls.tsx`：接收 `schema: MaterialSchema[]`，动态渲染控件
- [x] 16.3 实现 `widget: "number"` 控件（数字输入框）
- [x] 16.4 实现 `widget: "slider"` 控件（复用现有 `ui/slider.tsx`）
- [x] 16.5 实现 `widget: "color"` 控件（颜色选择器，支持 RGBA）
- [x] 16.6 实现 `widget: "select"` 控件（复用 `ui/select.tsx`）
- [x] 16.7 实现 `widget: "toggle"` 控件（复用 `ui/switch.tsx`）
- [x] 16.8 实现 `widget: "image_path"` 控件（文件选择按钮，调用 `pick_image_path`）
- [x] 16.9 实现 `widget: "text"` 控件（文本输入框）
- [x] 16.10 未知 widget 类型回退为只读文本 + 警告日志

## 17. 前端图层样式 / 变换编辑

- [x] 17.1 新建 `src/components/LayerStyleEditor.tsx`：颜色选择器（RGBA）+ 不透明度滑块 + 混合模式下拉
- [x] 17.2 新建 `src/components/LayerTransformEditor.tsx`：offset_x / offset_y / scale / rotation_deg 滑块
- [x] 17.3 控件值变化时调用 `update_layer` IPC，节流 16ms

## 18. ConfigApp 整体改造（src/ConfigApp.tsx）

- [x] 18.1 主布局调整：左侧图层列表 + 中间预览 + 右侧参数面板（响应式）
- [x] 18.2 删除旧的单一 crosshair 表单逻辑
- [x] 18.3 颜色快捷切换（`set_color_N` 快捷键）改为作用于"当前选中图层"
- [x] 18.4 Profile 切换 / 管理 UI 保持不变（仅内部从 crosshair 改为 layers）

## 19. 文档与示例

- [x] 19.1 更新 `AGENTS.md`：新增 `crates/material` 架构边界说明、Rhai 物料脚本约定、`DynamicContext` 说明
- [x] 19.2 新增 `docs/guide/material-scripting.md`：用户向物料脚本编写指南（build / defaults / schema 三函数、动态输入 API、示例）
- [x] 19.3 新增 `docs/guide/layers.md`：图层管理使用说明
- [x] 19.4 在 `docs/.vitepress/config.mts` 注册新文档侧边栏条目
- [x] 19.5 新增 `crates/material/examples/` 目录，提供 3-5 个示例物料脚本（动态时钟、跟随鼠标的点、键盘响应提示）
- [x] 19.6 更新 README.md（如提及样式数量的描述）

## 20. CI / 发布配置

- [x] 20.1 `.github/workflows/ci.yml` 的 lint job 新增 `cargo clippy -p peregrine_material -- -D warnings`
- [x] 20.2 CI build job 的 Windows 测试步骤新增 `cargo test -p peregrine_material`
- [x] 20.3 `Cargo.lock` 更新（含 rhai 及其依赖）
- [x] 20.4 根 `Cargo.toml` 的 `workspace.package.version` 改为 `0.2.0-alpha.0`
- [x] 20.5 `src-tauri/tauri.conf.json` 的 `version` 同步（或继承 workspace）
- [x] 20.6 `CHANGELOG_ALPHA.md` 新增 v0.2.0-alpha.0 条目（Added: 四层架构 / Rhai 物料 / 多图层；Changed: schema 重构；Migration: 自动迁移）

## 21. 端到端验证

- [ ] 21.1 准备 5 份真实用户旧配置（覆盖 12 种样式），手动测试迁移流程，确认零视觉退化
- [ ] 21.2 性能基准：1080p / 5 图层 / 60fps 渲染 1 小时无明显掉帧（frame time < 16ms）
- [ ] 21.3 内存基准：对比 v0.1.15，物料缓存 + Rhai engine 内存增量 < 10MB
- [ ] 21.4 二进制体积基准：release 构建增量 < 500KB
- [ ] 21.5 物料求值延迟：单图层单次求值 < 100µs（静态物料缓存命中应 < 1µs）
- [ ] 21.6 用户物料脚本错误场景测试：语法错误 / 运行时异常 / 死循环 / 调用未注册函数，均不崩溃
- [ ] 21.7 动态物料实际效果验证：时钟物料每秒更新、鼠标跟随物料延迟 < 50ms、键盘响应物料即时

## 22. 回归测试与稳定化

- [x] 22.1 `cargo test --workspace` 全部通过
- [x] 22.2 `cargo clippy --workspace -- -D warnings` 通过
- [x] 22.3 `cargo fmt --all -- --check` 通过
- [x] 22.4 `npm run build` 前端构建通过，无 TypeScript 错误
- [x] 22.5 `npx tauri build` release 构建成功，产物可正常启动 + 迁移 + 渲染
- [x] 22.6 手工冒烟测试：全新安装 / 旧配置升级 / 多图层叠加 / 用户物料加载 / 动态物料预览
