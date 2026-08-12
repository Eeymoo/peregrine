# openspec-github-integration Delta Spec

> 新增能力：定义 Peregrine 仓库 OpenSpec 工作流（`/opsx:propose` / `/opsx:apply` / `/opsx:archive`）与 GitHub 的端到端集成契约——每个 change 由一个 GitHub issue（跟踪）+ 一个 GitHub PR（实现）串联，状态通过 change 的 `.openspec.yaml` 元数据（`issue` / `branch` / `pr`）流转；change 的实现分支基于 `main`；归档在 PR 已 merged 后才允许，且归档后本地工作区切回默认分支并拉取最新。

## ADDED Requirements

### Requirement: `/opsx:propose` 创建跟踪 issue 并回写元数据

`/opsx:propose` 在生成全部 artifact（proposal/design/tasks/specs）后，MUST 通过 `gh issue create --label feature,openspec` 创建**恰好一个** GitHub 跟踪 issue，issue body MUST 摘要 proposal 的动机/目标/非目标/影响并引用 change 目录路径；创建后 MUST 把 `issue: <number>` 与 `branch: feature/<change-name>` 写入 change 的 `.openspec.yaml`，并在 `proposal.md` 顶部追加 `> 跟踪 issue：#<number>` 引用。

#### Scenario: 正常创建 issue

- **WHEN** `/opsx:propose` 完成全部 artifact 且 `gh` 可用
- **THEN** 仓库新增恰好一个 issue，labels 为 `feature,openspec`，body 含动机/目标/非目标/影响 + change 路径
- **AND** change 的 `.openspec.yaml` 含 `issue:` 与 `branch:` 两个新键
- **AND** `proposal.md` 顶部含 `> 跟踪 issue：#<n>` 引用

#### Scenario: gh 不可用时降级

- **WHEN** `gh auth status` 失败或 `gh` 不存在
- **THEN** propose 流程不阻塞，跳过 issue 创建并在输出中 warn
- **AND** change 的 `.openspec.yaml` 不含 `issue:` 键

#### Scenario: 幂等——不重复创建 issue

- **WHEN** change 的 `.openspec.yaml` 已存在 `issue:` 键时再次运行 `/opsx:propose`
- **THEN** 不创建新 issue，报告已有 issue 编号

### Requirement: `/opsx:apply` 强制基于 main 建分支

`/opsx:apply` 在任何代码改动之前 MUST 创建工作分支，且该分支 MUST 基于 `main`（仓库默认分支为 `master` 时基于 `master`）。当当前不在 `main`/`master` 时，workflow MUST 自动尝试 `git checkout main`（或 `master`）+ `git pull --ff-only` 后再建分支；仅在工作区有未提交变更或 `main` 与 origin 分叉时才暂停询问用户。禁止在 `main`/`master` 上直接提交实现代码，禁止将 change 的实现分支堆叠在另一个 feature 分支之上（除非切回 main 物理上不可能且用户显式确认）。

#### Scenario: 已在 main 直接建分支

- **WHEN** `git branch --show-current` 返回 `main` 且目标分支不存在
- **THEN** 执行 `git checkout -b feature/<change-name>`，工作区不脏时不询问用户

#### Scenario: 不在 main 自动切回

- **WHEN** 当前分支为某个 feature 分支，工作区干净（`git status --porcelain` 为空），`main` 可 fast-forward
- **THEN** 自动执行 `git checkout main && git pull --ff-only && git checkout -b feature/<change-name>`，不询问用户

#### Scenario: 工作区脏时暂停

- **WHEN** `git status --porcelain` 输出非空（有未提交变更）
- **THEN** 不执行任何 checkout，暂停并询问用户如何处理（commit/stash/abort）

#### Scenario: main 分叉时暂停

- **WHEN** `git pull --ff-only` 失败（本地 main 与 origin/main 分叉）
- **THEN** 不 force pull，暂停并询问用户是否 reset main 到 origin/main

#### Scenario: 复用已存在的分支

- **WHEN** 目标分支已存在且为当前分支
- **THEN** 不重建分支，直接在该分支上继续（适用 `/opsx:apply` 重入）

### Requirement: `/opsx:apply` 全部任务完成后开 PR 并回写编号

`/opsx:apply` 在 `tasks.md` 所有任务标记为 `[x]` 后，MUST 通过 `gh pr create --base <default-branch> --head feature/<change-name>` 创建**恰好一个** PR，PR body MUST 引用跟踪 issue（`Closes #<issue-number>`），PR title MUST 使用 conventional commit 前缀；创建后 MUST 把 `pr: <number>` 写入 change 的 `.openspec.yaml`，并把该元数据更新 commit + push 到同一分支。PR 创建前 MUST 运行本地 checks（`cargo fmt --check` / `cargo clippy` / `cargo test` / `npm run build`），任一失败时不得开 PR。

#### Scenario: 正常创建 PR

- **WHEN** tasks.md 全部 `[x]`，本地 checks 通过，`gh` 可用，`.openspec.yaml` 无 `pr:` 键
- **THEN** 仓库新增恰好一个 PR，base 为默认分支，head 为 `feature/<change-name>`，body 含 `Closes #<issue>`
- **AND** change 的 `.openspec.yaml` 含 `pr: <number>` 键
- **AND** 该元数据更新已 commit 并 push

#### Scenario: 本地 check 失败时不开 PR

- **WHEN** 任一本地 check（fmt/clippy/test/build）失败
- **THEN** 不执行 `gh pr create`，暂停并报告失败项

#### Scenario: 幂等——不重复开 PR

- **WHEN** change 的 `.openspec.yaml` 已存在 `pr:` 键时再次运行 `/opsx:apply`
- **THEN** 不创建新 PR，报告已有 PR 编号 + 当前 merge 状态

#### Scenario: gh 不可用时降级

- **WHEN** `gh` 不可用
- **THEN** apply 主流程不阻塞，跳过 PR 创建并在输出中 warn，`.openspec.yaml` 不含 `pr:` 键

### Requirement: `/opsx:archive` PR 合并硬门禁

`/opsx:archive` 在移动 change 目录到 `archive/` 之前 MUST 读取 change 的 `.openspec.yaml` 的 `pr:` 键，并通过 `gh pr view <number> --json state,merged` 校验；当 PR 的 `merged` 字段为 `false`（state 为 `OPEN` 或 `CLOSED`）时 MUST 阻塞归档（输出 `## Archive Blocked`），不移动 change 目录，不执行后续步骤。此门禁不可跳过、不可被用户确认绕过（除无 `pr:` 键的历史 change 外）。

#### Scenario: PR 已合并通过门禁

- **WHEN** `.openspec.yaml` 的 `pr: <n>` 对应 PR 的 `merged: true`
- **THEN** 门禁通过，继续后续归档步骤

#### Scenario: PR 未合并阻塞

- **WHEN** `.openspec.yaml` 的 `pr: <n>` 对应 PR 的 `merged: false`（state OPEN 或 CLOSED）
- **THEN** 输出 `## Archive Blocked`，change 目录不移动，流程停止，提示用户先合并 PR 再重跑

#### Scenario: 无 pr 键的历史 change 放行

- **WHEN** `.openspec.yaml` 不含 `pr:` 键
- **THEN** 询问用户是否确认归档无 PR 记录的 change；用户确认则放行并在最终摘要带 warning，用户拒绝则停止

### Requirement: `/opsx:archive` 归档后切回默认分支

`/opsx:archive` 在成功移动 change 目录到 `archive/YYYY-MM-DD-<name>/` 之后 MUST 执行 `git checkout main`（失败时回退 `git checkout master`）+ `git pull --ff-only`，使本地工作区停在默认分支且为最新状态。`--ff-only` 失败时不 force，暂停询问用户。

#### Scenario: 归档后切回 main

- **WHEN** change 目录已成功移动到 archive/
- **THEN** `git branch --show-current` 返回 `main`（或 `master`），且 `git pull --ff-only` 成功

#### Scenario: pull 失败时不 force

- **WHEN** 归档后 `git pull --ff-only` 失败（本地默认分支分叉）
- **THEN** 不 force pull，暂停并询问用户

### Requirement: change 元数据通过 .openspec.yaml 流转

change 的 `.openspec.yaml` MUST 支持三个可选自定义键：`issue: <number>`（由 propose 写入）、`branch: <string>`（由 propose 写入）、`pr: <number>`（由 apply 写入）。这些键 MUST 不被 OpenSpec CLI 解析或拒绝（CLI 仅识别 `schema`/`created`/`status`/`note`），它们是 workflow 私有状态，随 change 目录一起移动（archive 时一并进入 `archive/`）。

#### Scenario: 三阶段读写契约

- **WHEN** propose 执行时
- **THEN** `.openspec.yaml` 新增 `issue:` 与 `branch:` 键
- **WHEN** apply 执行时
- **THEN** `.openspec.yaml` 读取 `issue:`/`branch:`，新增 `pr:` 键（不覆盖前两者）
- **WHEN** archive 执行时
- **THEN** `.openspec.yaml` 读取 `pr:`（校验合并状态），不写入新键，文件随目录移动到 archive/

#### Scenario: CLI 向后兼容

- **WHEN** 含 `issue:`/`branch:`/`pr:` 键的 `.openspec.yaml` 被 OpenSpec CLI 读取（`openspec status` / `openspec archive` 等）
- **THEN** CLI 不报错，不拒绝该 change，自定义键被原样保留

### Requirement: 结构化 GitHub 模板

仓库 MUST 提供 `.github/ISSUE_TEMPLATE/feature_proposal.yml`（OpenSpec 提案 issue 模板，含动机/目标/非目标/影响/建议 change 名字段，labels 为 `feature,openspec`）与 `.github/PULL_REQUEST_TEMPLATE.md`（PR 模板，含关联 OpenSpec change/Issue 字段、变更类型多选、自检清单、可选截图）。

#### Scenario: issue 模板字段

- **WHEN** 在 GitHub issue 选择器选择 "✨ 功能提案 / Feature Proposal"
- **THEN** 表单含 motivation / goal / nongoal / impact / change_name 字段，且创建的 issue 自动带 `feature` 与 `openspec` 标签

#### Scenario: PR 模板字段

- **WHEN** 在 GitHub 上为本仓库新建 PR
- **THEN** PR body 默认含关联 OpenSpec change、Issue 编号、变更类型多选框、自检清单（tasks 全勾 / schema 同步 / i18n 补齐 / cargo+npm checks / serde 向后兼容 / Report Code 登记）

### Requirement: 三镜像 workflow 定义同步

`/opsx:propose` / `/opsx:apply` / `/opsx:archive` 的工作流定义 MUST 同时存在于三个位置并保持一致：`.opencode/commands/opsx-*.md`（opencode slash command，`/opsx-*` 语法）、`.agents/workflows/opsx-*.md`（agent workflow，`/opsx:*` 语法）、`.agents/skills/openspec-*/SKILL.md`（skill 自动触发）。commands 与 workflows 在 `/opsx-*` ↔ `/opsx:*` 归一后 MUST byte-identical（frontmatter 除外）；skills 正文与 workflows 一致，额外含 frontmatter（`name` / `allowed-tools` 含 `Bash(gh:*)` 与 `Bash(git:*)` / `version`）。

#### Scenario: commands 与 workflows 归一一致

- **WHEN** 对任一 workflow 名（propose/apply/archive）执行 `diff <(sed -E 's@/opsx-([a-z]+)@/opsx:\1@g' .opencode/commands/opsx-<name>.md) .agents/workflows/opsx-<name>.md`
- **THEN** 无输出（byte-identical）

#### Scenario: skills 含 gh 与 git 工具权限

- **WHEN** 读取 `.agents/skills/openspec-propose/SKILL.md` / `openspec-apply-change/SKILL.md` / `openspec-archive-change/SKILL.md` 的 frontmatter
- **THEN** `allowed-tools` 含 `Bash(gh:*)`（propose/apply/archive 均含）与 `Bash(git:*)`（apply/archive 含）

### Requirement: openspec/config.yaml 注入 operations guidance

`openspec/config.yaml` MUST 含 `operations.apply.guidance` 与 `operations.archive.guidance` 两个数组，分别注入 apply 阶段的分支规则 / PR 回写约定与 archive 阶段的合并门禁 / 切回 main 约定。这些 guidance 会被 `openspec instructions apply --change <name> --json` 与 `openspec instructions archive --change <name> --json` 作为 `operationGuidance` 字段返回，供 agent workflow 消费。

#### Scenario: apply guidance 注入

- **WHEN** 执行 `openspec instructions apply --change <name> --json`
- **THEN** 返回 JSON 含 `operationGuidance` 字段，且其内容包含"创建专属工作分支"与"通过 gh 创建 PR 并回写 pr 编号"两条约定

#### Scenario: archive guidance 注入

- **WHEN** 执行 `openspec instructions archive --change <name> --json`
- **THEN** 返回 JSON 含 `operationGuidance` 字段，且其内容包含"PR 必须 merged"与"归档后切回默认分支"两条约定

### Requirement: AGENTS.md 文档化端到端生命周期

`AGENTS.md` 的 OpenSpec 段 MUST 含"End-to-end OpenSpec lifecycle (propose → apply → archive) with GitHub integration"小节，覆盖：`.openspec.yaml` 元数据键（`issue`/`branch`/`pr`）、三阶段（propose 建 issue、apply 建分支+开 PR、archive 门禁+切回 main）的规则、三镜像同步约定、模板文件清单（`.github/ISSUE_TEMPLATE/feature_proposal.yml` + `.github/PULL_REQUEST_TEMPLATE.md`）。

#### Scenario: AGENTS.md 覆盖三阶段

- **WHEN** 读取 `AGENTS.md` 的 OpenSpec Workflow 段
- **THEN** 含 `/opsx:propose` 创建跟踪 issue、`/opsx:apply` 基于 main 建分支 + 开 PR、`/opsx:archive` PR 合并硬门禁 + 切回 main 的明确描述
- **AND** 含 `.openspec.yaml` 元数据键清单（`issue` / `branch` / `pr`）与三镜像同步约定
