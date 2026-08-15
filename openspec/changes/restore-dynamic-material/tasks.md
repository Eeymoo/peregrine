# 任务清单：restore-dynamic-material

## 1. Schema 与常量

- [x] 1.1 `crates/config/src/schema.rs`：新增 `MaterialSettings`（`dynamic_enabled: bool` 默认 true、`fps: Option<u32>` 限 30/60/120）挂到 `AppSettings.material`，`#[serde(default)]`；`AppConfig::validate()` 校验 fps 枚举集；补默认值 / serde 往返（旧配置缺失字段）/ 非法 fps 校验失败三类测试
- [x] 1.2 `src/types/config.ts`：`MaterialSettings` 类型（`dynamicEnabled: boolean`、`fps?: 30 | 60 | 120`）
- [x] 1.3 `crates/peregrine/src/lib.rs` + `src/lib/feature.ts`：`MATERIAL_DYNAMIC_INPUT_ENABLED` 翻 `true`，注释同步（恢复语义 + 回退方式）
- [x] 1.4 全局 grep `MATERIAL_DYNAMIC_INPUT_ENABLED` / `MATERIAL_RUNTIME_ENABLED` 枚举门控点，注释统一更新

## 2. 双层开关联动（design D2）

- [x] 2.1 `crates/peregrine/src/overlay_renderer.rs`：两处动态上下文选择改为合取 `MATERIAL_DYNAMIC_INPUT_ENABLED && settings.material.dynamic_enabled`——运行时关闭走 `static_context()`
- [x] 2.2 `src-tauri/src/lib.rs` 预览 IPC（`build_shapes_ipc`）：同上合取门控
- [x] 2.3 `src/components/LayerPanel.tsx`：物料选择器过滤条件改为合取（运行时关闭时隐藏 `is_dynamic` 物料与动态徽章）
- [x] 2.4 `src-tauri/src/overlay.rs` `compute_is_animated`：layers 分支纳入合取（运行时关闭恒 false）

## 3. 调度与帧率（design D3/D4/D6）

- [x] 3.1 `src-tauri/src/overlay.rs`：`frame_interval` 改为启动时从配置解析（`fps` → 系统刷新率 → 60 兜底），`UpdateConfig` 时热更新
- [x] 3.2 系统刷新率探测：overlay 线程窗口创建后经 `MonitorHandle::refresh_rate_millihertz()` 探测一次并缓存；异常值（<24 或 >480 Hz）回退 60
- [x] 3.3 `compute_is_animated` 扩展 layers 分支：任一可见图层物料 `is_dynamic`（经 registry 查询）且合取开关为真 → true；不引入缓存字段
- [x] 3.4 RandomOrb 保持既有分支，消费同一 `frame_interval`（行为随配置节拍，注释说明）
- [x] 3.5 `OverlayCommand::RefreshMaterials` 签名改为携带 `Arc<MaterialRegistry>`：overlay 侧替换 `material_registry` 句柄 + `needs_redraw = true` + `request_redraw`

## 4. Watcher 接线（design D9）

- [x] 4.1 `src-tauri/src/lib.rs` 物料 watcher：重建 registry 后经 proxy 发送 `RefreshMaterials(new_registry)` 给 overlay 线程
- [x] 4.2 补 TODO：`app.emit("peregrine:materials-changed", ())` 通知前端刷新物料列表
- [x] 4.3 前端监听 `peregrine:materials-changed` 重拉物料列表（`list_materials` IPC）

## 5. 时间物料归位与 API 统一（design D7/D10）

- [x] 5.1 `examples/time.rhai` → `crates/material/builtin/time.rhai`；`crates/material/src/lib.rs` `BUILTIN_MATERIALS` 加入 `"time"`
- [x] 5.2 `builtin/time.rhai` 与 `examples/clock.rhai`：`now_ms()` 改 `time_ms()`；`now_ms()` host function 保留注册，代码注释标注「不推荐，新脚本用 time_ms()」
- [x] 5.3 `crates/material` 单测：time 物料求值使用注入上下文的时间（固定 `DynamicContext.time_ms` → 输出对应时刻文本），防逃逸回归
- [x] 5.4 `crates/peregrine/src/shapes.rs` / `overlay_renderer.rs`：与求值缓存相关的陈旧注释清理（「version=0 永久缓存」表述删除）

## 6. 设置 UI：「物料」Tab（design D2/D3）

- [x] 6.1 `src/components/settings/MaterialTab.tsx` 新建：动态物料开关（Switch）+ 帧率选择器（系统 / 30 / 60 / 120，单选组）
- [x] 6.2 `src/SettingsApp.tsx`：注册「物料」Tab（图标、排序、路由）
- [x] 6.3 保存链路：开关 / FPS 变更走 `save_config` 防抖保存（复用现有 useConfigSave）
- [x] 6.4 i18n 六语（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）locale 补齐：Tab 名、开关标签、帧率标签（含「跟随系统」）、说明文案
- [x] 6.5 `i18n-audit` 流程跑一遍（key 对齐 + 无硬编码）

## 7. 预览实时跳动（design D8）

- [x] 7.1 `src/components/Preview.tsx`：profile 含 `is_dynamic` 物料且运行时开关开 → `setInterval(1000ms)` 重拉 `build_shapes_ipc`；条件变化 / 卸载时清理
- [x] 7.2 前端物料列表缓存 `is_dynamic` 信息供 Preview 判定（经 `list_materials`）

## 8. 规格同步与旧 change 归档

- [x] 8.1 spec deltas 编写（见 specs/ 目录，随本 tasks 一并评审）
- [x] 8.2 `openspec/changes/overlay-dynamic-text-fixes` / `material-e2e-validation` / `material-docs-examples` → `archive/2026-08-14-*`；三者 proposal.md 头部加「由 restore-dynamic-material 取代」说明
- [x] 8.3 `AGENTS.md`：动态物料段落重写（软关闭 → 已恢复，双层开关语义，FPS 设置）
- [x] 8.4 主 specs 归档同步时移除「软关闭期间不可达」注记（`overlay-dynamic-rendering` / `material-dynamic-input`）

## 9. 回归验证

- [x] 9.1 `cargo test -p peregrine_config -p peregrine_material -p peregrine` 全通过
- [x] 9.2 `cargo clippy`（3 crate）+ `cargo fmt --check` 通过
- [x] 9.3 `npm run build`（tsc + vite）通过
- [ ] 9.4 Windows 实机（release）：时钟物料每秒跳动（30/60/120 三档 + 系统档），无窗口交互
- [ ] 9.5 Windows 实机：纯静态 profile overlay 无空转（任务管理器 CPU 与现状一致）
- [ ] 9.6 Windows 实机：运行时开关热切换（开 → 关 → 开）即时生效，无需重启
- [ ] 9.7 Windows 实机：物料目录热重载（改 `is_dynamic` 或新增 user 物料）overlay 无重启即感知
- [ ] 9.8 Windows 实机：预览中时钟跳动与 overlay 显示一致（同一时刻 ±1s）
- [x] 9.9 手动验证清单补 B4 区动态物料条目（标注本次结果）
