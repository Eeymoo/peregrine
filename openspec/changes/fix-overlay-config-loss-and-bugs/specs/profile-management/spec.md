## MODIFIED Requirements

### Requirement: 多 Profile 的增删改查与复制

系统 SHALL 提供多个命名 Profile 的管理能力：列表查询、新建、重命名、删除、复制。系统 MUST 始终保留至少一个 Profile，删除最后一个 Profile 的请求 MUST 被拒绝。新建的 Profile MUST 默认为单图层兼容配置；复制的 Profile MUST 包含源 Profile 的全部图层与设置，且名称唯一不冲突。

所有 Profile 管理命令（`create_profile` / `rename_profile` / `delete_profile` / `set_active_profile` / `copy_profile`）SHALL 返回 `Result<T, IpcError>`（结构化错误对象），不再返回 `Result<T, String>`。错误对象包含 `code`（稳定错误码如 `NOT_FOUND` / `VALIDATION`）与 `message`（人类可读中文描述）。

#### Scenario: 新建 Profile 默认为单图层兼容

- **WHEN** 用户在 ProfileManager 中新建名为 `测试A` 的 Profile
- **THEN** 系统创建 `测试A` 并自动切换为激活 Profile
- **AND** `测试A` 满足 `is_legacy_compatible`（单图层、内置基础物料、参数等于物料 defaults）

#### Scenario: 重命名为已存在名称被拒绝（结构化错误）

- **WHEN** 用户将 Profile 重命名为一个已存在的名称
- **THEN** 系统拒绝该请求，返回 `IpcError { code: "VALIDATION", message: "..." }`
- **AND** 原 Profile 保持不变

#### Scenario: 删除最后一个 Profile 被拒绝

- **WHEN** 用户尝试删除当前仅剩的最后一个 Profile
- **THEN** 系统拒绝该请求并返回错误，配置保持至少一个 Profile

#### Scenario: 复制 Profile 内容一致且名称唯一

- **WHEN** 用户复制 Profile `测试A`
- **THEN** 系统生成唯一名称的副本（如 `测试A Copy`）
- **AND** 副本的图层、样式参数与 `测试A` 完全一致

## ADDED Requirements

### Requirement: 多图层模式下 profile 字段变更走 patch API

多图层编辑器（`LayersEditor`）中所有修改 Profile 顶层字段的操作（如修改 `target_window`）SHALL 通过 `update_profile_field` patch API 更新，**不再调用 `saveConfig(newConfig)` 全量覆盖**。前端内存态 config 不得作为 `saveConfig` 的数据源覆盖后端。

#### Scenario: 多图层模式下修改 target_window 不丢失图层

- **WHEN** 用户在多图层编辑器中已添加多个图层并修改了它们的参数，然后修改 `target_window`
- **THEN** 前端调用 `update_profile_field("default", { kind: "target_window", value: "新窗口" })`
- **AND** 不调用 `saveConfig`
- **AND** 后端 profile.layers 保持不变（不丢失已添加的图层）

#### Scenario: 多图层模式下切换 settings_hotkey 不丢失图层

- **WHEN** 用户在多图层编辑器中修改快捷键字段
- **THEN** 同样通过 patch API 更新，不触及 layers 字段
