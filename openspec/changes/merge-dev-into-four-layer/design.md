# 设计：合并 dev 的 UI 拆分重构到四层架构分支

## Context

`feature/four-layer-customization` 从 `f0f70b3` 分出后，dev 合入了 5 个提交（核心为 v0.1.13-alpha.0）：

- **前端拆分重构**：`ConfigApp.tsx` 从 471 行单体缩为 258 行骨架，逻辑下沉到 `src/hooks/`（useConfigAppState / useConfigSave / useOverlayActions / useUpdate）与 `src/components/config/`（AutoSwitchDialog / UpdateDialog / UpdateProgress / TargetWindowSelect）；`SettingsApp.tsx` 从 821 行缩为 140 行，五个标签页拆为 `src/components/settings/*Tab.tsx`，状态收敛到 `src/hooks/useAppState.ts` / `useSettingsSync.ts`
- **单例模式**：`tauri-plugin-single-instance`，第二实例启动聚焦已有窗口（`src-tauri/src/lib.rs`，git 已自动合并成功）
- **Markdown 更新日志**：`MarkdownReleaseNotes.tsx` + `react-markdown` + `@tailwindcss/typography`
- **镜像下载修复**：安装包下载链接套用镜像前缀（`src-tauri/src/lib.rs`，自动合并成功）

本分支同期改动：`ConfigApp.tsx` +607 行（LayersEditor / ProfileManager / DeveloperPanel / 单双图层模式切换 / 图层颜色快捷键 / legacy 单图层兼容），`SettingsApp.tsx` 仅 +46 行（快捷颜色重置按钮、关于信息 i18n），`src/lib/api.ts` +96 行（图层 / profile IPC 封装）。

两边是**正交重构**：dev 改"代码结构"，本分支改"数据模型与功能"。合并本质是把功能移植进新结构。

## Goals / Non-Goals

**Goals:**

- `git merge origin/dev` 后工作区无冲突标记，8 个冲突文件全部解决
- dev 的四个功能（拆分架构 / 单例 / Markdown 更新日志 / 镜像下载）合并后可用
- 本分支四层架构功能（图层编辑、物料、多 profile、开发者面板）零回归
- `bash scripts/test.sh` 全绿；推送后 PR #24 CI 恢复触发并全绿

**Non-Goals:**

- 不做额外重构（不趁合并再拆本分支的组件、不改 hooks 命名风格）
- 不解决 OpenSpec 两个变更遗留的 Windows 实机验收任务
- 不发布版本、不打 tag

## Decisions

### 决策 1：用 merge 而非 rebase

保留两条线的提交历史，冲突一次性解决。rebase 会让 60+ 个提交逐个与 dev 的拆分冲突，代价远大于一次 merge。

### 决策 2：ConfigApp 以 dev 骨架为底，移植 layers 逻辑

不保留本分支的 1090 行单体 ConfigApp。以 dev 的 258 行骨架（hooks + 子组件）为基础，把本分支功能按 dev 的架构风格归位：

- `useConfigAppState`：不动（加载 config / overlayActive / version）
- `useConfigSave`：dev 版返回 `crosshair` / `updateCrosshair` 助手（基于旧 schema）。改为基于新 schema：返回 `profile` / `layers`，单图层 legacy 模式下通过 `Profile::is_legacy_compatible()` 对应的 TS 侧判断提供与旧 `crosshair` 助手等价的封装（`StyleFields` 仍按单图层参数渲染）
- `useOverlayActions`：不动（start/stop 与 schema 无关）
- `useUpdate`：不动
- 本分支独有能力作为新组合：单/多图层模式切换 state、`LayersEditor`（多图层模式整体替换单图层表单）、`ProfileManager`（顶部栏）、`DeveloperPanel`（版本号 3 击解锁）
- dev 的 4 个 config 子组件（AutoSwitchDialog / UpdateDialog / UpdateProgress / TargetWindowSelect）原样保留接入

备选方案（保留本分支单体、只挑 dev 的 hooks 进来）被否决：等于放弃 dev 重构成果，后续维护两份结构。

### 决策 3：SettingsApp 整体采用 dev 版，回补本分支 2 处小改动

本分支对 SettingsApp 的改动只有 46 行且语义独立，直接以 dev 的 140 行骨架为准：

- 快捷颜色重置按钮 → 移植到 `settings/OverlayTab.tsx` 的快捷颜色区块
- 关于信息 i18n（publisher / license / repository）→ 移植到 `settings/AboutTab.tsx`

### 决策 4：版本号一律取 0.2.0-alpha.0，依赖取并集

- `Cargo.toml` / `package.json` / `src-tauri/tauri.conf.json` 版本字段：`0.2.0-alpha.0`（本分支高于 dev 的 0.1.13-alpha.0，且 CI release 约定四层架构为 v0.2.0）
- `package.json` 依赖：并集（dev 的 `react-markdown` / `@tailwindcss/typography` + 本分支全部新增）
- `Cargo.lock` / `package-lock.json`：机械冲突，删 lock 重新 `cargo build` / `npm install` 生成，避免手工解 lock 冲突出错
- `src-tauri/Cargo.toml`：git 已自动合并（dev 加 `tauri-plugin-single-instance`），只需确认编译

### 决策 5：`src/lib/api.ts` 保留双方

dev 的 `getCurrentWebviewWindow` re-export（3 行）+ 本分支 96 行图层 / profile IPC 封装，无语义冲突。同时把 ConfigApp/SettingsApp 里直接从 `@tauri-apps/api/webviewWindow` import 的地方统一走 api.ts re-export（dev 已确立此约定）。

## Risks / Trade-offs

- [useConfigSave 改造成 layers 后，`StyleFields` 单图层控件参数映射出错] → 复用本分支已实现的 `is_legacy_compatible` TS 判断 + `getDefaultCrosshairForStyle` presets；合并后用 `cargo test` + `tsc` + 手工预览截图比对验证
- [dev 的 hooks 与本分支组件（LayersEditor / ProfileManager）状态不同步] → 所有 hooks 以 `useConfigAppState` 的单一 `config` state 为源；ProfileManager 切换 profile 后走既有 `set_active_profile` → 刷新 config 的链路（multi-profile-config 变更已实现）
- [单例插件与本分支启动链路降级（fix: 启动链路降级，避免 expect panic）交互异常] → 合并后 `cargo check --target x86_64-pc-windows-msvc` 验证编译；运行时行为留待 Windows 实机冒烟（沿用既有实机验收清单 docs/manual-test-checklist.md）
- [lock 文件重新生成拉进意外版本漂移] → `Cargo.lock` 用 `--locked` 之外的常规解析；npm 侧 `npm install` 后核对 `react-markdown` / `@tailwindcss/typography` 存在于 lock

## Migration Plan

1. 提交/暂存当前工作区未提交改动（ci.yml test-report job、scripts、格式化等）
2. `git merge origin/dev`，按决策 2/3/4/5 解 8 个冲突文件
3. 删 lock 重新生成；`bash scripts/test.sh` 全量验证
4. 推送，确认 PR #24 CI 恢复触发且全绿
5. 回滚策略：合并前打临时分支 `backup/pre-dev-merge`，失败直接切回
