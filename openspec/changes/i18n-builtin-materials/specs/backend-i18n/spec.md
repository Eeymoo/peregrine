# backend-i18n Specification (Delta)

## ADDED Requirements

### Requirement: 窗口标题走翻译表

后端创建 config / settings 窗口时，MUST 使用 `translate(locale, key)` 按创建时的 resolved locale 设置窗口标题，MUST NOT 硬编码任何语言的标题字符串。对应 locale key 为 `window.configTitle` 与 `window.settingsTitle`，两个 key MUST 在全部 6 份 locale JSON 中存在。前端挂载后基于 `t()` 的 `setTitle` 逻辑保留，作为语言切换后的权威更新路径。

config 窗口与 settings 窗口 MUST 一致地在创建时设置 `visible(false)`，由既有"就绪后显示"流程统一控制可见性，避免本地化标题生效前的窗口内容/标题闪烁。

#### Scenario: 日文界面下创建窗口标题为日文

- **WHEN** 配置 locale 解析为 ja-JP，后端创建 settings 窗口
- **THEN** 窗口初始标题为 `translate("ja-JP", "window.settingsTitle")` 的返回值
- **AND** 代码中不存在任何 `.title("Peregrine 设置")` 形式的硬编码中文字面量

#### Scenario: window.* key 6 语齐全

- **WHEN** 后端翻译表加载完成
- **THEN** `window.configTitle` 与 `window.settingsTitle` 在 6 门语言下均可由 `translate()` 命中，无需走 key 原样返回兜底
- **AND** 存在与 `translate_tray_keys_exist_in_all_locales` 同类的回归测试覆盖这两个 key

#### Scenario: config 窗口创建时不可见

- **WHEN** 后端创建 config 窗口
- **THEN** 窗口以 `visible(false)` 构建，与 settings 窗口行为一致
- **AND** 既有显示流程仍能正常使其可见（首启引导路径不回归）
