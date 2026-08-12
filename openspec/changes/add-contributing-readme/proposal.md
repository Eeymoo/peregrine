> **跟踪 issue：#49**（https://github.com/Eeymoo/peregrine/issues/49）

## Why

仓库根目录的 `CONTRIBUTING.md` 在 `main` 上不存在（GitHub 页面 404），而 GitHub 会在贡献者新建 issue / PR 时自动提示阅读该文件，缺失会削弱引导效果。同时勘察发现文档站贡献指南（`docs/guide/contributing.md` 及其 zh-cn 镜像）与仓库现行约定存在冲突：文档站称「commit body 与代码注释用英文（项目走向国际化）」，而 `AGENTS.md` 明确规定「代码注释、文档、commit message body 一律使用简体中文」，实际 git 历史也以中文为主。补充根目录文件的同时应一并裁定并修正该冲突，避免新文件把矛盾固化。

## What Changes

- 新建根目录 `CONTRIBUTING.md`：英文**瘦指针**文件，仅含欢迎语、完整贡献指南链接（英文 / 简体中文文档站页面）、issue 模板与 Discussions 快捷入口；不复制文档站内容，避免双份维护漂移。
- 修正 `docs/guide/contributing.md`（英文页）：commit message body 语言约定由「英文」改为「简体中文」；代码文档注释语言约定由「英文」改为「简体中文」，与 `AGENTS.md` 对齐。
- 同步修正 `docs/zh-cn/guide/contributing.md`（中文页）中的对应表述。
- 在两份文档站页面补充澄清：中文约定仅针对代码注释 / 文档 / commit body；Issue 与 PR 描述使用英文仍然欢迎（面向国际审阅者）。

## 目标

- 根目录出现一份自包含的英文 `CONTRIBUTING.md`，GitHub 提 issue / PR 场景可正常引导贡献者。
- 贡献指南的「注释 / commit body 语言」约定在全仓库范围内统一为简体中文（以 `AGENTS.md` 为准），消除文档间矛盾。
- 根目录文件保持瘦指针形态，完整指南仍单一维护在文档站。

## 非目标

- 不在根目录 `CONTRIBUTING.md` 中复制文档站的完整贡献流程（分支命名、commit 规范、PR 流程等）。
- 不处理「分支前缀 `feat/` vs `feature/`」的文档冲突（挂为遗留问题，另行裁定）。
- 不新建中文版根目录 `CONTRIBUTING.zh-cn.md`（中文读者经链接跳转文档站 zh-cn 页面）。
- 不改动任何代码、CI 工作流或 issue 模板。

## Capabilities

### New Capabilities

- `contributor-guide`: 贡献者引导文档的存在性与内容约定——根目录 `CONTRIBUTING.md` 作为指向文档站完整指南的瘦指针；贡献指南中的语言约定（代码注释 / 文档 / commit message body 使用简体中文）与 `AGENTS.md` 保持一致。

### Modified Capabilities

<!-- 无：本次不涉及既有 spec 级行为变更 -->

## Impact

- **文档**：新增 `CONTRIBUTING.md`（根目录）；修改 `docs/guide/contributing.md`、`docs/zh-cn/guide/contributing.md`。
- **代码 / API / 依赖**：无影响。
- **CI**：`pages.yml` 文档站部署流程在下次稳定版 Release 或手动触发时自动携带修正后的页面，无需额外操作。
