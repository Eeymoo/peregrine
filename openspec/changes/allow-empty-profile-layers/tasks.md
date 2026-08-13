# 任务清单：allow-empty-profile-layers

## 1. Schema 放宽与统一谓词

- [x] 1.1 `crates/config/src/schema.rs`：`Profile::validate` 删除「profile has neither layers nor crosshair」Err 分支，空 layers + None crosshair 合法
- [x] 1.2 `crates/config/src/schema.rs`：新增 `Profile::has_renderable_content(&self) -> bool`（`crosshair.is_some() || layers.iter().any(|l| l.visible)`），附中文文档注释
- [x] 1.3 更新 schema 单元测试：原「空 layers + None crosshair 报错」用例改为断言合法；新增 `has_renderable_content` 用例（空 layers / 全隐藏 / 部分可见 / 纯 crosshair）

## 2. 渲染路径修正（overlay_renderer）

- [x] 2.1 `crates/peregrine/src/overlay_renderer.rs`：`use_new_format` 判定改为 `MATERIAL_RUNTIME_ENABLED && profile.map(|p| !p.layers.is_empty() || p.crosshair.is_none()).unwrap_or(false)`，更新注释说明 crosshair 存在与否是新旧格式标志
- [x] 2.2 验证空 layers + None crosshair 时 overlay 渲染空白（与预览 `build_shapes_ipc` 行为一致），legacy 配置（crosshair=Some）行为不变

## 3. 后端硬校验（src-tauri）

- [x] 3.1 `src-tauri/src/lib.rs` `start_overlay`：活跃 profile `!has_renderable_content()` 时返回 `Err`（走 `translate` 双语文案，新增 `backend.no_renderable_content` key）
- [x] 3.2 `src-tauri/src/lib.rs` `remove_layer`：overlay 活动（`overlay_active`）且删除后剩余可见层数 == 0 且 `crosshair.is_none()` 时返回 `Err`，不修改配置、不广播事件
- [x] 3.3 `src-tauri/src/lib.rs` `update_layer`：`patch.visible == Some(false)` 且该层当前可见且其余可见层数 == 0 且 overlay 活动且 `crosshair.is_none()` 时返回 `Err`，不修改配置、不广播事件
- [x] 3.4 确认三处校验失败路径均在 `persist_and_broadcast` / `emit_layers_changed` 之前返回（无副作用）

## 4. 前端 UX 层

- [x] 4.1 `overlayActive` 透传：`ConfigApp.tsx` → `LayersEditor.tsx` → `LayerPanel.tsx` 新增 prop
- [x] 4.2 `LayerPanel.tsx`：overlay 活动中禁用「最后可见图层」的删除按钮与可见性切换按钮 + tooltip 说明原因
- [x] 4.3 开始覆盖按钮：活跃 profile 可见图层数为 0 且无 crosshair 时禁用 + tooltip（`useOverlayActions.handleStartOverlay` 同步前置 return 兜底）
- [x] 4.4 `LayerPanel.tsx` 各 handler（handleDelete / handleDuplicate / handleMove / handleToggleVisible / handleToggleLock）补 `.catch(() => {})`，消除 unhandled rejection（invoke 包装已负责 toast）
- [x] 4.5 `ConfigApp.tsx`：修正空锚点早退拦截——`!crosshair && !hasLayers` 仅在单图层模式拦截（提示页 + 生效的切换入口），多图层模式放行至图层编辑器空态；「配置格式异常」文案改为空态提示（`config.emptyLayers`，6 语）

## 5. i18n 与文案

- [x] 5.1 新增后端文案 key（`backend.no_renderable_content`、渲染中禁止删除/隐藏最后图层）并接入 `translate` 现有 locale 机制
- [x] 5.2 前端 6 语 locale（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）同步新增：开始覆盖禁用提示、最后可见图层禁用提示

## 6. 验证

- [x] 6.1 `cargo test -p peregrine_config` 通过（schema 测试更新）
- [x] 6.2 `cargo clippy -p peregrine_config -- -D warnings` 与 `cargo fmt --check` 通过
- [x] 6.3 `npm run build`（tsc 类型检查 + vite build）通过
- [x] 6.4 手动复现原事故链：多图层模式删除最后一个图层 → 成功且无 unhandled rejection；空图层点开始覆盖 → 按钮禁用/后端 Err；渲染中删/隐最后可见层 → 按钮禁用/后端 Err
- [x] 6.5 复现「配置格式异常死循环」：删光图层 → 多图层模式直接呈现图层编辑器空态；单图层模式显示空态提示，切换按钮生效离开提示页
