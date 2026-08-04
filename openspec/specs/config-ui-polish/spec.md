# config-ui-polish Specification

## Purpose

定义设置面板的 UI 修正规范：图层编辑器「变换」区块在物料运行时软关闭期间暂时隐藏（组件、i18n key 与数据保留），以及 ProfileManager 编辑态的布局约束（隐藏切换下拉框、输入框自适应，320px 宽面板内不溢出）。

## Requirements

### Requirement: 图层编辑器暂时隐藏「变换」区块

图层编辑器右侧参数面板 SHALL NOT 渲染「变换」区块（位移 / 缩放 / 旋转），直到物料运行时恢复。`LayerTransformEditor` 组件源码与 `layers.transformSection`、`layers.offsetX`、`layers.offsetY`、`layers.scale`、`layers.rotation` 等 i18n key MUST 保留，便于后续重新启用。

#### Scenario: 打开图层编辑器选中图层

- **WHEN** 用户在图层编辑器中选中任意图层
- **THEN** 右侧参数面板仅展示「参数」与「样式」区块，不出现「变换」区块及其标题

#### Scenario: 变换配置数据保留

- **WHEN** 配置中图层已存在 `transform` 数据
- **THEN** 隐藏「变换」区块后该数据不被修改、不被清除，预览与覆盖层渲染行为不变

### Requirement: ProfileManager 编辑态不渲染切换下拉框

ProfileManager 进入新建 / 重命名编辑态时，MUST 隐藏配置方案切换下拉框（Select），编辑态仅渲染「输入框 + 确认按钮 + 取消按钮」，输入框占据原下拉框位置，使整行控件在 320px 宽的设置面板内不溢出。退出编辑态后 MUST 恢复显示切换下拉框。

#### Scenario: 进入新建编辑态

- **WHEN** 用户点击「新建」按钮
- **THEN** 切换下拉框隐藏，同一行仅显示名称输入框、「添加」、「取消」按钮，且整行宽度不超出父容器

#### Scenario: 进入重命名编辑态

- **WHEN** 用户点击「重命名」按钮
- **THEN** 切换下拉框隐藏，输入框预填当前方案名，同一行仅显示输入框、「保存」、「取消」按钮，且整行宽度不超出父容器

#### Scenario: 退出编辑态恢复切换下拉框

- **WHEN** 用户完成确认、点击取消或按下 Escape
- **THEN** 编辑态关闭，切换下拉框恢复显示，方案列表与激活方案保持最新状态
