## ADDED Requirements

### Requirement: 提供可重复执行的 i18n 审查 skill

仓库 MUST 在 `.agent/skills/i18n-audit/` 提供一个 i18n 审查 skill，供 AI 编码代理审查前端国际化覆盖情况。该 skill MUST 覆盖以下审查维度：

1. 硬编码 UI 文案：`src/` 下 JSX/TSX 中直接向用户展示、但未走 `t()` 的文本（需排除注释、日志、`console.*`、纯样式 className）。
2. 缺失 key：代码中 `t("...")` 引用但在 `src/i18n/locales/zh-CN.json` 或 `en.json` 中不存在的 key。
3. 双语对齐：`zh-CN.json` 与 `en.json` 扁平化后 key 集合不一致的条目。
4. 冗余 key：locale 文件存在但代码中无任何引用的 key（仅报告，不强制删除）。

skill 的审查结果 MUST 以分类清单输出，标注文件与行号，便于直接修复。

#### Scenario: 运行审查发现硬编码文案

- **WHEN** 代理按 skill 指引对 `src/` 执行审查
- **THEN** 输出所有疑似未国际化的用户可见文案，包含文件路径、行号与建议的 i18n key

#### Scenario: 运行审查发现缺失 key

- **WHEN** 代码中存在 `t("common.add")` 等引用而 locale 文件缺少对应条目
- **THEN** 审查结果列出缺失 key 及其引用位置，并给出双语待补文案清单

### Requirement: 补齐审查发现的缺失文案

依据 i18n 审查结果，代码中所有 `t()` 引用的 key MUST 在 `zh-CN.json` 与 `en.json` 中同时存在；审查发现的硬编码用户可见文案 MUST 迁移为 `t()` 调用并补充双语条目。

#### Scenario: 所有引用 key 均有文案

- **WHEN** 修复完成后重新执行 i18n 审查
- **THEN** 缺失 key 清单为空，双语 key 集合一致

#### Scenario: 界面双语显示正常

- **WHEN** 用户在设置中切换语言（zh-CN / en）
- **THEN** 本次补齐的文案在两种语言下均显示对应翻译，不出现原始 key 串
