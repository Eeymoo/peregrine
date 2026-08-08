# material-dynamic-input 增量规格

## ADDED Requirements

### Requirement: 动态输入采集在运行链路中停止

主程序 SHALL NOT 为物料求值轮询动态上下文：`platform::poll_dynamic_context`（时间/鼠标位置/键盘状态）MUST NOT 在 overlay 事件循环或渲染路径中被调用；overlay 事件循环 MUST NOT 包含基于物料 `is_dynamic` 判定的定期唤醒/重绘调度，重绘 MUST 仅由配置变更通知、窗口跟随位置变化与系统窗口事件触发。动态输入 host function 的实现代码与测试 MUST 保留在 `peregrine_material` crate 中，供将来恢复时使用。

#### Scenario: 事件循环无动态调度

- **WHEN** overlay 窗口处于运行状态且配置未发生变化
- **THEN** 事件循环处于等待状态，不存在因动态物料判定产生的定时重绘，CPU 不出现周期性渲染开销

#### Scenario: 配置变更仍可即时重绘

- **WHEN** 用户在设置面板修改准星参数并保存
- **THEN** overlay 在配置热重载通知到达后重绘并显示新外观

#### Scenario: 动态输入代码保持可测试

- **WHEN** 执行 `cargo test -p peregrine_material`
- **THEN** 动态上下文（时间/鼠标/键盘/随机数）相关测试全部通过
