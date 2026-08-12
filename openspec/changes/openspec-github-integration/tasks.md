## GitHub 模板

- [x] 新建 `.github/ISSUE_TEMPLATE/feature_proposal.yml`（OpenSpec 提案 issue 模板，字段：动机/目标/非目标/影响/建议 change 名；labels `feature,openspec`）
- [x] 新建 `.github/PULL_REQUEST_TEMPLATE.md`（PR 模板，字段：关联 change/issue、变更类型多选、自检清单、可选截图；顶部注释提示合并后跑 `/opsx:archive`）

## `/opsx:propose` 三镜像

- [x] `.opencode/commands/opsx-propose.md`：在第 4 步（artifact 创建）后加第 5 步"创建跟踪 GitHub issue"（读 proposal.md → `gh issue create --label feature,openspec` → 解析 issue 编号 → Edit 写 `issue:`/`branch:` 到 `.openspec.yaml` → Edit 在 proposal.md 顶部加 `> 跟踪 issue：#<n>` 引用），原第 5 步顺延为第 6 步
- [x] `.agents/workflows/opsx-propose.md`：同步 commands 内容，仅把 `/opsx-*` 改为 `/opsx:*`
- [x] `.agents/skills/openspec-propose/SKILL.md`：同步 workflows 内容 + frontmatter 加 `Bash(gh:*)` 到 `allowed-tools`，version bump `1.1`

## `/opsx:apply` 三镜像

- [x] `.opencode/commands/opsx-apply.md`：第 1 步读 `.openspec.yaml` 的 `issue:`/`branch:`；**第 2 步重写为"基于 main 建分支"**（探测当前分支与工作区 → 目标分支已存在则复用 → 已在 main 直接建 → 不在 main 自动切回+pull+建 → 脏树/分叉才停下问用户）；加第 8 步"开 PR"（commit+push → 本地 checks → `gh pr create --base <default> --head feature/<name>` → 写 `pr:` → commit metadata）；第 9 步显示完成
- [x] `.agents/workflows/opsx-apply.md`：同步 commands 内容，`/opsx-*` → `/opsx:*`
- [x] `.agents/skills/openspec-apply-change/SKILL.md`：同步 workflows 内容 + frontmatter 加 `Bash(gh:*)` / `Bash(git:*)`，version bump `1.1`

## `/opsx:archive` 三镜像

- [x] `.opencode/commands/opsx-archive.md`：在原第 1 步后插入第 2 步"PR 合并硬门禁"（读 `.openspec.yaml` 的 `pr:` → `gh pr view <n> --json state,merged` → `merged: false` 输出 `## Archive Blocked` 并停手；无 `pr:` 询问用户）；原步骤顺延；加第 7 步"归档后切回 main"（`git checkout main || git checkout master` + `git pull --ff-only`，失败不 force）；第 8 步摘要含 PR 合并状态与切回结果
- [x] `.agents/workflows/opsx-archive.md`：同步 commands 内容，`/opsx-*` → `/opsx:*`
- [x] `.agents/skills/openspec-archive-change/SKILL.md`：同步 workflows 内容 + frontmatter 加 `Bash(gh:*)` / `Bash(git:*)`，version bump `1.1`

## openspec/config.yaml

- [x] 在 `rules` 段后新增 `operations.apply.guidance`（4 条：分支规则 / main 上直接建 / 非默认分支先确认 / PR 回写）
- [x] 新增 `operations.archive.guidance`（2 条：PR 合并硬门禁 / 归档后切回 main + pull）

## AGENTS.md

- [x] 把原"`/opsx-apply` branching rule"单段替换为"End-to-end OpenSpec lifecycle (propose → apply → archive) with GitHub integration"完整段：`.openspec.yaml` 元数据键（issue/branch/pr）+ 三阶段规则 + 三镜像同步约定 + 模板文件清单

## 验证

- [x] YAML 合法性：`node -e "require('js-yaml').load(require('fs').readFileSync('.github/ISSUE_TEMPLATE/feature_proposal.yml','utf8'))"`（feature_proposal.yml / openspec/config.yaml 两个文件；config.yml 属于 i18n change，不在此 change 范围）
- [x] 三镜像一致性：commands↔workflows 在 `/opsx-*`↔`/opsx:*` 归一后 byte-identical（对 propose/apply/archive 跑 diff，归一后无输出）
- [x] skills frontmatter 含 `Bash(gh:*)`（三个 skill 均含）与 `Bash(git:*)`（apply/archive 含）
- [x] `openspec status --change openspec-github-integration` 显示所有 artifact 为 done
- [x] `openspec validate openspec-github-integration`（若 CLI 支持）通过

## 归档前

- [x] 运行 `/opsx:apply openspec-github-integration` 落库全部改动（基于 main 建 `feature/openspec-github-integration` 分支）
- [ ] 本地 checks 通过（`cargo fmt --check` / `cargo clippy` / `cargo test` / `npm run build`）—— 注：本 change 不改 Rust/TS 源码，这些 checks 应全绿
- [ ] 开 PR（`gh pr create --base main --head feature/openspec-github-integration`，body `Closes #<issue>`），写回 `pr:` 到 `.openspec.yaml`
- [ ] PR 合并后运行 `/opsx:archive openspec-github-integration`（门禁校验 PR merged，归档，切回 main）
