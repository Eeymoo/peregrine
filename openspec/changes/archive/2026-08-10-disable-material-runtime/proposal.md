# disable-material-runtime 提案

## Why

动态物料功能（layers + Rhai 物料运行时）上线后问题反复：时钟物料多次出现"时间又不显示"类回归（见 `overlay-dynamic-text-fixes` 变更的修复记录），动态刷新调度、文本图元渲染等链路持续出现缺陷，维护成本已超过其带来的收益。在 v0.1.x 稳定线收束的阶段，决定对该功能做**软关闭**：运行时与界面均不再触达物料体系，回退到久经验证的旧版 `Crosshair` 渲染路径，代码保留以便将来修复后重新启用。

## What Changes

- 渲染器软关闭：overlay 渲染路径强制走旧版 `Crosshair` 分支（`build_shapes`），忽略 `Profile.layers`，不再调用 `build_layers_shapes` 与物料求值。
- 配置降级：加载到含 `layers` 的配置时不再按新格式渲染；若旧 `crosshair` 字段缺失则回退到 `Crosshair::default_crosshair()`，保证任何配置下都有可见准星。
- 前端隐藏：设置面板隐藏图层编辑器（LayersEditor）入口，恢复/保留旧版准星样式设置控件作为唯一编辑入口。
- 动态输入停采：不再为物料求值轮询鼠标/键盘/时间等动态上下文。
- 保留代码：`crates/material`、图层数据结构与迁移逻辑代码原样保留（不参与编译路径的调用点除外），通过一个集中的开关常量控制，便于日后重新启用。
- 进行中变更 `overlay-dynamic-text-fixes` / `material-e2e-validation` / `material-docs-examples` 随本收尾决策暂停（保留不动），等待将来物料功能重新启用后继续推进。

## Capabilities

### New Capabilities

（无新能力；本变更纯为关闭既有能力。）

### Modified Capabilities

- `material-runtime`: 物料运行时在主程序中不再被实例化与求值；内置物料脚本、注册表代码保留但不进入渲染链路。
- `layer-composition`: `Profile.layers` 不再参与渲染与设置面板编辑；渲染回退到旧 `crosshair` 路径；图层管理 UI 隐藏。
- `material-dynamic-input`: 动态输入 host function（时间/鼠标/键盘/随机数）不再被运行时消费，主程序停止动态上下文轮询。

## Impact

- **代码**：
  - `crates/peregrine/src/overlay_renderer.rs`（强制旧路径）
  - `crates/peregrine/src/shapes.rs`（`build_layers_shapes` 调用点收口）
  - `crates/peregrine/src/platform/mod.rs`（`poll_dynamic_context` 停止调用）
  - `src-tauri/src/overlay.rs`（移除动态重绘调度分支）
  - `src/App.tsx`、`src/components/`（隐藏 LayersEditor，恢复旧准星控件）
- **依赖**：`crates/material`（`rhai` 等）暂时不再被 `crates/peregrine` 引用，可选择移出 workspace 成员编译或保留编译但不链接；决策细节见 design.md。
- **用户配置**：已有 `layers` 配置的用户降级为默认/旧准星外观，配置文件中 `layers` 数据保留不删除，重新启用时可恢复。
- **规格**：`material-runtime` / `layer-composition` / `material-dynamic-input` 三个能力的需求级别调整为"暂停生效"。
