# overlay-dynamic-rendering Specification

## Purpose

定义 overlay 事件循环在动态物料（`is_dynamic == true`）驱动下的持续重绘行为，以及物料热重载与 overlay 线程的联动。本规范约束动态性判定的缓存策略、ControlFlow 切换时机、物料热重载通知机制，并定义文本图元的字重（`font_weight`）支持与时间物料的加粗参数。

> 注：动态物料运行时当前处于软关闭状态（`MATERIAL_DYNAMIC_INPUT_ENABLED = false`，见 `material-dynamic-input` 规范），下列「持续重绘」相关需求在此期间不可达；代码路径已就位，待重新启用后生效。

## Requirements

### Requirement: 动态物料驱动的持续重绘

当 active profile 的任一**可见**图层引用的物料 `is_dynamic() == true` 时，overlay 事件循环 MUST 以帧间隔（约 60FPS）持续重绘；当所有可见图层均为静态物料时，overlay MUST 进入 `ControlFlow::Wait`，仅在配置变更、窗口事件或显式 Invalidate 时重绘。

动态性判定结果 MUST 缓存，且仅在以下时机重新计算：配置快照更新（`OverlayCommand::UpdateConfig`）、物料热重载（`OverlayCommand::RefreshMaterials`）、overlay 创建。

#### Scenario: 含时钟物料的 profile 持续刷新

- **WHEN** active profile 包含引用 `builtin.time` 的可见图层，overlay 正在运行
- **THEN** 时钟文本每秒自动更新，无需任何窗口交互

#### Scenario: 纯静态 profile 不空转

- **WHEN** active profile 所有可见图层均为静态物料（如 `builtin.cross`）
- **THEN** 事件循环进入 `ControlFlow::Wait`，无周期性重绘，CPU 占用与修复前一致

#### Scenario: 切换 profile 后动态性即时生效

- **WHEN** overlay 运行中从纯静态 profile 切换到含动态物料的 profile（或反之）
- **THEN** 重绘调度行为在下一次 `about_to_wait` 即切换，无需重启 overlay

#### Scenario: 隐藏图层不参与动态性判定

- **WHEN** 某动态物料图层的 `visible == false` 且其余图层均为静态
- **THEN** overlay 进入 `ControlFlow::Wait`，不因隐藏图层持续重绘

### Requirement: 物料热重载通知 overlay 线程

物料目录热重载（`peregrine:materials-changed`）重建 `MaterialRegistry` 后，src-tauri MUST 向 overlay 线程发送 `OverlayCommand::RefreshMaterials`；overlay 线程收到后 MUST 更新 registry 句柄、使动态性判定缓存失效并重绘一帧。

#### Scenario: 物料由静态改为动态后即时生效

- **WHEN** 用户编辑物料脚本将 `is_dynamic()` 从 `false` 改为 `true`，热重载完成
- **THEN** 引用该物料的 overlay 无需重启即开始持续重绘

### Requirement: 文本图元字重支持

`Element::Text` MUST 支持可选字重字段 `font_weight`（100–900 的百位整数倍，缺省等价于 400）。overlay 的 SVG 渲染后端 MUST 将 `font_weight` 输出为 `<text>` 元素的 `font-weight` 属性；前端 Canvas 预览 MUST 以相同字重渲染，保持 WYSIWYG。缺少 `font_weight` 字段的旧配置 MUST 正常反序列化。

#### Scenario: 加粗时钟渲染

- **WHEN** 物料 build 返回的 Text 图元含 `font_weight: 700`
- **THEN** overlay 与前端预览均以粗体渲染该文本

#### Scenario: 旧配置兼容

- **WHEN** 反序列化不含 `font_weight` 字段的 Text 图元 JSON
- **THEN** 解析成功且字重按常规（400）渲染

#### Scenario: 非法字重校验

- **WHEN** 物料 build 返回的 Text 图元 `font_weight` 为 150 或 1000 等非法值
- **THEN** 该图层物料求值失败（`ElementField` 错误），图层渲染被跳过，其余图层与 overlay 不受影响

### Requirement: 时间物料加粗参数

`builtin.time` 物料 MUST 在 `schema()` 中提供 `bold` 参数（widget: `toggle`，默认 `false`）；`bold == true` 时 build 输出的 Text 图元 `font_weight` MUST 为 700，否则为缺省。

#### Scenario: 用户开启加粗

- **WHEN** 用户在参数面板打开时间物料的「加粗」开关
- **THEN** overlay 与预览中的时钟文本变为粗体
