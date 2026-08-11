## ADDED Requirements

### Requirement: 全局对话框层与编辑模式解耦

`ConfigApp` SHALL 把全局对话框组件（`AutoSwitchDialog` / `UpdateDialog` / `UpdateProgress`）挂载在与编辑模式（单图层/多图层）无关的「全局对话框层」。这些组件 MUST 在 `layersMode` 为 true 和 false 时都能正常渲染与响应状态变化。

具体实现要求：全局对话框组件 MUST 作为 ConfigApp 最外层 `<ErrorBoundary>` 内、`layersMode return` 之前的兄弟节点挂载，而不是嵌套在单图层模式的 return JSX 内。

#### Scenario: 多图层模式下点击启动后弹出 AutoSwitchDialog

- **WHEN** 用户在多图层编辑器（layersMode=true）模式下点击「开始覆盖」按钮，`auto_switch_on_overlay` 设置为 `"ask"`
- **THEN** overlay 启动
- **AND** AutoSwitchDialog 对话框正常弹出（询问「是否隐藏配置窗口并切换到游戏？」）
- **AND** 用户点击「切换到游戏」后配置窗口销毁并聚焦目标窗口

#### Scenario: 多图层模式下发现新版本时弹出 UpdateDialog

- **WHEN** 应用检测到新版本可用，用户当前处于多图层编辑器模式
- **THEN** UpdateDialog 对话框正常弹出
- **AND** 用户可以选择立即更新或稍后

#### Scenario: 多图层模式下更新下载时显示 UpdateProgress

- **WHEN** 用户触发更新下载，处于多图层编辑器模式
- **THEN** UpdateProgress 进度条正常显示
- **AND** 进度更新实时可见

### Requirement: 单图层模式 opacity 展示为百分比

单图层模式（`ConfigApp.tsx` 右上角设置面板）的 crosshair.opacity 数值展示 MUST 以百分比形式呈现。存储值 0-1 在 UI 显示为 0%-100%。

#### Scenario: opacity=0.5 显示为 50%

- **WHEN** crosshair.opacity = 0.5
- **THEN** 右上角 opacity 标签旁的数值显示为 `50%`（而非 `0.50`）

#### Scenario: opacity=1.0 显示为 100%

- **WHEN** crosshair.opacity = 1.0
- **THEN** 数值显示为 `100%`

### Requirement: 图层操作失败时用户可见错误提示

`ConfigApp` 的所有图层操作（通过 `LayerPanel` / `LayerEditors` / `MaterialParamControls` 等组件触发）失败时，系统 MUST 通过 toast 向用户显示错误消息。错误处理 MUST 由 `api.ts` 的统一 `invoke` 包装负责，调用方不需要重复样板代码。

#### Scenario: 修改图层 opacity 到非法值时显示错误

- **WHEN** 用户拖动 opacity 滑块产生瞬时非法值（如 1.0000001），`updateLayer` 命令返回错误
- **THEN** 屏幕右上角弹出红色 toast 显示「layer style opacity must be in [0.0, 1.0]」
- **AND** toast 在 10 秒后自动消失（或用户点击关闭）

#### Scenario: 添加图层失败时显示错误

- **WHEN** 用户在添加图层对话框选择物料并确认，但 `addLayer` 因物料不存在失败
- **THEN** toast 显示错误消息
- **AND** 添加对话框保持打开供用户重试
