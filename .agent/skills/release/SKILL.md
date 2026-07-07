---
name: release
description: Release 发布流程与规范。按标准格式打 tag、编写 Release Notes 并触发 GitHub Actions 自动构建发布。当需要发布新版本时使用。
---

## 版本号规则

遵循语义化版本（SemVer）：

| 版本位 | 何时递增 | 示例 |
| --- | --- | --- |
| **major** | 不兼容的 API / 重大架构变更 | `0.1.0` → `1.0.0` |
| **minor** | 新增功能（向下兼容） | `0.1.0` → `0.2.0` |
| **patch** | 修复 bug / 小改进（向下兼容） | `0.1.0` → `0.1.1` |

预发布版本在 patch 后加 `-alpha.N` 或 `-beta.N`，如 `0.2.0-alpha.0`。

## Release Notes 编写指南

### 确定功能点

回顾自上个 tag 以来的 git 提交历史，归类整理：

```bash
# 查看自上个版本以来的所有提交
git log --oneline <上个tag>..

# 只看合并提交（feature branch 合并）
git log --merges --oneline <上个tag>..
```

按以下类别整理：

| 类别 | 说明 | 对应 commit type |
| --- | --- | --- |
| **新增** | 新功能、新特性 | `feat` |
| **修复** | 修复的 bug | `fix` |
| **变更** | 重构、行为变化、依赖升级 | `refactor`、`chore` |
| **构建** | CI/CD、构建流程变化 | `ci` |

### 编写规范

1. **标题行**：`Peregrine v<版本号>`，如 `Peregrine v0.1.0`
2. **概述**：用一句话说明本版本的核心价值，例如"首个正式版本。一个用于缓解 3D 眩晕的桌面辅助贴图工具……"
3. **功能列表**：按编号列出每条改动，每行末尾标注作者 `@Eeymoo`
4. **许可与下载**：结尾固定段落

### 模板

正式版使用完整模板：

```
Peregrine v<版本号>

<版本概述>

1. <改动 1> @Eeymoo
2. <改动 2> @Eeymoo
3. <改动 3> @Eeymoo
...

许可

• 采用 PolyForm Noncommercial 1.0.0 协议。

下载

• Windows x86 / x86_64 / ARM64 可执行文件见 Release Assets。
```

预发布版可简写，列出核心改动即可。

## 作者署名

每条功能点末尾标注实际贡献者，格式为 `@<github-username>`。可通过 `git log` 查看每个改动的作者：

```bash
git shortlog -sn <上个tag>..
```

## 发布前检查清单

打 tag 并推送前，必须完成以下步骤：

1. **代码格式化** — 在仓库根目录执行以下命令，确保 CI 的格式检查通过：

   ```bash
   cargo fmt --all
   cargo fmt --all -- --check
   ```

   若有未提交的格式化改动，需提交后再继续。

2. **版本号** — 确认版本号符不符合 SemVer（major/minor/patch 是否合理），正式版还是预发布。
3. **Tag 消息** — 展示完整的 tag 提交信息（即 Release body），让用户审阅。
4. **最后机会** — 推送 tag 会触发真实构建发布，等待用户明确回复"确认"或"推送"再执行。

```bash
# 创建正式版 tag
git tag v0.1.0

# 或创建预发布 tag
git tag v0.2.0-alpha.0

# 推送到远程，触发 GitHub Actions 自动构建
git push origin v0.1.0
```

推送后自动构建 Windows x86 / x86_64 / ARM64 产物并创建 GitHub Release。
