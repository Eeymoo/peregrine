# disable-material-runtime 设计

## Context

物料体系（layers + Rhai 运行时）上线后缺陷反复，最新一次是时钟物料"时间又不显示"。历史上已投入多个变更修复（`material-runtime`、`material-dynamic-input`、`overlay-dynamic-text-fixes` 等），但动态刷新调度 + 文本图元 + 配置迁移三条链路耦合复杂，在单人维护节奏下性价比过低。

当前代码现状（已核实）：

- **渲染双路径已存在**：`overlay_renderer.rs` 根据 `profile.layers.is_empty()` 选择新（`build_layers_shapes`）旧（`build_shapes`）路径，旧路径完整可用。
- **前端双模式已存在**：`ConfigApp.tsx` 保留旧版准星 UI（`StyleFields` + `crosshair`），`layersMode` 状态切换进 `LayersEditor`；`crosshair` 缺失时还能从 `layers[0]` 反向生成兜底。
- **动态性调度在 overlay.rs**：`has_dynamic_material` 判定 + 帧缓存 + `ControlFlow` 唤醒逻辑，是"时钟不跳"类 bug 的高发区。
- **配置**：`Profile.layers` 与旧 `crosshair` 字段共存，迁移逻辑（`profile-migration`）已把旧配置升级为 layers。

利益相关：最终用户（需要稳定可用的准星工具）、维护者（降低维护面）。

## Goals / Non-Goals

**Goals:**

- overlay 渲染 100% 走旧 `Crosshair` 路径，任何配置下都有可见准星（无 layers 且无 crosshair 时用 `Crosshair::default_crosshair()`）。
- 设置面板不再出现图层编辑器入口，用户只能看到旧版准星设置 UI。
- 动态上下文轮询（鼠标/键盘/时间）与动态重绘调度从运行链路移除，overlay 回到静态渲染（仅配置变更/窗口事件触发重绘）。
- 物料相关代码（`crates/material`、`shapes.rs` 的 layers 分支、`LayersEditor` 组件）保留在仓库中，通过单一开关点关闭，可低成本恢复。
- 用户已有 `layers` 配置数据不删除、不报错。

**Non-Goals:**

- 不删除 `crates/material` crate、不删除图层/迁移代码与相关测试。
- 不做 layers → crosshair 的"反向迁移"写回用户配置文件（`layers` 数据原样保留在 config.json）。
- 不修改 `element-primitives` 图元定义（软关闭后图元仅被旧路径使用的子集消费，定义保留）。
- 不处理其他进行中变更（`overlay-dynamic-text-fixes`、`material-e2e-validation`、`material-docs-examples` 的终止由项目管理层面处理，不在本变更代码范围内）。

## Decisions

### D1：开关形式——集中常量 `MATERIAL_RUNTIME_ENABLED = false`

在 `crates/peregrine/src/lib.rs`（或新 `feature.rs` 模块）定义 `pub const MATERIAL_RUNTIME_ENABLED: bool = false;`，所有软关闭分支以它为条件；前端在 `src/lib/feature.ts` 定义同名常量。重新启用时改回 `true` 即可。

- **备选 A：Cargo feature flag**——更"正规"，但需要改 workspace 依赖声明、CI 矩阵、条件编译散布多处，恢复成本反而高，且前端无法共享。否决。
- **备选 B：运行时配置项**——把地雷暴露给用户，违背"收尾、降维护面"的初衷。否决。
- **结论**：编译期常量，死代码由编译器消除，零运行时开销，恢复只需翻一个常量。

### D2：渲染路径——在 `overlay_renderer.rs` 单点强制旧路径

`use_new_format` 的计算改为：

```rust
let use_new_format = crate::MATERIAL_RUNTIME_ENABLED
    && profile.map(|p| !p.layers.is_empty()).unwrap_or(false);
```

常量折叠后 `build_layers_shapes`、`MaterialRegistry`、图片预加载的 layers 分支全部成为死代码被优化掉；`material_registry` 字段可保留构造（开销极小）或同样用常量门控，实现时择简。

- **备选：删除 layers 分支代码**——属于"彻底删除"方案，已被用户否决（软关闭）。

### D3：动态调度移除——`overlay.rs` 删除动态判定，`ControlFlow` 回到纯事件驱动

删除 `has_dynamic_material` 判定函数、动态性缓存、以及 `about_to_wait` 中动态物料的定期唤醒分支；overlay 重绘仅由以下事件触发：配置热重载通知、窗口跟随位置变化、`RedrawRequested` 链。这正是旧架构的行为，风险最低。

`platform::poll_dynamic_context` 不再被调用（`mod.rs` 中函数保留，调用点移除）。

### D4：前端——隐藏入口而非删除组件

- `ConfigApp.tsx` 中 `setLayersMode(true)` 的三处入口（异常兜底按钮、两个切换按钮）以 `MATERIAL_RUNTIME_ENABLED` 常量门控隐藏；`layersMode` 初始值强制 `false`。
- `LayersEditor.tsx`、`LayerPanel.tsx`、`LayerEditors.tsx` 组件文件保留不删，仅不再被引用（或仅在被门控的分支引用）。
- i18n 文案键保留。

### D5：配置兼容——只读降级，不写回

- 含 `layers` 的配置加载后照常反序列化与校验（`schema.rs` 不动），渲染与前端编辑只消费 `crosshair`；`crosshair` 为 `None` 时回退 `Crosshair::default_crosshair()`（渲染层兜底已存在，前端沿用现有"从 layers[0] 反向生成 crosshair"逻辑作为显示兜底）。
- **不**把降级结果写回 config.json：`layers` 数据保留，将来翻开关即可恢复。

### D6：`crates/material` 依赖处理——保留编译

`crates/peregrine` 对 `peregrine_material` 的依赖保留（`DynamicContext`、`MaterialRegistry` 类型仍在签名中），crate 照常参与 workspace 编译与测试，只是运行链路不触达。这样恢复时无 Cargo 变更，也避免大面积改类型签名。

## Risks / Trade-offs

- [死代码随时间腐化，重新启用时可能已编译不过] → CI 继续全 workspace `cargo build`/`cargo test`，物料 crate 测试照常运行，保证代码始终可编译。
- [用户已有的 layers 配置"看起来丢了"（显示为默认/旧准星）] → 前端在准星设置页显示一次性提示"多图层功能已暂时停用，配置已保留"；发布说明中明确说明。
- [常量门控散落多处导致恢复时遗漏] → 开关集中在两个文件（Rust/TS 各一），所有门控点 grep `MATERIAL_RUNTIME_ENABLED` 可枚举；tasks 中要求每个门控点加统一注释标记。
- [静态渲染下 overlay 完全不自动重绘，若配置通知链路有遗漏会"卡画面"] → 旧架构本就如此运行且经过长期验证；保留窗口跟随的 16ms 轮询触发的重绘。

## Migration Plan

1. 实现 D1–D5，本地 `cargo test` + `cargo clippy` + 前端 `npm run build` 全绿。
2. 手动验证：含 layers 的旧配置文件启动 → 显示默认/旧准星；纯旧配置 → 行为不变。
3. 发布一个 stable 版本（奇数版本号），release notes 说明物料功能暂停。
4. 回滚策略：将两个 `MATERIAL_RUNTIME_ENABLED` 常量改回 `true` 即可恢复全部功能，无数据迁移不可逆步骤。

## Open Questions

- 三个物料相关进行中变更（`overlay-dynamic-text-fixes` 等）是否在本变更落地后归档/删除？——建议归档到 `openspec/changes/archive` 并标注 superseded，由用户确认后另行处理。
