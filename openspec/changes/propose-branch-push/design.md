## Context

`/opsx:propose` 目前的收尾只做两件事：把 `issue:` / `branch:` 写入 `.openspec.yaml`、在 proposal 顶部加跟踪引用。工件留在本地。三个镜像（`.opencode/commands/opsx-propose.md`、`.agents/workflows/opsx-propose.md`、`.agents/skills/openspec-propose/SKILL.md`，各 160+ 行，内容等价）必须同步修改；apply 的技能名是 `openspec-apply-change`。

仓库现状约束：执行 propose 时工作区常在别的特性分支上且可能有未提交内容；`gh` 需要 `safe.directory`（本机 `.gitconfig` 只读，用 `GIT_CONFIG_COUNT/KEY/VALUE` 环境变量注入）。既有先例：本次 `docs-settings-guide` 已手动演练过 worktree 建分支 + 推送 + issue 互链的完整路径，可行性已验证。

## Goals / Non-Goals

**Goals:**

- propose 收尾：建分支（基于 `origin/main`）→ 提交工件 → 推送 → issue 正文补分支与工件链接
- apply 能接手已存在的远端分支
- 三镜像 + propose/apply 两侧同步一致

**Non-Goals:**

- 不改 openspec CLI、不动 archive
- 不改变 apply 的 PR 后开规则

## Decisions

### D1 用 `git worktree add` 而非直接 checkout

执行 propose 的会话往往停在别的分支（可能有未提交工作）。直接 `checkout main` 再建分支会扰乱现场。统一用：

```bash
git fetch origin main
git worktree add /tmp/opencode/<name> -b feature/<name> origin/main
cp -r openspec/changes/<name>/ /tmp/opencode/<name>/openspec/changes/
# 在 worktree 内 commit + push -u
```

若当前恰好在 main 且工作区干净，允许直接 checkout -b（等价路径，worktree 是保守默认）。
备选（否）：直接 checkout——会打断当前分支现场。

### D2 基点固定为 `origin/main`，不询问

propose 阶段无实现代码依赖，几乎不存在"必须基于某特性分支"的场景。固定 `origin/main` 消除一次交互；确有依赖时 apply 阶段仍可由人工重定基。备选（否）：每次询问基点——把决策成本前移到了信息最少的时刻。

### D3 issue 互链 = 正文追加区块 + 一条评论

正文追加「分支链接 + 逐工件（proposal/design/specs/tasks）blob 链接」区块（一次 `gh issue edit`），再 `gh issue comment` 一条说明（后续 apply 在此分支实施）。正文区块保证链接长期可见，评论提供时间线锚点。

### D4 apply 接手：优先检出既有远端分支

apply 开头的建分支步骤前插一步：`.openspec.yaml` 有 `branch:` 且 `git ls-remote --heads origin <branch>` 非空 → 直接检出（worktree 或 checkout）继续；否则走原建分支逻辑（含基点询问，保持现状）。向后兼容：老 change 无 `branch:` 或分支不存在时行为不变。

### D5 传播顺序：propose 三镜像 → apply 三镜像，同 PR 内完成

一个 change 同时覆盖两侧六个文件，避免中间态（propose 已推分支但 apply 不认识）长期存在。

## Risks / Trade-offs

- [propose 阶段推送产生"空 PR 前"远端分支] → 这是预期行为；分支即提案载体，apply 直接续用，PR 仍在 apply 收尾才开
- [worktree 目录残留] → 模板中明确 `git worktree remove` 清理时机（推送后即清，apply 时重新检出）
- [三镜像漂移] → tasks 中把"三文件逐节 diff 一致"列为独立验收项
- [fetch/push 需要网络与凭证] → `gh` 不可用时沿用现有降级：跳过推送，`.openspec.yaml` 照写 `branch:`，issue 互链步骤整体跳过并警告

## Migration Plan

模板合入后，既有未归档 change（无远端分支者）不受影响——apply 走原路径（D4 兼容分支）。

## Open Questions

无。
