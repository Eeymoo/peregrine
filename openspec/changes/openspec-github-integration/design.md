## Context

本 change 是**纯流程/文档层**改造，不改任何 Rust / TypeScript 源码，不涉及编译期或运行时架构。设计核心是"如何在不 fork OpenSpec schema、不改 OpenSpec CLI 的前提下，把 GitHub issue / PR / 分支策略 / 合并门禁串进 slash workflow"。

参考来源：OpenSpec 官方 [customization.md](https://raw.githubusercontent.com/Fission-AI/OpenSpec/refs/heads/main/docs/customization.md) 的 `operations.{apply,archive}.guidance` 段——`openspec instructions apply --change <name> --json` 与 `... archive ...` 会把 `operations.<phase>.guidance` 作为 `operationGuidance` 字段返回，agent workflow 读取并"遵循适用的、与内置步骤兼容的条目"。这意味着分支规则 / PR 回写 / 合并门禁可以同时通过两条路径注入：(1) 写进 `openspec/config.yaml` 的 `operations.guidance`（CLI 层、对所有 agent 生效），(2) 直接写进 slash workflow markdown（opencode 层、显式步骤化）。本 change 两条路径都用，互为冗余。

## Goals / Non-goals

（详见 `proposal.md`。设计层面补充：）

- **Goal**：让一个 change 从 propose 到 archive 的每一步都有可观测的 GitHub 产物（issue 编号、PR 编号、merge 状态），并保证 archive 时主干已包含该 change 的全部代码。
- **Non-goal**：不把 OpenSpec 改成"必须有 GitHub 才能用"——所有 gh 步骤都必须能在 `gh` 不可用时降级（跳过 issue / 跳过 PR，归档门禁对无 `pr:` 的历史 change 放行）。仓库在离线 / 私有 / 无 gh 环境下仍可跑完三阶段。

## Design

### 1. `.openspec.yaml` 作为状态串联的单一事实源

`.openspec.yaml` 是 OpenSpec CLI 生成的 change 元数据文件，CLI 不解析我们自定义的键，但它跟着 change 目录一起移动（archive 时一起进 `archive/`），是天然的"change 私有状态"载体。

扩展后的键集合：

```yaml
schema: spec-driven
created: 2026-08-12
status: active
note: |              # 可选，原有
  ...
issue: 42            # /opsx:propose 写入：跟踪 GitHub issue 编号
branch: feature/openspec-github-integration   # /opsx:propose 写入：预期工作分支名
pr: 17               # /opsx:apply 写入：实现 PR 编号
```

三阶段读写契约：

| 阶段 | 读 | 写 |
|---|---|---|
| propose | — | `issue`, `branch` |
| apply | `issue`, `branch` | `pr` |
| archive | `issue`, `pr` | —（随目录移动） |

**为什么不用 `note:`**：`note:` 是自由文本，机器解析不可靠；专用键让 `gh pr view $(yq '.pr' .openspec.yaml)` 这类命令可一行取值。

**为什么不用外部状态文件**（如 `.openspec.github.json`）：多一个文件 = 多一处不同步风险；`.openspec.yaml` 已是 change 的唯一元数据入口，扩展它最省心。

### 2. `/opsx:propose` —— issue 创建

**触发点**：artifact 全部 `done` 之后、显示最终状态之前。这样 issue body 可以引用已生成的 `proposal.md` 内容。

**issue body 结构**（中文为主，技术术语保留英文）：
```markdown
## 动机 / Why
<从 proposal.md 的 Why 段提取>

## 目标 / Goals
<从 proposal.md 的目标段提取>

## 非目标 / Non-goals
<从 proposal.md 的非目标段提取>

## 影响范围 / Impact
<从 proposal.md 的 Impact 段提取>

---
OpenSpec change: `openspec/changes/<name>/`
用 `/opsx:apply <name>` 开始实施。
```

**labels**：`feature,openspec`。首次 propose 时 `gh label create feature --force` / `gh label create openspec --force`（`--force` 幂等，已存在不报错）。

**降级**：`gh auth status` 失败 → 跳过 issue 创建，`.openspec.yaml` 不写 `issue:`，在摘要里 warn。不阻塞 propose 主流程。

**幂等**：`.openspec.yaml` 已有 `issue:` → 跳过创建，报告已有 issue。

**proposal.md 回写**：用 Edit 工具在现有状态 blockquote 之后插一行 `> **跟踪 issue：#<n>**（<url>）`，不重写整个文件。

### 3. `/opsx:apply` —— 分支策略（核心设计决策）

**规则**：change 的分支**永远基于 `main`**（或 `master`），不基于其他 feature 分支。理由：

| 分支基础 | 能否独立 push | 能否独立开 PR | 能否被 GitHub merge button 合并 | archive 门禁 |
|---|---|---|---|---|
| 基于 `main` | ✓ | ✓（`--base main`） | ✓ | ✓（merge 后 `pr.merged=true`） |
| 基于 feature/X | ✓ | ✓（`--base main`） | ✗（diff 含 X 的提交，需先合并 X） | ✗（依赖未合并的父分支） |

**流程**（替换原"问用户基于哪"的逻辑）：

```
1. git branch --show-current + git status --porcelain
2. 目标分支已存在且已 checkout → 复用，跳过
3. 目标分支已存在未 checkout → git checkout 复用，跳过
4. 已在 main → 直接 git checkout -b feature/<name>
5. 不在 main：
   a. 工作区脏（porcelain 非空）→ 停下问用户（commit/stash/abort），绝不丢弃
   b. 工作区干净 → git checkout main || git checkout master
   c. git pull --ff-only
      - 失败（main 分叉）→ 停下问用户（reset main 到 origin/main 或 abort），绝不 force
   d. git checkout -b feature/<name>
6. 宣告 "Created branch feature/<name> based on main"
```

**只在两种情况下停下问用户**：工作区脏、main 分叉。这两种都是"自动操作有破坏风险"，必须人工裁决。其余情况全自动，不再问"基于当前分支还是 main"——因为答案永远是 main。

**降级**：detached HEAD（CI 环境）/ 离线 → 才允许 fallback 到基于当前 HEAD，且必须用户显式确认。

### 4. `/opsx:apply` —— PR 创建

**触发点**：tasks.md 全 `[x]` 后。不在循环中途开 PR。

**前置 checks**：`cargo fmt --check` / `cargo clippy` / `cargo test` / `npm run build`（含 `tsc`）。任一失败 → 停下，不开 PR。这是"不开红 PR"约定。

**PR body**：只传 OpenSpec 关联摘要 + `Closes #<issue>`；自检清单 / 变更类型等由 GitHub 自动从 `PULL_REQUEST_TEMPLATE.md` 注入。`gh pr create --body` 与模板的关系：`--body` 内容会作为 PR body 的开头，模板字段在后面——所以 `--body` 只放关联段，不重复模板内容。

**base 分支**：默认 `main`；用 `git remote show origin | grep HEAD` 探测实际默认分支，若为 `master` 则用 `master`。

**回写 `pr:`**：Edit 工具在 `.openspec.yaml` 插 `pr: <number>` 行，不动 `issue:`/`branch:`。

**metadata commit**：`pr:` 写入后单独 commit（`chore(<name>): record PR #<n> in openspec metadata`）并 push，让 PR 的 diff 包含最终的 `.openspec.yaml`。

**幂等**：`.openspec.yaml` 已有 `pr:` → 跳过创建，报告已有 PR + 当前 merge 状态。

**降级**：`gh` 不可用 → 跳过 PR 创建，`.openspec.yaml` 不写 `pr:`，warn。用户可后续手工开 PR 并回填 `pr:`。

### 5. `/opsx:archive` —— 合并门禁（HARD GATE）

**为什么是硬门禁**：archive 把 change 目录移到 `archive/`，这是"该 change 已完成并落库"的信号。如果代码没合并就归档，归档目录与主干状态背离——后来者看 archive 以为功能已上线，实际主干没有。这是 OpenSpec 工作流最危险的隐性 bug。

**校验逻辑**：
```bash
gh pr view <pr-number> --json state,merged
# {"state": "CLOSED", "merged": true}  → 通过
# {"state": "OPEN",   "merged": false} → 阻塞
# {"state": "CLOSED", "merged": false} → 阻塞（被关闭未合并）
```

`merged: false` → 输出 `## Archive Blocked`，**不动 change 目录**，停下等用户去 GitHub 合并或关 PR 后重跑。

**无 `pr:` 的放行规则**：历史 change（本 change 落地前归档的）或 `gh` 在 apply 时不可用的 change，`.openspec.yaml` 没有 `pr:`。这类允许放行，但必须用户**显式确认**（AskUserQuestion），并在最终摘要里带 warning："archived without PR on record (legacy change)"。不静默放行。

### 6. `/opsx:archive` —— 归档后切回 main

```bash
git checkout main 2>/dev/null || git checkout master
git pull --ff-only
```

**为什么 `--ff-only`**：归档动作（`mv changeRoot archive/`）会产生工作区变更，通常会被 commit；切回 main 后若本地 main 落后于 origin，fast-forward 即可。若 main 本地分叉（不该发生，但防御），不 force，停下问用户。

**为什么必须切回**：下个 change 的 `/opsx:apply` 第 2 步会基于 main 建分支；如果归档后停在已合并的 feature 分支上，下次 apply 又要走"切回 main"流程。归档时顺手切好，下次直接进。

**feature 分支清理**：归档后**可选**提议 `git branch -d feature/<name>`（已合并才允许 `-d`），但必须用户确认，不自动删。

### 7. 三镜像同步策略

三个位置承载同一份 workflow：

| 位置 | 语法 | 触发 | 是否入库 |
|---|---|---|---|
| `.opencode/commands/opsx-*.md` | `/opsx-propose` | opencode slash command | ✗（`.opencode/` 被 `.gitignore` 排除，本地） |
| `.agents/workflows/opsx-*.md` | `/opsx:propose` | agent workflow（`/opsx:*`） | ✓ |
| `.agents/skills/openspec-*/SKILL.md` | `/opsx-propose`（正文） | skill 自动触发 | ✓ |

**同步规则**：commands 与 workflows 在 `/opsx-*` ↔ `/opsx:*` 归一后 byte-identical；skills 多一层 frontmatter（`name`/`allowed-tools`/`version`），正文与 workflows 一致（仅个别措辞如 "no clear input" vs "no input" 保留历史差异）。

**验证脚本**（人工跑，不入库）：
```bash
for name in propose apply archive; do
  diff <(sed -E 's@/opsx-([a-z]+)@/opsx:\1@g' ".opencode/commands/opsx-$name.md") ".agents/workflows/opsx-$name.md"
done
```
归一后应无输出。本 change 落地时已验证。

### 8. `openspec/config.yaml` operations guidance

用官方机制把约定下沉到 CLI 层（不依赖 agent 读 workflow markdown）：

```yaml
operations:
  apply:
    guidance:
      - 第一步必须创建专属工作分支（feature/<change-name>），禁止在 main/master 上直接提交实现代码
      - 在 main/master 上直接创建分支即可；若当前已在非 main/master 分支，须先与用户确认基于当前分支还是基于 main
      - 所有 tasks.md 任务完成后，最后一步必须通过 gh 创建 PR，并把 PR 编号写回 change 的 .openspec.yaml（pr: <number>）
      - PR 标题使用 conventional commits（feat/fix/refactor/docs/build），body 引用对应 issue（Closes #<n>）
  archive:
    guidance:
      - 归档前置硬门禁：change 对应的 PR 必须已 merged，未合并禁止归档（不可跳过）
      - 归档完成后自动切回默认分支（main 优先，回退 master）并 git pull --ff-only 拉取最新
```

**注**：本 change 的 apply 分支策略最终演进为"永远基于 main、自动切回、脏树/分叉才问"，比 config 里的 guidance 第 2 条更激进。config guidance 是 CLI 层的"保底约定"，workflow markdown 是 opencode 层的"完整步骤"——两者不冲突，workflow 更具体。后续可同步收紧 config 措辞，但不是本 change 的阻塞项。

## Risks

- **`gh` 不可用时的降级路径**：propose 跳过 issue、apply 跳过 PR、archive 对无 `pr:` 放行——三处都已设计降级，但意味着"无 gh 环境"下的 change 没有 GitHub 跟踪。可接受（这类环境通常是 CI 或离线 dev）。
- **`.openspec.yaml` 手工编辑风险**：用户手改 `pr:` 到错误编号 → archive 门禁查到 `merged: false` 报错。这是正确行为（防归档未合并的 change），用户需修正 `pr:` 或真正合并 PR。
- **三镜像漂移**：今后改 workflow 必须同步三处。AGENTS.md 已明示此约束，但没有自动化校验脚本（验证靠人工跑 diff）。风险可控——三处中 `.opencode/` 不入库，实际只需同步 workflows + skills 两处。
- **PR 模板与 `--body` 冲突**：`gh pr create --body "..."` 会覆盖模板还是拼接？实测 GitHub 行为是 `--body` 完全替代模板。所以 workflow 设计为 `--body` 只放关联段 + `Closes #<n>`，自检清单由模板承担——但 `--body` 替代意味着模板字段丢失。**当前实现已接受这个权衡**（关联段比自检清单更关键），后续可改成 `--body-file` 拼接模板 + 关联段。

## Open Questions

无。本 change 落地时所有设计决策已闭环。遗留的两个优化项（config guidance 措辞收紧、PR body 模板拼接）列为后续可选改进，不阻塞本 change。
