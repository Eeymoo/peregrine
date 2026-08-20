## ADDED Requirements

### Requirement: propose 收尾建分支并推送工件

`/opsx:propose` 在跟踪 issue 创建完成后，SHALL 基于 `origin/main` 创建 `feature/<name>` 分支（当前工作区不在 main 或有未提交内容时 MUST 使用 `git worktree add`，不得扰乱现场）、提交全部变更工件并推送到远端。

#### Scenario: 在他人特性分支上执行 propose

- **WHEN** 执行 `/opsx:propose` 时当前分支不是 main 且工作区有未提交内容
- **THEN** 流程 MUST 通过 `git worktree add` 基于 `origin/main` 创建 `feature/<name>`，MUST NOT 切换或污染原分支工作区，工件提交推送后清理临时 worktree

#### Scenario: gh 不可用

- **WHEN** `gh` 未认证或网络不可用
- **THEN** 跳过分支推送与 issue 互链步骤并给出警告，工件保留在本地，`.openspec.yaml` 照常写入 `branch:`

### Requirement: issue 与分支工件互链

propose 收尾 SHALL 编辑跟踪 issue 正文，追加包含分支链接与全部工件（proposal / design / specs / tasks）的 blob 链接区块，并 SHALL 发一条评论说明后续 apply 将在该分支实施。

#### Scenario: issue 页直接查看提案

- **WHEN** propose 完成，用户打开 GitHub issue 页
- **THEN** 正文 MUST 含指向 `feature/<name>` 分支的链接和每个工件文件的可直接打开链接

### Requirement: apply 接手已存在的远端分支

`/opsx:apply` 在建分支步骤前 SHALL 检查 `.openspec.yaml` 的 `branch:` 对应分支是否已存在于远端；存在时 MUST 直接检出该分支继续实施，MUST NOT 重复创建或询问基点。分支不存在或无 `branch:` 键时，行为 MUST 与现状完全一致（含基点询问）。

#### Scenario: apply 接手 propose 推送的分支

- **WHEN** `.openspec.yaml` 含 `branch: feature/<name>` 且远端已存在该分支
- **THEN** apply MUST 检出该分支并直接进入任务实施，不重新建分支、不询问基点

#### Scenario: 老 change 无远端分支

- **WHEN** change 无 `branch:` 键或远端不存在对应分支
- **THEN** apply MUST 沿用既有建分支逻辑（含"基于当前分支还是 main"的询问），行为与本次变更前一致
