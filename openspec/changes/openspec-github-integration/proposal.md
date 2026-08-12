> **状态：待实施（Ready）**
> 三镜像 + 模板文件已就地起草，待 `/opsx-apply` 落库 + 验证。运行 `/opsx-apply openspec-github-integration` 启动。
> **跟踪 issue：#43**（https://github.com/Eeymoo/peregrine/issues/43）

## Why

当前仓库的 OpenSpec 工作流（`/opsx:propose` / `/opsx:apply` / `/opsx:archive`）只产出本地的 proposal/design/tasks/specs，与 GitHub 完全脱节：

- **没有跟踪入口**：一个 change 从提案到归档，仓库里找不到对应的 issue / PR，维护者只能去翻 `openspec/changes/` 目录。
- **没有合并门禁**：`/opsx:archive` 只检查 tasks.md 是否全勾，不校验代码是否真的进了主干。change 可以被"归档"但代码从未合并，归档目录与主干状态背离。
- **分支策略只在 AGENTS.md 里一句话**：`/opsx:apply` 没有强制建分支的步骤，agent 容易直接在 `main` 或某个无关 feature 分支上提交实现代码，污染主干或堆叠出无法独立合并的分支。
- **没有结构化模板**：手工开 issue / PR 时信息收集不统一（动机、目标、非目标、影响、自检清单、关联 change）。

参考 OpenSpec 官方 customization 文档（`operations.apply.guidance` / `operations.archive.guidance` + 自定义 rules），可以把这三点一次性补齐：让每个 change 由"一个 GitHub issue + 一个 GitHub PR"端到端串联，分支永远基于 `main`，归档前强制校验 PR 已 merged，归档后自动切回 `main` 拉取最新。

## 目标

- **`/opsx:propose` 创建跟踪 issue**：artifact 生成后，自动 `gh issue create --label feature,openspec`，body 摘要动机/目标/非目标/影响 + change 路径；把 `issue:` 和 `branch: feature/<name>` 写入 `.openspec.yaml`；在 `proposal.md` 顶部加 `> 跟踪 issue：#<n>` 引用。`gh` 不可用时降级（跳过 issue，不阻塞 propose）。
- **`/opsx:apply` 强制基于 `main` 建分支**：第一步永远从 `main`（或 `master`）切出 `feature/<name>`；当前不在 `main` 时**自动尝试切回 `main` 并 `git pull --ff-only`**，工作区脏或 `main` 分叉才停下询问；分支已存在则复用。保证分支可独立 push / 开 PR / 合并。
- **`/opsx:apply` 最后一步开 PR 并回写编号**：tasks 全勾且本地 checks 过绿后，`gh pr create --base main --head feature/<name>`，body `Closes #<issue>` 并继承 `PULL_REQUEST_TEMPLATE.md`；`pr: <number>` 写入 `.openspec.yaml` 并 commit+push；每个 change 只开一次 PR。
- **`/opsx:archive` 硬门禁 + 归档后切回 main**：归档前置门禁——读 `.openspec.yaml` 的 `pr:`，`gh pr view <n> --json state,merged`，`merged: false` 则 `## Archive Blocked` 停手（不可跳过）；无 `pr:` 的历史 change 须用户显式确认才放行。归档完成后 `git checkout main || git checkout master` + `git pull --ff-only`。
- **结构化模板**：新增 `.github/ISSUE_TEMPLATE/feature_proposal.yml`（OpenSpec 提案 issue，labels `feature,openspec`）与 `.github/PULL_REQUEST_TEMPLATE.md`（关联 change/issue、变更类型、自检清单、合archive 提示）。
- **`openspec/config.yaml` 注入 operations guidance**：用官方 `operations.apply.guidance` / `operations.archive.guidance` 把分支规则、PR 回写、归档门禁、切回 main 写成可被 `openspec instructions apply/archive` 消费的项目级约定。
- **AGENTS.md 文档化**：把单段分支规则扩成"End-to-end lifecycle"段，覆盖 `.openspec.yaml` 元数据键、三阶段规则、三镜像同步约定。
- **三镜像同步**：`/opsx-*` 的 commands（`.opencode/commands/`，本地）、workflows（`.agents/workflows/`，`/opsx:*` 语法）、skills（`.agents/skills/openspec-*/SKILL.md`，自动触发）三处定义保持 byte-identical（仅斜杠语法与 frontmatter 不同）。

## 非目标

- **不引入自定义 schema**：继续用内置 `spec-driven`，只通过 `operations.guidance` + 工作流 markdown 注入约定，不 fork schema。
- **不替换 OpenSpec CLI**：`openspec` 命令行为不变，所有 GitHub 集成由 slash-workflow 文档驱动 agent 执行 `gh` / `git`，不改 CLI。
- **不做 CI 层面的归档自动化**：归档门禁由 `/opsx:archive` workflow 在本地执行 `gh pr view` 校验，不在 CI 加 status check。
- **不强制 PR review 人数 / CI 状态作为门禁**：archive 门禁只看 `merged: true`；review/CI 由 PR 模板自检清单约定，不写进硬门禁。
- **不处理多 store / 跨仓库场景**：本 change 只覆盖单仓库（`origin` 指向 `Eeymoo/peregrine`）的 gh 集成，`--store` 透传逻辑保持原样。
- **不补 i18n**：issue/PR 模板与 workflow 文档以中文为主、技术术语保留英文，不进 `src/i18n/locales/`（这些不是应用 UI 文案）。

## What Changes

- **`.github/ISSUE_TEMPLATE/feature_proposal.yml`**（新增）：OpenSpec 提案 issue 模板，字段含动机 / 目标 / 非目标 / 影响范围 / 建议 change 名；labels `feature,openspec`。
- **`.github/PULL_REQUEST_TEMPLATE.md`**（新增）：PR 模板，含关联 OpenSpec change / Issue 字段、变更类型多选、自检清单（tasks 全勾、schema 同步、i18n 补齐、cargo/npm checks、serde 向后兼容、Report Code 登记）、可选截图。
- **`.opencode/commands/opsx-propose.md`**（新增/重写）：在原 5 步后加第 5 步"创建跟踪 GitHub issue"（`gh issue create` + 写 `issue:`/`branch:` 到 `.openspec.yaml` + 在 proposal.md 顶部加引用），第 6 步显示最终状态。
- **`.opencode/commands/opsx-apply.md`**（新增/重写）：第 1 步选 change（读 `.openspec.yaml` 的 issue/branch）；**第 2 步强制基于 `main` 建分支**（自动切回 main、脏树/分叉才询问）；第 3-7 步原 status/instructions/context/progress/implement；**第 8 步最后开 PR**（commit+push、checks、`gh pr create`、写 `pr:`、commit metadata）；第 9 步显示完成。
- **`.opencode/commands/opsx-archive.md`**（新增/重写）：第 1 步选 change；**第 2 步硬门禁**（读 `pr:`、`gh pr view --json state,merged`、未合并 `## Archive Blocked`）；第 3-5 步原 artifact/task/sync 检查；第 6 步归档移动；**第 7 步切回 main + pull**；第 8 步摘要。
- **`.agents/workflows/opsx-{propose,apply,archive}.md`**（重写）：与 commands 三镜像同步，仅斜杠语法从 `/opsx-*` 改为 `/opsx:*`。
- **`.agents/skills/openspec-{propose,apply-change,archive-change}/SKILL.md`**（重写）：与 workflows 三镜像同步，frontmatter 加 `allowed-tools: Bash(gh:*)` / `Bash(git:*)`，version bump 到 `1.1`。
- **`openspec/config.yaml`**（修改）：在 `rules` 下新增 `operations.apply.guidance`（分支规则 + PR 回写）与 `operations.archive.guidance`（合并门禁 + 切回 main）。
- **`AGENTS.md`**（修改）：把原"`/opsx-apply` branching rule"单段替换为"End-to-end OpenSpec lifecycle (propose → apply → archive) with GitHub integration"完整段，含 `.openspec.yaml` 元数据键、三阶段规则、三镜像同步约定、模板文件清单。

## Capabilities

### New Capabilities

- `openspec-github-integration`：OpenSpec 工作流与 GitHub 双向集成——propose 建跟踪 issue、apply 建分支+开 PR、archive 校验 PR 已合并并切回 main，全程通过 `.openspec.yaml` 的 `issue`/`branch`/`pr` 元数据串联。

## Impact

- **流程层（无运行时代码）**：本 change 不改任何 Rust / TypeScript 源码，只改 `.github/`、`.agents/`、`.opencode/`、`openspec/config.yaml`、`AGENTS.md`。对应用二进制零影响。
- **agent 行为**：所有 opencode / openspec slash workflow 的行为变化——`/opsx:propose` 会多创建一个 issue，`/opsx:apply` 会建分支+开 PR，`/opsx:archive` 会校验 PR 合并状态。这是预期行为升级。
- **`.openspec.yaml` schema 扩展**：从 `schema/created/status/note` 扩展到可选 `issue/branch/pr`。OpenSpec CLI 不解析这些键（它们是 workflow 私有），向后兼容。
- **GitHub 仓库**：会新增 `feature` / `openspec` 两个 label（首次 propose 时由 `gh label create --force` 创建）；issue 选择器多一个"✨ 功能提案 / Feature Proposal"入口；PR 默认带模板。
- **三镜像维护成本**：今后修改任一 `/opsx-*` 流程必须同时改三处（commands / workflows / skills）。AGENTS.md 已明示此约束。
- **无新依赖**：`gh` 与 `git` 是已有前置条件；不引入 npm / cargo 包。
