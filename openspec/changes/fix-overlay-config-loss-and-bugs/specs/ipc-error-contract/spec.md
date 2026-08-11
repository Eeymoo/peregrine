## ADDED Requirements

### Requirement: IPC 结构化错误协议

所有图层与 Profile 相关的 Tauri 命令 SHALL 返回 `Result<T, IpcError>`，其中 `IpcError` 是结构化错误对象 `{ code: String, message: String }`，通过 `#[derive(serde::Serialize)]` 由 Tauri IPC 序列化给前端。`code` MUST 是稳定错误码（如 `VALIDATION` / `NOT_FOUND` / `INTERNAL`），`message` MUST 是人类可读的中文错误描述。

前端 `api.ts` SHALL 提供统一的 `invoke` 包装函数，捕获 IPC reject（`IpcError` 对象或遗留字符串），**包装为标准 `Error` 对象**（设置 `message` 与 `code` 属性），并通过 `showToast` 显示错误消息后重新抛出。

#### Scenario: 后端校验失败时前端显示 toast

- **WHEN** 用户拖动 opacity 滑块产生瞬时非法值（如 1.0000001），前端调用 `update_layer` 命令
- **THEN** 后端 `config.validate()` 失败，返回 `IpcError { code: "VALIDATION", message: "layer style opacity must be in [0.0, 1.0]" }`
- **AND** 前端 `invoke` 包装捕获错误，弹出 toast 显示 message 内容
- **AND** 调用方 catch 到的是标准 `Error` 对象（不再是字符串）
- **AND** Sentry/GlitchTip 不再报「Non-Error promise rejection」警告

#### Scenario: 图层不存在时返回 NOT_FOUND 错误码

- **WHEN** 调用 `update_layer` 传入已被删除的 layer_id
- **THEN** 后端返回 `IpcError { code: "NOT_FOUND", message: "layer 'layer-xxx' not found" }`
- **AND** 前端 toast 显示该消息

#### Scenario: 错误对象被包装为标准 Error

- **WHEN** Tauri IPC reject 一个 `IpcError` 对象 `{ code: "VALIDATION", message: "..." }`
- **THEN** 前端 `invoke` 包装创建 `new Error(message)`，并在对象上设置 `(err as any).code = "VALIDATION"`
- **AND** 重新 throw 该 Error 对象供调用方 catch

### Requirement: 图层操作错误统一捕获并提示

前端所有图层操作（add / remove / move / duplicate / update layer / 切换显隐 / 切换锁定 / 修改样式 / 修改变换 / 修改参数）SHALL 在调用 `invoke` 时使用 try/catch 捕获错误，**不再静默吞掉**。错误消息 MUST 通过 toast 显示给用户（复用 `globalErrorToast.ts` 的 `showToast`）。

调用方可选择在 catch 后执行额外的 UI 回滚（如恢复滑块位置），但 toast 显示由统一包装负责，不需要每个调用点重复样板代码。

#### Scenario: 修改图层样式失败时用户看到提示

- **WHEN** 用户拖动某图层的 opacity 滑块到非法值，`updateLayer` 调用失败
- **THEN** toast 显示「layer style opacity must be in [0.0, 1.0]」
- **AND** UI 滑块位置可在下次 `getConfig` 同步时恢复（或保持原位由用户调整）

#### Scenario: 添加图层失败时用户看到提示

- **WHEN** 用户点击「添加图层」选择某物料，但 `addLayer` 因物料不存在而失败
- **THEN** toast 显示错误消息
- **AND** 添加图层对话框保持打开（不关闭），用户可重试或取消

#### Scenario: 静默吞错的 console.error 模式被移除

- **WHEN** 代码扫描 `src/` 目录
- **THEN** 所有图层/profile 操作相关的 `.catch(console.error)` 模式被替换为统一 invoke 包装或显式 try/catch
- **AND** 仅保留 `.catch(() => {})` 用于真正可忽略的场景（如 `setTitle` 失败）
