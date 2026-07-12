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

### 奇偶版本号约定

- **正式版 = 奇数版本号**：0.1.1, 0.1.3, 0.1.5, 0.1.7 …
- **尝鲜版 = 偶数/预发布版本号**：0.1.8-alpha.0, 0.1.8-alpha.1 …

发布正式版时，从当前 dev 分支的最新偶数 patch +1 得到奇数版本号。

### 默认递增策略

**除非用户明确要求，只递增 patch（z 位）**，不要主动升级 major（x）或 minor（y）：

- 默认：`0.1.0` → `0.1.1`（升 patch）
- 用户明确说"加新功能""大版本"等才递增 minor：`0.1.0` → `0.2.0`
- 用户明确说"不兼容""破坏性变更"等才递增 major：`0.1.0` → `1.0.0`

推荐版本号时按此默认值给出，并在确认环节向用户说明当前递增的是哪一位，方便其纠正。

## 分支策略

- **main**：稳定分支，仅包含正式版代码。正式版发布后合并到 main。
- **dev**：开发分支，包含正在测试的功能。测试通过后合并到 main 发布正式版。
- 功能开发在 dev 上进行，main 保持与最新正式版一致。

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
2. **概述**：用一句话说明本版本的核心价值
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

• 采用 MIT 协议。

下载

• Windows x86 / x86_64 / ARM64 NSIS 安装包 + 便携 zip 见 Release Assets。
```

预发布版可简写，列出核心改动即可。

## 作者署名

每条功能点末尾标注实际贡献者，格式为 `@<github-username>`。可通过 `git log` 查看每个改动的作者：

```bash
git shortlog -sn <上个tag>..
```

## 发布产物

CI 构建完成后，每个架构产生以下产物：

| 产物 | 说明 |
| --- | --- |
| `peregrine-v*.exe`（NSIS 安装包） | 带签名，支持自动更新 |
| `*.sig` | 安装包签名文件 |
| `peregrine-v*-*.zip`（便携 zip） | 解压即用，不支持自动更新 |
| `latest.json` | Tauri updater 清单（版本号、签名、下载 URL） |

正式版（纯版本号 tag）和预发布版（带 `-` 后缀）产物格式相同。

## 自动更新

项目集成了 `tauri-plugin-updater`，安装版（NSIS）用户可通过「设置 → 检查更新」自动下载安装新版本。

- **签名密钥**：私钥存在本地 `.tauri/peregrine.key`（已被 `.gitignore` 排除），公钥写入 `tauri.conf.json`。
- **GitHub Secrets**：`TAURI_SIGNING_PRIVATE_KEY`（私钥内容）和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（密码）配置在 repo settings 中，CI 构建时用于签名。
- **更新清单**：`latest.json` 由 CI 自动生成，上传到 GitHub Release。
- **注意事项**：私钥和密码丢失后将无法发布可自动更新的版本，请妥善备份。便携 zip 用户无法自动更新，需手动下载替换。

## 发布前检查清单

打 tag 并推送前，必须完成以下步骤：

1. **代码格式化** — 在仓库根目录执行以下命令，确保 CI 的格式检查通过：

   ```bash
   cargo fmt --all
   cargo fmt --all -- --check
   ```

   若有未提交的格式化改动，需提交后再继续。

2. **版本号** — 确认版本号符不符合 SemVer（major/minor/patch 是否合理），正式版还是预发布。**除非用户明确说明，默认只递增 patch（z 位），不要主动升级 major（x）或 minor（y）**；确认时向用户说明本次递增的是哪一位。
3. **Tag 消息** — 展示完整的 tag 提交信息（即 Release body），让用户审阅。
4. **最后机会** — 推送 tag 会触发真实构建发布，等待用户明确回复"确认"或"推送"再执行。

```bash
# 创建正式版 tag
git tag -a v0.1.0 -m "Release message"

# 或创建预发布 tag
git tag -a v0.2.0-alpha.0 -m "Pre-release message"

# 推送到远程，触发 GitHub Actions 自动构建
# 注意：分支和 tag 同名时用 refs/tags/ 前缀避免歧义
git push origin refs/tags/v0.1.0
```

推送后自动构建 Windows x86 / x86_64 / ARM64 产物（NSIS 安装包 + 便携 zip + latest.json）并创建 GitHub Release。
