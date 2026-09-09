> **跟踪 issue：#81**（https://github.com/Eeymoo/peregrine/issues/81）

> **跟踪 issue：#81**（https://github.com/Eeymoo/peregrine/issues/81）

## Why

当前 `/opsx:propose` 只把 issue 号写进 `.openspec.yaml`，工件留在本地工作区（往往还在别人的特性分支上未提交）。GitHub issue 页面看不到任何分支 / 工件链接，无法直接查看提案内容；后续 `/opsx:apply` 还要自己判断基点，容易把新 change 的工件混进当前未合并分支。

## What Changes

- `/opsx:propose` 新增收尾步骤（三个镜像同步：`.opencode/commands/opsx-propose.md`、`.agents/workflows/opsx-propose.md`、`.agents/skills/openspec-propose/SKILL.md`）：
  1. 创建 issue 后，基于 `origin/main` 创建 `feature/<name>` 分支（当前工作区不在 main 时用 `git worktree add`，不打扰未提交工作）
  2. 提交全部变更工件并推送
  3. 编辑 issue 正文追加「分支 + 逐工件链接」区块，并留一条说明评论
- `/opsx:apply` 镜像微调：检测到 `.openspec.yaml` 已有 `branch:` 且分支已存在于远端时，直接检出该分支继续，省去重复建分支判断

## 目标

- 提案创建完成时，GitHub issue 页即可直接点开分支与全部工件
- 工件从诞生起就落在专用分支上，不污染执行 propose 时所在的分支
- `/opsx:apply` 能无缝接手 propose 已建的分支

## 非目标

- 不改变 apply 的"分支先行"原则与 PR 后开规则，只消除重复建分支
- 不改动 `openspec` CLI 本身
- 不涉及 archive 流程

## Capabilities

### New Capabilities

- `propose-branch-push`: propose 阶段的分支创建 / 工件推送 / issue 互链行为规范

### Modified Capabilities

（无——apply 接手机制作为 `propose-branch-push` 的新增需求一并提供）

## Impact

- `.opencode/commands/opsx-propose.md`、`.agents/workflows/opsx-propose.md`、`.agents/skills/openspec-propose/SKILL.md`（三镜像同步）
- `.opencode/commands/opsx-apply.md`、`.agents/workflows/opsx-apply.md`、`.agents/skills/openspec-apply/SKILL.md`（接手逻辑微调）
- 纯工作流模板变更，不改应用代码
