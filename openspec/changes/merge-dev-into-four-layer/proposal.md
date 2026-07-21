# 合并 dev 分支 UI 拆分重构到四层架构分支

## Why

`dev` 分支在 `feature/four-layer-customization` 分出之后合入了 v0.1.13-alpha.0 的前端组件拆分重构（ConfigApp / SettingsApp 拆分为 hooks + 子组件）、单例模式、Markdown 更新日志与镜像下载修复。PR #24 因此与 dev 产生 8 个文件的合并冲突，GitHub 跳过了该 PR 的全部 CI 检查，四层架构分支处于"CI 失明"状态。越早合并代价越小：dev 仍在演进，拖下去两个大改文件（ConfigApp.tsx / SettingsApp.tsx）会越差越远。

## What Changes

- 将 `origin/dev` 合并进 `feature/four-layer-customization`，解决全部 8 个冲突文件
- 琐碎冲突（机械解决）：
  - `Cargo.toml` / `package.json` / `src-tauri/tauri.conf.json`：版本号取 `0.2.0-alpha.0`（本分支），package.json 同时保留 dev 新增的 `react-markdown` / `@tailwindcss/typography` 依赖
  - `Cargo.lock` / `package-lock.json`：删除后重新生成
  - `src/lib/api.ts`：保留双方改动（dev 的 `getCurrentWebviewWindow` re-export + 本分支新增的图层/profile IPC 封装）
- 硬冲突（人工合并，本变更的核心）：
  - `src/ConfigApp.tsx`：**保留 dev 的拆分骨架**（hooks + 子组件架构），将本分支的 layers / 多 profile 逻辑移植进新骨架
  - `src/SettingsApp.tsx`：同上
- 合并后完整验证：本地测试套件（test/clippy/fmt/tsc/build）+ 推送后确认 PR #24 CI 恢复触发并全绿

## Capabilities

### New Capabilities

- `dev-merge-integration`: dev 分支功能（单例模式、Markdown 更新日志、镜像下载修复、前端 hooks 拆分架构）与四层架构（layers / 物料 / 多 profile）在同一代码库中共存且行为不回归

### Modified Capabilities

<!-- 无：openspec/specs/ 暂无已归档的主 spec，本合并不改变任何已归档需求 -->

## Impact

- **代码**：`src/ConfigApp.tsx`、`src/SettingsApp.tsx`（人工合并）；`Cargo.toml`、`package.json`、`src-tauri/tauri.conf.json`、`src/lib/api.ts`、`Cargo.lock`、`package-lock.json`（机械解决）；`src-tauri/src/lib.rs`（git 自动合并成功，含单例插件 + 镜像下载修复，需验证编译）
- **新增依赖**：`react-markdown`、`@tailwindcss/typography`（来自 dev）、`tauri-plugin-single-instance`（Rust 侧，来自 dev）
- **CI**：合并冲突消除后 PR #24 的 `pull_request` 工作流恢复触发；本地新增的 `test-report` / `frontend-report` job 随之首次在 CI 运行
- **行为约束**：dev 引入的 v0.1.13 功能（单例聚焦、Markdown 更新日志、镜像下载、更新检查 UI）在合并后必须可用；本分支的四层架构功能（图层编辑、物料、多 profile）不得回归
