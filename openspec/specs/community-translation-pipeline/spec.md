# community-translation-pipeline Specification

## Purpose

定义 Peregrine 的社区翻译修正端到端闭环：通过 GitHub Issue 模板体系（bug 反馈 / 翻译改进 / 提问）结构化收集用户反馈；通过 opencode agent 自动 PR 工作流（`translation` 标签触发）将翻译改进 issue 自动转为 JSON 修改 PR；并通过针对 `src/i18n/locales/*.json` 的 CI 校验门（JSON 可解析、单语 key 集合不变、6 语 key 集合对齐）阻挡结构性错误；最终由维护者人工 review 合并，构建一条"用户报翻译问题 → 自动开 PR → CI 校验 → 维护者合并"的管线。

## Requirements

### Requirement: GitHub Issue 模板体系

仓库 MUST 在 `.github/ISSUE_TEMPLATE/` 提供四份文件：`bug_report.yml`（默认推荐模板）、`translation-improvement.yml`（翻译改进模板）、`question.yml`（兜底提问模板）、`config.yml`（选择器配置）。`config.yml` MUST 设置 `blank_issues_enabled: false`，强制所有 issue 走模板（替代任意空白 issue 入口）。

#### Scenario: 用户新建 issue 时看到模板选择器

- **WHEN** 用户在仓库点 "New issue"
- **THEN** 看到 bug 反馈、翻译改进、提问三个模板选项，且**没有** "Open a blank issue" 入口

#### Scenario: 翻译改进模板收集结构化字段

- **WHEN** 用户选择 "翻译改进" 模板并提交
- **THEN** issue body MUST 包含结构化字段：语言（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU 下拉）、i18n key、当前译文、建议译文、上下文截图

### Requirement: 翻译修正自动 PR 闭环

`.github/workflows/opencode.yml` MUST 新增 `auto-translate` job：当某 issue 被打上 `translation` 标签时，触发 opencode agent 解析 issue 表单字段（语言 / key / 当前译文 / 建议译文），自动编辑 `src/i18n/locales/<locale>.json`、开分支、提交、并以 "Closes #N" 引用原 issue 开 PR。`auto-label` job 的 prompt MUST 扩展规则：使用 `translation-improvement.yml` 模板提交的 issue 自动打 `translation` 标签。

#### Scenario: 用户提交翻译改进 issue 后自动开 PR

- **WHEN** 用户用 `translation-improvement.yml` 提交 issue 描述：语言=ja-JP、key=`tray.settings`、建议译文=`設定`
- **THEN** `auto-label` 打 `translation` 标签，`auto-translate` job 随后开 PR 修改 `src/i18n/locales/ja-JP.json` 的 `tray.settings` 字段为 `設定`，PR body 含 "Closes #N"

#### Scenario: 维护者保留最终 review 责任

- **WHEN** opencode 自动 PR 创建完成
- **THEN** PR 处于"等待 review"状态，维护者 MUST 手动 review 并 merge；MUST NOT 自动合并

### Requirement: 工作流权限声明

`auto-translate` job MUST 在 `permissions` 中显式声明 `contents: write`（用于推分支）和 `pull-requests: write`（用于开 PR）；MUST NOT 继承 workflow 级别的宽松权限。

#### Scenario: auto-translate job 能成功推分支与开 PR

- **WHEN** `auto-translate` job 触发执行
- **THEN** opencode agent 拥有充足权限推送 feature 分支与创建 PR，不因权限不足失败

### Requirement: 翻译 PR 的 CI 校验门

仓库 MUST 在 `.github/workflows/` 配置一个针对 `src/i18n/locales/*.json` 修改的 PR 触发的 CI 校验 job，对每份被修改的 locale JSON 执行三项检查：

1. **JSON 可解析**：被修改的 JSON MUST 能被 `serde_json::from_str` 成功反序列化。
2. **单语 key 集合不变**：PR diff 前后，被修改 locale JSON 的扁平化 key 集合 MUST 完全一致——只允许改 value，不允许加/删 key。
3. **6 语 key 集合对齐**：PR 合并前后，6 份 locale JSON（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）扁平化后的 key 集合 MUST 仍完全一致。

校验失败 MUST 阻塞 PR 合并。该 job 与人工 review 并行，不增加额外流程延迟。

#### Scenario: opencode 自动 PR 改坏 JSON 结构时被 CI 拦下

- **WHEN** opencode 自动生成的 PR 中 `ja-JP.json` 存在语法错误（如多余逗号、引号不闭合）
- **THEN** CI 校验 (a) 失败红灯，PR 无法 merge，维护者在 review 前就看到机械错误

#### Scenario: 翻译 PR 误删某 key 时被 CI 拦下

- **WHEN** 某个翻译修正 PR 中 `de-DE.json` 比 diff 前少了一个 key（opencode 误删或人为失误）
- **THEN** CI 校验 (b) 单语 key 集合不变 失败红灯，阻塞 merge

#### Scenario: 翻译 PR 只改了一门语言漏了其它 5 门时被 CI 拦下

- **WHEN** 某 PR 只修改 `ja-JP.json` 加了一个新 key，但 `zh-CN` / `en` / `de-DE` / `fr-FR` / `ru-RU` 未同步
- **THEN** CI 校验 (c) 6 语 key 集合对齐 失败红灯，阻塞 merge

#### Scenario: 仅修改译文 value 的合规 PR 顺利通过

- **WHEN** 某 PR 把 `ja-JP.json` 的 `tray.settings` value 从 `設定` 改为 `セットアップ`，未动任何 key
- **THEN** CI 校验三项全绿，PR 进入等待人工 review 状态
