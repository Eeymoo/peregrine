# profile-management Specification

## Purpose

定义 Peregrine 的多 Profile 管理能力：支持多个命名 Profile 的增删改查与复制，激活 Profile 切换与持久化，单图层兼容性判定，编辑器模式（单图层/多图层）的持久化与恢复，以及配置结构序列化兼容性。

## Requirements

### Requirement: 多 Profile 的增删改查与复制

系统 SHALL 提供多个命名 Profile 的管理能力：列表查询、新建、重命名、删除、复制。系统 MUST 始终保留至少一个 Profile，删除最后一个 Profile 的请求 MUST 被拒绝。新建的 Profile MUST 默认为单图层兼容配置；复制的 Profile MUST 包含源 Profile 的全部图层与设置，且名称唯一不冲突。

#### Scenario: 新建 Profile 默认为单图层兼容

- **WHEN** 用户在 ProfileManager 中新建名为 `测试A` 的 Profile
- **THEN** 系统创建 `测试A` 并自动切换为激活 Profile
- **AND** `测试A` 满足 `is_legacy_compatible`（单图层、内置基础物料、参数等于物料 defaults）

#### Scenario: 重命名为已存在名称被拒绝

- **WHEN** 用户将 Profile 重命名为一个已存在的名称
- **THEN** 系统拒绝该请求并返回错误，原 Profile 保持不变

#### Scenario: 删除最后一个 Profile 被拒绝

- **WHEN** 用户尝试删除当前仅剩的最后一个 Profile
- **THEN** 系统拒绝该请求并返回错误，配置保持至少一个 Profile

#### Scenario: 复制 Profile 内容一致且名称唯一

- **WHEN** 用户复制 Profile `测试A`
- **THEN** 系统生成唯一名称的副本（如 `测试A Copy`）
- **AND** 副本的图层、样式参数与 `测试A` 完全一致

### Requirement: 激活 Profile 切换与持久化

系统 SHALL 维护 `active_profile` 字段标识当前激活的 Profile。切换激活 Profile 后，系统 MUST 立即将新配置快照广播给 overlay 线程（不等待文件 watcher），overlay MUST 按新激活 Profile 渲染。`active_profile` MUST 随配置文件持久化，应用重启后恢复最后激活的 Profile。

#### Scenario: 切换激活 Profile 后 overlay 立即更新

- **WHEN** overlay 正在运行，用户在设置面板切换激活 Profile
- **THEN** overlay 无需重启即按新激活 Profile 的样式渲染

#### Scenario: 重启后恢复最后激活的 Profile

- **WHEN** 用户切换到 Profile `测试A` 后关闭并重新启动应用
- **THEN** 激活 Profile 为 `测试A`

#### Scenario: 切换到不存在的 Profile 报错

- **WHEN** 调用 `set_active_profile` 传入不存在的名称
- **THEN** 系统返回错误且 `active_profile` 不变

### Requirement: 单图层兼容性判定

系统 SHALL 提供 Profile 级单图层兼容性判定（`is_legacy_compatible`）。兼容条件 MUST 同时满足：仅含一个图层；该图层使用与旧版 Crosshair 样式对应的内置基础物料；无额外几何变换与非默认混合模式；图层 `params` 与物料 `defaults` 完全一致（逐字段比较，由 Tauri 命令层通过 `MaterialRegistry` 完成，config crate 只做快速预检查）。

#### Scenario: 默认新建 Profile 判定为兼容

- **WHEN** 对一个默认新建的单图层 Profile 调用兼容性判定
- **THEN** 返回兼容（`true`）

#### Scenario: 多图层 Profile 判定为不兼容

- **WHEN** 对含两个及以上图层的 Profile 调用兼容性判定
- **THEN** 返回不兼容（`false`）

#### Scenario: 参数偏离 defaults 判定为不兼容

- **WHEN** 单图层 Profile 的图层 `params` 存在物料 `defaults` 中没有的键或值不相等
- **THEN** 返回不兼容（`false`）

### Requirement: 单图层模式绑定激活 Profile

单图层（旧版）UI SHALL 只编辑当前激活的 Profile，且仅在其通过兼容性判定时可编辑。当激活 Profile 不兼容时，单图层 UI MUST 显示不兼容提示并禁用编辑控件，不得将多图层配置改坏。

#### Scenario: 兼容 Profile 在单图层模式正常编辑

- **WHEN** 激活 Profile 兼容，用户在单图层模式修改准星样式
- **THEN** 修改写入该 Profile 并即时生效

#### Scenario: 不兼容 Profile 禁用单图层编辑

- **WHEN** 激活 Profile 不兼容单图层模式
- **THEN** 单图层 UI 显示不兼容提示且编辑控件禁用

### Requirement: 编辑器模式持久化与恢复

系统 SHALL 将编辑器模式（`layersMode`，单图层/多图层）持久化到 `localStorage`。应用启动时 MUST 无差别恢复关闭前的模式：单图层关闭则单图层打开，多图层关闭则多图层打开；无持久化值时默认为单图层。恢复后 MUST 仍套用兼容性规则（单图层模式下激活 Profile 不兼容时提示并切换到多图层，并写回持久化值）。物料运行时软关闭期间 MUST NOT 对模式恢复做特判。

#### Scenario: 多图层模式关闭后恢复多图层

- **WHEN** 用户在多图层模式下关闭应用并重新启动
- **THEN** 启动后编辑器处于多图层模式

#### Scenario: 单图层模式关闭后恢复单图层

- **WHEN** 用户在单图层模式下关闭应用并重新启动
- **THEN** 启动后编辑器处于单图层模式

#### Scenario: 首次使用默认单图层

- **WHEN** 用户首次启动应用（无持久化模式值）
- **THEN** 编辑器处于单图层模式

### Requirement: 配置结构序列化兼容

多 Profile 改造 MUST NOT 新增破坏性的配置字段；既有配置文件（含旧版单 `crosshair` 格式，迁移规则见 `profile-migration`）升级后 MUST 保持可用，`AppConfig::validate` 与原子写入不变式不受影响。

#### Scenario: 既有配置升级后保持可用

- **WHEN** 应用加载多 Profile 改造前生成的配置文件
- **THEN** 配置正常反序列化并通过校验，应用正常启动
