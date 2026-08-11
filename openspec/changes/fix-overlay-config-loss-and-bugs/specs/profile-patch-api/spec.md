## ADDED Requirements

### Requirement: Profile 字段级 patch 更新命令

系统 SHALL 提供后端 Tauri 命令 `update_profile_field(profile_name, update)`，按字段级 patch 更新指定 Profile 的顶层字段（`target_window`、`settings_hotkey`），**不触及 `layers` 与 `crosshair` 字段**。该命令 MUST 用于替代多图层编辑链路下的全量 `save_config` 调用，避免前端内存态的旧 layers 覆盖后端最新值。

命令参数 MUST 使用强类型枚举 `ProfileFieldUpdate`（而非 `serde_json::Value`），保证字段名与值类型在编译期可检查。命令执行成功后 MUST 通过 `persist_and_broadcast` 持久化并广播配置变更，同步推送 `OverlayCommand::UpdateConfig` 给 overlay 线程。

#### Scenario: 多图层模式下修改 target_window 不丢失图层

- **WHEN** 用户在多图层编辑器中已添加 3 个图层（A、B、C），然后修改 `target_window` 为「游戏窗口」
- **THEN** 后端 `update_profile_field` 命令被调用，仅更新 `target_window` 字段
- **AND** 后端 shared snapshot 中 `profile.layers` 仍为 `[A, B, C]`，不被覆盖
- **AND** overlay 收到新配置快照并继续按 3 个图层渲染

#### Scenario: patch API 不触及 layers 字段

- **WHEN** 调用 `update_profile_field("default", { kind: "target_window", value: "新窗口" })`
- **THEN** 后端 `profile.layers` 字段保持调用前的值不变
- **AND** 前端不需要在调用前同步整个 config

#### Scenario: 切换不存在的 Profile 报错

- **WHEN** 调用 `update_profile_field("不存在", { kind: "target_window", value: "X" })`
- **THEN** 命令返回 `IpcError { code: "NOT_FOUND", message: "profile '不存在' not found" }`
- **AND** 后端配置不被修改

#### Scenario: settings_hotkey 字段更新

- **WHEN** 调用 `update_profile_field("default", { kind: "settings_hotkey", value: "F12" })`
- **THEN** 后端 `profile.settings_hotkey` 更新为 `"F12"`
- **AND** 持久化到配置文件并广播给 overlay

### Requirement: peregrine:layers-changed 事件同步整个 config 到前端

前端 `LayersEditor` 在监听 `peregrine:layers-changed` 事件时，SHALL 除调用 `listLayers()` 刷新图层列表外，**同时调用 `getConfig()` 同步整个 config 到 ConfigApp 的 setConfig**。该兜底机制 MUST 防止前端内存态 config 在图层操作后过期，作为 `update_profile_field` patch API 的二层防御。

#### Scenario: 图层操作后前端 config 自动同步

- **WHEN** 用户通过 `update_layer` 命令修改了某图层的参数，后端 emit `peregrine:layers-changed` 事件
- **THEN** 前端 `LayersEditor` 监听器同时执行 `listLayers()` 和 `getConfig()`
- **AND** ConfigApp 的 `config` state 更新为后端最新快照（包含最新 layers）

#### Scenario: 连续多次图层操作不产生竞态

- **WHEN** 用户连续快速触发多次图层操作（如连续点击显隐切换）
- **THEN** 每次事件都触发一次 `getConfig()` 同步
- **AND** 最终前端 config 与后端一致（允许中间瞬时状态不一致）
