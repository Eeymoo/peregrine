## 1. 后端 IPC 错误协议改造（#27 根因）

- [x] 1.1 在 `src-tauri/src/lib.rs` 定义 `IpcError` 结构体（`code: String, message: String` + `#[derive(serde::Serialize)]`）
- [x] 1.2 定义错误码常量（`VALIDATION` / `NOT_FOUND` / `INTERNAL`），可放 `telemetry.rs` 或独立模块
- [x] 1.3 改造 `save_config` 命令返回 `Result<(), IpcError>`，map_err 时填充 code
- [x] 1.4 改造 `add_layer` / `remove_layer` / `move_layer` / `duplicate_layer` 命令返回 `Result<T, IpcError>`
- [x] 1.5 改造 `update_layer` 命令返回 `Result<(), IpcError>`，校验失败 code=VALIDATION
- [x] 1.6 改造 `create_profile` / `rename_profile` / `delete_profile` / `set_active_profile` / `copy_profile` 命令返回 `Result<T, IpcError>`
- [x] 1.7 `persist_and_broadcast` 内部 helper 错误类型同步升级
- [x] 1.8 `safe_try!` 宏适配新的错误类型（保持 PGR-2101 上报不变）
- [x] 1.9 `cargo build -p peregrine-tauri` 通过、`cargo clippy` 无警告

## 2. 后端 patch API 新增（#34 根治）

- [x] 2.1 在 `crates/config/src/schema.rs` 或 `src-tauri/src/lib.rs` 定义 `ProfileFieldUpdate` 强类型枚举（`TargetWindow { value }` / `SettingsHotkey { value }`）
- [x] 2.2 新增 Tauri 命令 `update_profile_field(profile_name: String, update: ProfileFieldUpdate) -> Result<(), IpcError>`
- [x] 2.3 实现仅 patch 对应字段、不触及 layers/crosshair 的逻辑
- [x] 2.4 复用 `persist_and_broadcast` 持久化 + 广播 + 推送 OverlayCommand::UpdateConfig
- [x] 2.5 在 `invoke_handler` 注册新命令
- [x] 2.6 单元测试覆盖：修改 target_window 不影响 layers、profile 不存在返回 NOT_FOUND
- [x] 2.7 `cargo test -p peregrine_config` 通过

## 3. 前端 IPC 统一包装（#27 + #31）

- [x] 3.1 在 `src/lib/api.ts` 新增统一 `invoke<T>(cmd, args?)` 包装函数，捕获 reject 并包装为 Error + showToast
- [x] 3.2 复用 `src/lib/globalErrorToast.ts` 的 `showToast` 函数（必要时 export）
- [x] 3.3 定义前端 `IpcError` 类型（与后端对应），处理「对象」和「字符串」两种 reject 形态（兼容过渡期）
- [x] 3.4 新增 `updateProfileField(name, update)` 包装函数
- [x] 3.5 所有现有 `invoke` 直接调用改为走新包装（除真正可忽略的场景）

## 4. 前端图层操作错误处理（#31）

- [x] 4.1 `src/components/LayerPanel.tsx`：`handleAdd` / `handleDelete` / `handleMove` / `handleDuplicate` / `handleToggleVisible` / `handleToggleLock` 全部加 try/catch（调用方 catch 不再强制 toast，由 invoke 包装负责）
- [x] 4.2 `src/components/LayerEditors.tsx`：`LayerStyleEditor.update` / `LayerTransformEditor.update` 加 try/catch
- [x] 4.3 `src/components/LayerPanel.tsx` 的 `MaterialParamControls.updateParam` 加 try/catch
- [x] 4.4 `src/components/LayersEditor.tsx` 的 `updateLayerName` 移除 `console.error("updateLayerName failed")`，改由 invoke 包装 toast
- [x] 4.5 `src/hooks/useConfigSave.ts` 的 `debouncedSave` 移除 `.catch(console.error)`，改用统一 invoke
- [x] 4.6 `src/components/LayersEditor.tsx` 的 `updateTargetWindow` 移除 `.catch(console.error)`
- [x] 4.7 审视 `src/hooks/useOverlayActions.ts`、`src/ConfigApp.tsx`、`src/SettingsApp.tsx` 中剩余的 `.catch(console.error)`，可忽略的改为 `.catch(() => {})` 并加注释
- [x] 4.8 `npm run build` 通过、TypeScript 检查无错

## 5. 配置丢失根治（#34）

- [x] 5.1 `src/components/LayersEditor.tsx` 的 `updateTargetWindow` 改调 `updateProfileField(name, { kind: "target_window", value })`，不再调 `saveConfig`
- [x] 5.2 `src/components/LayersEditor.tsx` 的 `peregrine:layers-changed` 事件监听器除 `refresh()` 外，同时调 `getConfig()` → 通过新增的 `onConfigChange` 回调同步整个 config
- [x] 5.3 验证 ConfigApp 传入 LayersEditor 的 `onConfigChange` 正确接到 `setConfig`（已有，确认即可）
- [ ] 5.4 手测：多图层模式下加 3 个图层、修改参数，然后改 target_window，确认后端 layers 不丢失
- [x] 5.5 审视 `src/hooks/useConfigSave.ts` 的 `updateCrosshair`（单图层模式 hook）是否在多图层模式下产生副作用，确认 layers[1..n] 不被丢弃

## 6. 全局对话框层重构（#28）

- [x] 6.1 重构 `src/ConfigApp.tsx` 的渲染结构：把 `if (layersMode) return <LayersEditor/>` 改为三元表达式，与单图层模式并列
- [x] 6.2 把 `<AutoSwitchDialog>` 从单图层 return 内移到最外层（layersMode 判断之后）
- [x] 6.3 把 `<UpdateDialog>` 同上移到全局层
- [x] 6.4 把 `<UpdateProgress>` 同上移到全局层
- [x] 6.5 确认所有 dialog 用到的 state（showAutoSwitchDialog / updateAvailable / updating）与回调（saveAutoSwitchPreference / startUpdate 等）在两种模式下都可用
- [ ] 6.6 手测：多图层模式下点「开始覆盖」，AutoSwitchDialog 正常弹出
- [ ] 6.7 手测：多图层模式下检测到新版本，UpdateDialog 正常弹出

## 7. 透明度显示修复（#32）

- [x] 7.1 `src/components/fields/SliderField.tsx` 新增 `format?: (v: number) => string` 可选参数
- [x] 7.2 SliderField 渲染逻辑：传入 format 时显示 `format(value)` 且隐藏 unit；否则保持原行为
- [x] 7.3 `src/ConfigApp.tsx:257` 单图层 opacity 改为 `{Math.round(ch.opacity * 100)}%`
- [x] 7.4 `src/components/LayerEditors.tsx:50` 多图层 opacity SliderField 传 `format={(v) => Math.round(v * 100) + "%"}`，移除 `unit="%"`
- [ ] 7.5 手测：两种模式下 opacity=0.5 都显示为 `50%`

## 8. grid.rhai 算法修复（#29）

- [ ] 8.1 `crates/material/builtin/grid.rhai` cols/rows 计算改用 `floor`（`int(w / cell)`）
- [ ] 8.2 edge 模式 cell_w 直接用 `cell`（用户设定值），不再用 `w / cols` 重算
- [ ] 8.3 center 模式 total_w = cell * cols（保证不超屏）
- [ ] 8.4 手测：grid_size=200 / 1920 屏 → 9 列（非 10）
- [ ] 8.5 手测：grid_size=120 / 1920 屏 → 16 列（填满）

## 9. 物料 dead parameter 处理（#30）

- [ ] 9.1 `crates/material/builtin/border_frame.rhai`：build 根据 `params.inset` 控制 offset 符号或位置（inset=true 内偏、inset=false 贴边）
- [ ] 9.2 `crates/config/src/schema.rs`：`Element::Rect` 增加 `corner_radius: Option<f32>` 字段（serde default）
- [ ] 9.3 `crates/material/builtin/edge_rect.rhai`：build 输出 Rect shape 携带 `corner_radius: params.corner_radius`
- [ ] 9.4 `crates/peregrine/src/svg_renderer.rs`：SVG `<rect>` 支持 `rx` 属性
- [ ] 9.5 `crates/peregrine/src/overlay_renderer.rs`：CPU 光栅化圆角矩形（若实现复杂则降级直角 + warn 日志）
- [ ] 9.6 `crates/material/builtin/random_orb.rhai`：build 实现中心规避（拒绝采样，重试上限 10）
- [ ] 9.7 `crates/material/builtin/random_orb.rhai`：mode select 控件标记为 disabled / 加 "coming soon" 提示（通过 schema 返回 special 字段，或前端 MaterialParamControls 识别 mode 字段加 disabled）
- [ ] 9.8 `crates/peregrine/src/shapes.rs` 测试：corner_radius 字段反序列化向后兼容
- [ ] 9.9 `cargo test -p peregrine_config -p peregrine -p peregrine_material` 全部通过

## 10. 物料 schema slider max 扩充（#33）

- [ ] 10.1 按 proposal 分级表修改 `crates/material/builtin/cross.rhai` schema max（size=1920, gap=200, thickness=50）
- [ ] 10.2 修改 `large_cross.rhai`（thickness=50）
- [ ] 10.3 修改 `corner_dots.rhai`（offset=1920, thickness=50, radius=500）
- [ ] 10.4 修改 `ring.rhai`（thickness=50，ring_radius_pct 不变）
- [ ] 10.5 修改 `custom_orb.rhai`（radius=500, offset=1920）
- [ ] 10.6 修改 `random_orb.rhai`（offset=1920, jitter=1920, radius_min/max=500，center_deviation 不变）
- [ ] 10.7 修改 `border_frame.rhai`（thickness=50, offset=1920）
- [ ] 10.8 修改 `edge_rect.rhai`（size/secondary_size/margin=1920, corner_radius=500）
- [ ] 10.9 修改 `edge_arrows.rhai`（size=400, distance=1920, width=200, tail_*=1920）
- [ ] 10.10 修改 `grid.rhai`（grid_size=1920, thickness=50）
- [ ] 10.11 修改 `image.rhai`（scale=50, offset_x/y=±1920，width/height 不变）
- [ ] 10.12 手测：cross 物料 size 可拖到 1920

## 11. README 嵌入视频（#35）

- [ ] 11.1 `README.md` 在「Quick Start」之前嵌入 bilibili iframe，用 `<div style="max-width: 100%">` 包裹
- [ ] 11.2 `README.zh-cn.md` 在「快速开始」之前同样嵌入
- [ ] 11.3 验证 GitHub 渲染 iframe 正常显示
- [ ] 11.4 验证 VitePress 文档站点（如有引用 README）渲染正常

## 12. 集成验证

- [ ] 12.1 `cargo fmt` 全 workspace
- [ ] 12.2 `cargo clippy --workspace -- -D warnings` 通过
- [ ] 12.3 `cargo test --workspace` 通过
- [ ] 12.4 `npm run build` 通过
- [ ] 12.5 `npx tauri build` 或 `npx tauri dev` 启动成功
- [ ] 12.6 手测回归：单图层模式所有操作正常（opacity 显示 %、滑块可拖、保存生效）
- [ ] 12.7 手测回归：多图层模式所有操作正常（加图层、改样式、改 target_window、切换 profile、启动 overlay + AutoSwitchDialog）
- [ ] 12.8 手测：配置丢失场景（多图层下加图层 + 改 target_window）不再丢图层
- [ ] 12.9 手测：错误反馈（拖 opacity 到越界值）显示 toast 而非静默
- [ ] 12.10 关闭已修复的 issue（#27~#35）
