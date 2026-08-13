# layer-composition 增量规格

## ADDED Requirements

### Requirement: 图层功能在渲染与设置界面暂停生效

`Profile.layers` 数据 MUST 继续被正常反序列化、校验与保留（不删除、不写回降级），但 SHALL NOT 参与 overlay 渲染与设置面板编辑：

- 渲染器 MUST 忽略 `layers`，强制走旧版 `crosshair` 路径；当 `crosshair` 缺失时 MUST 回退到 `Crosshair::default_crosshair()`，保证任何历史配置下 overlay 均有可见准星。
- 设置面板 MUST NOT 显示图层编辑器（LayersEditor）入口；`layersMode` 初始值 MUST 为 `false` 且所有切换到图层模式的按钮 MUST 被 `MATERIAL_RUNTIME_ENABLED` 常量门控隐藏。
- 当用户配置历史上含 `layers` 时，前端 MUST 沿用现有"从 layers[0] 反向生成 crosshair"的兜底逻辑显示可编辑的准星设置，不得报格式错误。

#### Scenario: 含 layers 的配置降级渲染

- **WHEN** 加载一份 `layers` 非空且 `crosshair` 为 null 的配置文件并启动 overlay
- **THEN** overlay 显示默认准星（或前端反向生成的旧准星），不渲染任何图层内容，不报错崩溃

#### Scenario: 纯旧配置行为不变

- **WHEN** 加载一份仅含旧 `crosshair` 字段的配置文件并启动 overlay
- **THEN** 渲染外观与软关闭前完全一致

#### Scenario: 图层数据不丢失

- **WHEN** 应用在软关闭状态下读取并保存配置
- **THEN** 配置文件中的 `layers` 数据原样保留，不被清除或覆盖为旧格式

#### Scenario: 设置面板无图层入口

- **WHEN** 用户打开设置面板
- **THEN** 界面中不存在任何进入图层编辑器的按钮或链接，仅显示旧版准星设置控件
