## 1. propose 镜像修改（三文件同步）

- [ ] 1.1 在 `.agents/skills/openspec-propose/SKILL.md` 步骤 5（创建 issue）后新增步骤 6「建分支并推送工件」：`git fetch origin main` → `git worktree add /tmp/opencode/<name> -b feature/<name> origin/main`（当前在 main 且干净时允许直接 checkout -b）→ 复制工件 → commit + `push -u` → `git worktree remove` 清理；gh/网络不可用时整体跳过并警告
- [ ] 1.2 同文件步骤 6 内含 issue 互链：`gh issue edit` 正文追加「分支链接 + proposal/design/specs/tasks 逐工件 blob 链接」区块 + `gh issue comment` 一条后续实施说明
- [ ] 1.3 将 1.1–1.2 内容同步到 `.agents/workflows/opsx-propose.md` 与 `.opencode/commands/opsx-propose.md`

## 2. apply 镜像修改（三文件同步）

- [ ] 2.1 在 `.agents/skills/openspec-apply-change/SKILL.md` 建分支步骤前插入接手检查：`.openspec.yaml` 有 `branch:` 且 `git ls-remote --heads origin <branch>` 非空 → 检出该分支继续（worktree 或 checkout），跳过建分支与基点询问；否则走原逻辑不变
- [ ] 2.2 同步到 `.agents/workflows/opsx-apply.md` 与 `.opencode/commands/opsx-apply.md`

## 3. 一致性验收

- [ ] 3.1 propose 三镜像逐节 diff：新增步骤内容等价（仅语法包装差异）
- [ ] 3.2 apply 三镜像逐节 diff：接手检查内容等价
- [ ] 3.3 用一个真实新 change 走一遍 `/opsx:propose`，验证：issue 正文含分支与工件链接、远端分支存在、worktree 已清理
