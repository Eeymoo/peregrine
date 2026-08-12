## Context

仓库现状（已在探索阶段勘察确认）：

- 根目录 `CONTRIBUTING.md` 在 `main` 上不存在（`git ls-tree origin/main` 无此文件，GitHub 页面 404）。
- `README.md` 第 12 行已将贡献者引导至文档站 `https://peregrine.aukcraft.org/guide/contributing.html`。
- 文档站已有完整贡献指南：`docs/guide/contributing.md`（英文，131 行）与 `docs/zh-cn/guide/contributing.md`（中文镜像），覆盖分支命名、Conventional Commits、开发流程、代码风格、测试要求、PR 流程（Squash & Merge）、Issue 要素。
- `.github/ISSUE_TEMPLATE/` 已有 4 个模板（`bug_report` / `feature_proposal` / `question` / `translation-improvement`），`config.yml` 含文档站与 Discussions 的 contact links。
- **冲突**：`docs/guide/contributing.md` L68 称 commit body 用英文、L93 称代码注释用英文；`AGENTS.md` 规定注释 / 文档 / commit body 一律简体中文；git 历史实际以中文为主。

约束：所有产出物用简体中文撰写；本次为纯文档变更，不触碰代码。

## Goals / Non-Goals

**Goals:**

- 根目录新增英文瘦指针 `CONTRIBUTING.md`，GitHub 新建 issue / PR 时可正常引导。
- 文档站贡献指南的语言约定修正为与 `AGENTS.md` 一致（简体中文），消除三处真相源之间的矛盾。
- 完整指南维持单一维护点（文档站），根目录文件零复制、零漂移风险。

**Non-Goals:**

- 不复制完整贡献流程到根目录文件。
- 不裁定分支前缀 `feat/` vs `feature/` 的冲突（遗留问题）。
- 不改动代码、CI、issue 模板。

## Decisions

### 决策 1：根目录文件采用「瘦指针」而非「精华版」或「全量镜像」

- **选择**：瘦指针——欢迎语 + 完整指南链接（英 / 中）+ issue 模板 / Discussions 快捷入口。
- **备选**：
  - 精华版（自包含核心流程 + 链接）：内容与文档站部分重叠，仍有漂移面；用户已明确选瘦指针。
  - 全量镜像：两处维护同一内容，必然漂移，直接排除。
- **理由**：完整指南已在文档站双语维护；README 已指向文档站；瘦指针把根目录文件的维护成本降为零，且满足 GitHub 的展示约定。

### 决策 2：根目录文件用英文，不建中文副本

- **选择**：单一英文文件，文件内给出 zh-cn 文档站链接。
- **备选**：仿 README 的 `CONTRIBUTING.zh-cn.md` 双文件——增加一个几乎不会被单独维护的文件；中文读者经链接跳转即可。
- **理由**：GitHub 场景面向国际贡献者，英文是默认语言；中文完整指南在文档站已存在。

### 决策 3：语言约定冲突以 `AGENTS.md`（简体中文）为准

- **选择**：修正文档站两页——commit message body 与代码文档注释（`///`、`//!`）使用简体中文。
- **备选**：以文档站（英文）为准——需要改写 `AGENTS.md` 并逆转既有中文注释存量，成本与影响面大得多。
- **理由**：`AGENTS.md` 是仓库现行约定且与 git 历史实践一致；文档站的相关表述是早期「国际化」设想的遗留，未落地。
- **配套澄清**：在两份页面补充说明——中文约定仅针对代码注释 / 文档 / commit body；Issue 与 PR 描述用英文仍然欢迎，避免误伤非中文贡献者。

### 决策 4：根目录文件内容结构

```markdown
# Contributing to Peregrine

<一句欢迎语>

完整贡献指南（分支命名、commit 规范、开发流程、代码风格、测试要求、PR 流程）
在文档站维护：
- English: https://peregrine.aukcraft.org/guide/contributing.html
- 简体中文: https://peregrine.aukcraft.org/zh-cn/guide/contributing.html

快捷入口：
- Bug 报告 / 功能建议 → issue 模板（issues/new/choose）
- 翻译改进 → translation-improvement 模板
- 问题与讨论 → GitHub Discussions
```

链接全部使用文档站线上地址（与 README 第 12 行一致），不链仓库内相对路径，保证在 GitHub 以外场景（如克隆后本地阅读）也可达。

## Risks / Trade-offs

- [根目录瘦指针过于简略，贡献者可能跳过阅读完整指南] → 快捷入口直接列出 issue 模板与 Discussions，即使不点指南链接也能完成最低成本的正确动作。
- [文档站地址变更会导致链接失效] → 链接与 README 第 12 行同源，地址变更时需一并更新；该域名（`peregrine.aukcraft.org`）为项目自有自定义域，变更概率低。
- [修正文档站语言约定可能与未来真正的国际化方向冲突] → 本次仅对齐现状并在 spec 中固化；未来若转向英文注释，属新的 spec 级变更，需另立 change。

## Open Questions

- 分支前缀 `feat/`（文档站）vs `feature/`（OpenSpec 工作流实践）的统一：留待后续 change 裁定。
