# layer-composition 增量规格：allow-empty-profile-layers

## ADDED Requirements

### Requirement: 空图层列表是合法配置状态

Profile 的 `layers` 为空且 `crosshair` 为 `None` 时 MUST 视为合法配置，语义为「当前不显示任何锚点」。`Profile::validate` SHALL NOT 因此返回错误，配置 MUST 可正常持久化与加载。

该状态下渲染输出 SHALL 为空白（预览与 overlay 一致），SHALL NOT 回退渲染默认准星。

#### Scenario: 删除最后一个图层后配置合法

- **WHEN** 一个迁移后的 Profile（`crosshair = None`）只有 1 个图层
- **AND** overlay 未运行，用户删除该图层
- **THEN** `remove_layer` SHALL 成功
- **AND** 配置文件持久化为 `layers = []`、`crosshair = None`
- **AND** 下次启动时该配置 SHALL 被正常加载（不触发损坏备份回退）

#### Scenario: 空图层时预览与 overlay 渲染一致

- **WHEN** 活跃 Profile 的可见图层数为 0 且 `crosshair = None`
- **THEN** 预览（`build_shapes_ipc`）SHALL 返回空图元列表
- **AND** overlay 渲染路径（`build_layers_shapes`）SHALL 输出空图元列表
- **AND** 两处 SHALL NOT 回退到 `Crosshair::default_crosshair`

### Requirement: 渲染不变量——运行中至少保留一个可见图层

overlay 运行期间，活跃 Profile MUST 始终保持至少 1 个可见图层（或存在 legacy `crosshair`）。系统 SHALL 在后端拦截任何将使可见图层数归零的图层操作：

- `remove_layer`：overlay 活动且被删图层是最后一个可见图层且 `crosshair = None` 时 MUST 返回错误，不修改配置。
- `update_layer`：`patch.visible == false` 且目标图层是最后一个可见图层且 overlay 活动且 `crosshair = None` 时 MUST 返回错误，不修改配置。

以下操作 SHALL 不受影响：渲染中删除已隐藏的图层；渲染中可见图层数 ≥ 2 时删除/隐藏任一图层；overlay 未运行时的任何删除/隐藏。

#### Scenario: 渲染中删除最后可见图层被拒绝

- **WHEN** overlay 正在运行
- **AND** 活跃 Profile 仅剩 1 个可见图层且 `crosshair = None`
- **AND** 前端调用 `remove_layer` 删除该图层
- **THEN** 命令 SHALL 返回错误
- **AND** 配置不被修改
- **AND** 前端 toast 显示原因且不产生 unhandled rejection

#### Scenario: 渲染中隐藏最后可见图层被拒绝

- **WHEN** overlay 正在运行
- **AND** 活跃 Profile 仅剩 1 个可见图层
- **AND** 前端调用 `update_layer` 将该图层 `visible` 置为 `false`
- **THEN** 命令 SHALL 返回错误
- **AND** 配置不被修改

#### Scenario: 渲染中删除非最后可见图层被放行

- **WHEN** overlay 正在运行
- **AND** 活跃 Profile 有 2 个可见图层
- **AND** 前端删除其中 1 个
- **THEN** 命令 SHALL 成功，overlay 继续渲染剩余图层

#### Scenario: 未渲染时删除最后一个图层被放行

- **WHEN** overlay 未运行
- **AND** 活跃 Profile 只有 1 个图层
- **AND** 前端删除该图层
- **THEN** 命令 SHALL 成功，Profile 进入合法的空图层状态

### Requirement: 无可渲染内容时禁止启动覆盖

`start_overlay` SHALL 在活跃 Profile 无可渲染内容（可见图层数为 0 且 `crosshair = None`）时返回错误，不启动 overlay。存在 legacy `crosshair` 的纯旧配置 SHALL 不受影响。

前端 SHALL 在活跃 Profile 无可渲染内容时禁用「开始覆盖」按钮并提示原因；后端校验为最终防线。

#### Scenario: 空图层状态下启动覆盖被拒绝

- **WHEN** 活跃 Profile 可见图层数为 0 且 `crosshair = None`
- **AND** 前端调用 `start_overlay`
- **THEN** 命令 SHALL 返回错误
- **AND** overlay 不启动

#### Scenario: 全部图层隐藏后启动覆盖被拒绝

- **WHEN** 活跃 Profile 有 2 个图层但全部 `visible = false`
- **AND** 前端调用 `start_overlay`
- **THEN** 命令 SHALL 返回错误（全部隐藏等价于无可渲染内容）

#### Scenario: legacy 配置正常启动

- **WHEN** 活跃 Profile 为纯旧配置（`crosshair = Some(...)`，`layers = []`）
- **AND** 前端调用 `start_overlay`
- **THEN** 命令 SHALL 正常执行（目标窗口等既有校验不变）

### Requirement: 渲染中前端禁用最后可见图层的删除与隐藏

图层管理面板 SHALL 感知 overlay 活动状态：overlay 运行中，最后一个可见图层的删除按钮与可见性切换按钮 MUST 禁用并提示原因。面板各图层操作（增删/排序/复制/显隐/锁定）的失败路径 MUST 捕获 rejection（invoke 包装负责 toast），SHALL NOT 产生 unhandled rejection。

#### Scenario: 渲染中最后可见图层操作按钮禁用

- **WHEN** overlay 正在运行且活跃 Profile 仅剩 1 个可见图层
- **THEN** 该图层的删除按钮与隐藏（眼睛）按钮 SHALL 处于禁用状态
- **AND** 悬浮提示说明禁用原因

#### Scenario: 图层操作失败不产生 unhandled rejection

- **WHEN** 任一图层操作 IPC 返回错误
- **THEN** 前端 SHALL toast 显示错误信息
- **AND** 不触发 `unhandledrejection` 事件（不产生 PGR-3003 上报）

### Requirement: 空锚点状态下编辑器可达且可恢复

活跃 Profile 无 crosshair 且无图层时，配置编辑器 SHALL NOT 将该状态判定为「配置格式异常」：

- 多图层模式 SHALL 正常渲染图层编辑器（空图层列表 + 「添加图层」入口），不拦截。
- 单图层模式 SHALL 显示空态提示（非错误文案）与「切换到图层编辑器」入口，该入口 MUST 实际生效（切换后离开提示页）。
- 「开始覆盖」按既有需求禁用（无法遮盖，而非报错死循环）。

#### Scenario: 删光图层后停留在图层编辑器

- **WHEN** 多图层模式下用户删除最后一个图层
- **THEN** 图层编辑器 SHALL 保持可用（空态列表 + 添加入口）
- **AND** SHALL NOT 出现「配置格式异常」提示页

#### Scenario: 单图层模式空态可恢复

- **WHEN** 活跃 Profile 无 crosshair 且无图层，且当前为单图层模式
- **THEN** 页面显示空态提示与切换入口
- **AND** 点击切换入口后进入图层编辑器（不再回到提示页）

## MODIFIED Requirements

### Requirement: 图层操作通过 Tauri commands 暴露

系统 MUST 提供以下 Tauri commands 支持前端管理图层：
- `add_layer(material_id, name)` — 在当前 Profile 末尾添加图层
- `remove_layer(layer_id)` — 删除指定图层；overlay 活动中删除最后一个可见图层时返回错误（见「渲染不变量」需求）
- `move_layer(layer_id, new_index)` — 调整图层顺序
- `duplicate_layer(layer_id)` — 复制图层（生成新 id）
- `update_layer(layer_id, patch)` — 批量更新图层字段（params / style / transform / visible / name 等）；overlay 活动中将最后一个可见图层置为不可见时返回错误（见「渲染不变量」需求）
- `list_layers()` — 返回当前 Profile 的所有图层

所有图层操作 MUST 即时持久化到配置文件，并广播 `peregrine:layers-changed` 事件让前端同步刷新。校验失败的命令 MUST NOT 修改配置、SHALL NOT 广播变更事件。

#### Scenario: 添加图层

- **WHEN** 前端调用 `add_layer("builtin.cross", "中心十字")`
- **THEN** 当前 Profile 的 `layers` 末尾 SHALL 新增一个图层
- **AND** 图层 `params` 取物料的 `defaults()`
- **AND** 配置文件立即更新
- **AND** 前端通过 `peregrine:layers-changed` 事件收到新图层列表

#### Scenario: 移动图层顺序

- **WHEN** 当前图层顺序为 `[A, B, C]`
- **AND** 前端调用 `move_layer("A", 2)`
- **THEN** 新顺序为 `[B, C, A]`
- **AND** overlay 渲染顺序立即更新

#### Scenario: 校验失败不产生副作用

- **WHEN** `remove_layer` / `update_layer` 因渲染不变量校验返回错误
- **THEN** 配置文件与共享快照 SHALL 保持不变
- **AND** SHALL NOT 广播 `peregrine:layers-changed` 事件
