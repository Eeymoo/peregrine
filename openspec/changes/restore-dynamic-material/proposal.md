# restore-dynamic-material 提案

> 取代关系：本 change 收编并取代三个挂起变更——`overlay-dynamic-text-fixes`、`material-e2e-validation`、`material-docs-examples`（三者随本 change 归档，见 tasks §8；其未竟事项中与本 change 范围重叠的部分已并入下方 What Changes，其余宣告放弃）。

## Why

动态物料链路（layers + Rhai 求值 + 动态调度）自 `material-static-rendering` 起处于「仅动态输入软关闭」状态：静态多图层渲染活跃，但 `MATERIAL_DYNAMIC_INPUT_ENABLED = false` 导致动态物料冻结渲染、选择器隐藏、重绘调度缺失。探索性审计（2026-08-14）确认：

1. **调度机制已删除**：`compute_is_animated`（src-tauri/src/overlay.rs:144）只识别旧格式 RandomOrb，layers 物料的 `is_dynamic` 判定与 `dynamic_dirty` 缓存已被 `disable-material-runtime` 删除；`OverlayCommand::RefreshMaterials` 退化为普通重绘；物料 watcher 的通知 TODO（src-tauri/src/lib.rs:2355）未接线。仅翻编译期常量得到的将是「冻结但偶发跳帧」的残废动态物料。
2. **元数据与文档漂移**：规格声称「静态物料求值缓存因 version=0 永久命中」，但代码中不存在任何求值缓存（`build_layers_shapes` 每帧全量求值）；`DynamicContext.version` 字段无消费者。
3. **时间 API 逃逸**：`time.rhai`（现居 `examples/`）使用 `now_ms()` 直读墙钟而非上下文快照 `time_ms()`，预览与 overlay 求值时刻不一致，WYSIWYG 漂移。
4. **帧率硬编码**：`frame_interval` 固定 16.67ms（≈60FPS），无用户可选档位。

本 change 以「跑通动态物料端到端」为唯一核心目标，用最小机制恢复完整链路，不做性能优化（缓存）、不做输入分级协议。

## What Changes

### 动态链路恢复（核心）

- 编译期开关 `MATERIAL_DYNAMIC_INPUT_ENABLED` 翻回 `true`（Rust `crates/peregrine/src/lib.rs` + TS `src/lib/feature.ts` 成对），保留编译期总闸语义（翻回 `false` 即整体回退软关闭）。
- 运行时新增用户开关：`settings.material.dynamic_enabled`（默认 `true`），与编译期开关联动门控动态链路（详见 design D2）。
- `compute_is_animated` 扩展：active profile 含**可见**图层引用 `is_dynamic` 物料且运行时开关开启 → 持续重绘；沿用「每轮 about_to_wait 直接计算」哲学，不恢复 `dynamic_dirty` 缓存（详见 design D4）。
- `frame_interval` 配置化：从 `settings.material.fps` 推导，`UpdateConfig` 时热更新。
- `OverlayCommand::RefreshMaterials` 恢写实义：更新 overlay 线程持有的 registry 句柄 + 触发重绘；物料 watcher 补接线（重建 registry 后发送该命令 + `app.emit("peregrine:materials-changed")`）。
- `time.rhai` 归位 `builtin/`（`BUILTIN_MATERIALS` 列表同步），时钟成为开箱即用的动态物料。

### 设置侧：新增「物料」Tab

- 设置窗口新增「物料」Tab，包含：
  - **动态物料开关**（`dynamic_enabled`）；
  - **动画帧率选择器**（`fps: Option<u32>`，档位 30 / 60 / 120，默认 `None` = 跟随系统主屏刷新率，检测失败回退 60）。
- FPS 语义为「动画最高帧率节拍（cap）」：纯静态 profile 不受影响；时钟等低频物料按需渲染，帧率档位约束的是调度节拍上限（详见 design D6）。

### 预览实时跳动

- `Preview.tsx` 检测 profile 含动态物料时启动 ~1s 定时器重拉 `build_shapes_ipc`，时钟在预览中实时跳动；无动态物料时维持现有事件驱动。

### 时间 API 统一

- `builtin/time.rhai` 改用 `time_ms()`（上下文快照），消除 `now_ms()` 逃逸导致的预览/overlay 漂移；`now_ms()` host function 保留注册（兼容既有用户脚本），文档标注为不推荐。

### 范围内小修

- `LayerPanel.tsx` 物料选择器：动态开关联动放开 `is_dynamic` 过滤，恢复「动态」徽章。
- `AGENTS.md` 软关闭描述更新；`openspec/specs/` 相关规格的「软关闭不可达」注记移除。
- 旧 change 归档：`overlay-dynamic-text-fixes` / `material-e2e-validation` / `material-docs-examples` 移入 `archive/2026-08-14-*`，proposal 头部加取代说明（tasks §8）。

## Non-Goals

- **鼠标 / 键盘动态输入的用户侧启用**：`poll_dynamic_context` 随编译期开关恢复全量采集（成本极低），但 UI 仅提供 `builtin.time` 一个动态物料，鼠标跟随 / 键盘响应物料仍留在 `examples/` 不内置。
- **求值缓存 / 输入分级协议 / 秒边界对齐调度**：120FPS 下每秒全量求值的浪费作为已知取舍记录（design D5），留待后续 change。
- **性能基线 / 1h 稳定性验证**（原 `material-e2e-validation` 主体）：本 change 仅做功能性实机验证；性能基线宣告放弃，将来需要时另立 change。
- **物料创作文档**（原 `material-docs-examples` 主体）：宣告放弃；`examples/` 三份示例脚本保留在仓库。

## Capabilities

### New Capabilities

- `material-settings`：「物料」设置 Tab——动态物料运行时开关、动画帧率档位（系统/30/60/120）及其语义。

### Modified Capabilities

- `material-dynamic-input`：软关闭语义从编译期单开关改为「编译期总闸 + 运行时用户开关」双层；物料选择器可见性随运行时开关联动。
- `overlay-dynamic-rendering`：持续重绘帧率由 `settings.material.fps` 推导（系统刷新率默认）；移除「软关闭期间不可达」注记；`RefreshMaterials` 恢复实义。
- `material-runtime`：`time.rhai` 回归内置；内置时间物料使用上下文时间而非直读墙钟。

## Impact

- **代码**：
  - `crates/config/src/schema.rs`（`MaterialSettings` 结构 + 校验 + 测试）
  - `crates/peregrine/src/lib.rs`、`src/lib/feature.ts`（常量翻转）
  - `src-tauri/src/overlay.rs`（`compute_is_animated` 扩展、`frame_interval` 配置化、`RefreshMaterials` 恢复、系统刷新率探测）
  - `src-tauri/src/lib.rs`（watcher 接线 + `materials-changed` emit + 预览 IPC 开关联动）
  - `crates/peregrine/src/overlay_renderer.rs` / `shapes.rs`（无结构改动，注释同步）
  - `crates/material/builtin/time.rhai` 归位 + `time_ms()` 改造；`crates/material/src/lib.rs`（`BUILTIN_MATERIALS`）
  - `src/SettingsApp.tsx` + `src/components/settings/MaterialTab.tsx`（新）+ `src/components/LayerPanel.tsx` + `src/components/Preview.tsx` + `src/types/config.ts`
  - i18n 六语 locale 补齐
- **用户配置**：新增 `settings.material` 字段全部 `#[serde(default)]`，旧配置无感升级；`fps = None` 缺省即「跟随系统」。
- **行为变更**：动态开关联动影响 overlay CPU 占用（含动态图层时从空闲变为按 FPS 节拍渲染）——预期内。
- **文档**：`AGENTS.md` 动态物料段落重写；主 specs 注记同步。

## 发布版本

随下一 preview（奇数版本号）发布，实机验证通过后随 stable 合并。
