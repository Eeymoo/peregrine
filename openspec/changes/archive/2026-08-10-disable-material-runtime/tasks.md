# disable-material-runtime 任务清单

## 1. 开关常量

- [x] 1.1 在 `crates/peregrine/src/lib.rs` 新增 `pub const MATERIAL_RUNTIME_ENABLED: bool = false;`，附中文注释说明用途与恢复方式
- [x] 1.2 在 `src/lib/feature.ts` 新建 `export const MATERIAL_RUNTIME_ENABLED = false;`，附中文注释

## 2. 渲染器软关闭（Rust）

- [x] 2.1 修改 `crates/peregrine/src/overlay_renderer.rs`：`use_new_format` 计算改为 `crate::MATERIAL_RUNTIME_ENABLED && !layers.is_empty()`，门控点加统一注释标记 `// MATERIAL_RUNTIME_ENABLED 门控`
- [x] 2.2 检查 `overlay_renderer.rs` 中 `MaterialRegistry` 构造、layers 图片预加载等仅服务新路径的逻辑，用同一常量门控或确认其为无害死代码 → 确认：`use_new_format` 为编译期 `false` 后相关分支常量折叠为死代码，`cargo check`（Linux + windows-gnu target）无警告，注册表构造开销极小故保留
- [x] 2.3 确认 `crates/peregrine/src/shapes.rs` 的 `build_layers_shapes` 不再存在活跃调用点（保留函数与测试）→ 仅余编译期死分支引用，活跃路径无调用

## 3. 动态调度移除（src-tauri）

- [x] 3.1 删除 `src-tauri/src/overlay.rs` 中 `has_dynamic_material` 判定函数与动态性缓存字段（`dynamic_dirty` / `is_animated_cache` 已删除，`compute_is_animated` 简化为仅旧格式 RandomOrb 判定）
- [x] 3.2 删除 `about_to_wait` 中动态物料定期唤醒分支，`ControlFlow` 回到纯事件驱动（仅配置通知/窗口跟随/系统事件触发重绘；RandomOrb 旧动画保留）
- [x] 3.3 移除 `platform::poll_dynamic_context` 的全部调用点（`mod.rs` 中函数定义保留）→ 实现方式：`use_new_format` 编译期为 `false`，渲染器内两处调用随分支常量折叠不可达，恢复时随开关一并还原
- [x] 3.4 全局 grep 确认无残留：`DynamicContext`、`is_dynamic`、`material_registry` 在 `src-tauri/` 与 `crates/peregrine/src/overlay_renderer.rs` 活跃路径中无引用（`src-tauri/src/lib.rs` 保留的物料相关 Tauri commands 供 LayersEditor 使用，UI 已隐藏不会被调用，代码按设计保留）
- [x] 3.5 补漏 `build_shapes_ipc`（`src-tauri/src/lib.rs` 预览 IPC）：软禁用时该命令仍会调用 `build_layers_shapes` + `DynamicContext::preview_snapshot`，导致拖滑块时在 Tauri 线程跑 Rhai 物料求值。改为用 `peregrine::MATERIAL_RUNTIME_ENABLED` 门控——软禁用走新增的 `build_shapes_from_crosshair`（复用 `build_shapes` 几何 + crosshair 颜色/不透明度），与 overlay 旧路径 WYSIWYG；启用分支保留原 layers 求值。`crates/peregrine/src/shapes.rs` 新增 `build_shapes_from_crosshair` 入口及回归测试 `build_shapes_from_crosshair_carries_color_and_opacity`

## 4. 前端隐藏图层入口

- [x] 4.1 修改 `src/ConfigApp.tsx`：`layersMode` 初始值强制 `false`（`useConfigAppState.ts` 中 `MATERIAL_RUNTIME_ENABLED && !compatible`），三处 `setLayersMode(true)` 入口用 `MATERIAL_RUNTIME_ENABLED` 门控隐藏（含异常兜底的"切换到图层编辑器"按钮）
- [x] 4.2 确认 `LayersEditor` / `LayerPanel` / `LayerEditors` 组件文件保留但不被活跃路径引用（保留源码不删除）
- [x] 4.3 在准星设置页增加一次性中文提示："多图层功能已暂时停用，原有图层配置已保留"（新增 i18n 键 `profile.layersDisabled`，zh-CN/en 均已添加；同时放开不兼容 profile 的编辑禁用，crosshair 重新成为权威配置）
- [x] 4.4 检查 `SettingsApp.tsx`、`ProfileManager.tsx` 等其他组件是否暴露图层相关入口，一并门控 → grep 确认无引用，无需修改

## 5. 配置兼容验证

- [x] 5.1 验证含 `layers` 且 `crosshair: null` 的配置：overlay 显示默认准星（`overlay_renderer.rs` 既有 `unwrap_or_else(Crosshair::default_crosshair)` 兜底）、前端经 `layerToCrosshair(layers[0])` 反向生成准星设置，不报错（代码路径静态确认；Windows 运行时验证待 CI/实机）
- [x] 5.2 验证纯旧 `crosshair` 配置：渲染外观与软关闭前一致（`use_new_format` 对纯旧配置本就为 false，行为不变）
- [x] 5.3 验证保存配置后 config.json 中 `layers` 数据原样保留（`updateCrosshair` 仅同步 `layers[0]` 镜像 crosshair 编辑，其余图层原样保留，格式不降级；后端 `validate` 对两者共存以 layers 为准校验、不清除 crosshair）

## 6. 测试与质量

- [x] 6.1 `cargo test`（三个库 crate 共 108 项测试全部通过；`src-tauri` 在本 Linux 环境因缺 glib 无法编译，依赖 Windows CI）
- [x] 6.2 `cargo clippy -- -D warnings` 与 `cargo fmt --check` 通过（Linux target；windows-gnu target 下 `platform/windows.rs` 存在两处既有 doc 格式 lint，与本次改动无关，未触碰该文件）
- [x] 6.3 `npm run build`（前端 TypeScript 检查 + 构建）通过
- [x] 6.4 如有受影响的既有测试（引用动态调度或 layers 渲染路径），按软关闭语义修正，不删除物料 crate 自身测试 → 无受影响测试，`peregrine_material` 全部测试原样通过

## 7. 收尾

- [x] 7.1 更新 `AGENTS.md`：标注物料运行时处于软关闭状态、开关常量位置与恢复方式
- [x] 7.2 全局 grep `MATERIAL_RUNTIME_ENABLED` 枚举所有门控点，确认每处均有统一注释标记（共 8 处：Rust 2 文件、TS 3 文件、overlay.rs 2 处注释，均已枚举确认）
- [x] 7.3 向用户确认三个进行中物料相关变更（`overlay-dynamic-text-fixes`、`material-e2e-validation`、`material-docs-examples`）的处置方式 → 用户决定**保留不动**，等待后续重新启用物料功能时继续推进
