# 任务清单：合并 dev 的 UI 拆分重构到四层架构分支

## 1. 合并前准备

- [x] 1.1 创建回滚备份分支 `backup/pre-dev-merge`（指向当前 HEAD，不推送）
- [x] 1.2 提交当前工作区未提交改动（ci.yml test-report / frontend-report job、`.config/nextest.toml`、`scripts/`、AGENTS.md、格式化改动），保证合并前工作区干净

## 2. 执行合并与机械冲突

- [x] 2.1 执行 `git merge origin/dev`，确认 8 个冲突文件清单与预期一致
- [x] 2.2 解 `Cargo.toml` / `package.json` / `src-tauri/tauri.conf.json`：版本取 `0.2.0-alpha.0`，package.json 依赖取并集（保留 `react-markdown`、`@tailwindcss/typography`）
- [x] 2.3 解 `src/lib/api.ts`：保留 `getCurrentWebviewWindow` re-export + 本分支全部图层 / profile IPC 封装
- [x] 2.4 检查 `src-tauri/src/lib.rs`（git 自动合并）：确认单例插件注册与本分支启动降级逻辑共存，无语义矛盾
- [x] 2.5 删除 `Cargo.lock` / `package-lock.json`，执行 `cargo build` 与 `npm install` 重新生成，核对 lock 中含 `react-markdown` / `@tailwindcss/typography`

## 3. ConfigApp.tsx 人工合并（dev 骨架 + layers 移植）

- [x] 3.1 以 dev 版 258 行骨架为底：保留 hooks（useConfigAppState / useOverlayActions / useUpdate）与 4 个 config 子组件接入
- [x] 3.2 改造 `useConfigSave` 适配新 schema：返回 `profile` / `layers`，单图层 legacy 模式通过 TS 侧 `isLegacyCompatible` 判断提供与旧 `crosshair` 助手等价的封装
- [x] 3.3 移植单/多图层模式切换 state 与切换按钮（预览层右上角）
- [x] 3.4 接入 `LayersEditor`（多图层模式替换单图层表单）、`ProfileManager`（顶部栏）、`DeveloperPanel`（版本号 3 击解锁）
- [x] 3.5 统一 `getCurrentWebviewWindow` 走 `src/lib/api.ts` re-export（dev 约定）

## 4. SettingsApp.tsx 人工合并（dev 版 + 2 处回补）

- [x] 4.1 整体采用 dev 版 140 行骨架（五 *Tab 组件 + hooks）
- [x] 4.2 快捷颜色重置按钮移植到 `settings/OverlayTab.tsx`
- [x] 4.3 关于信息 i18n（publisher / license / repository）移植到 `settings/AboutTab.tsx`

## 5. 合并后验证

- [x] 5.1 冲突标记扫描：`grep -rn "<<<<<<<" --exclude-dir=node_modules --exclude-dir=target` 无结果
- [x] 5.2 `bash scripts/test.sh` 全量通过（test / clippy / fmt / Windows MSVC check / tsc / build）
- [x] 5.3 人工走查合并后 ConfigApp / SettingsApp：单图层模式渲染、多图层模式渲染、profile 切换、更新对话框、五标签页设置窗口
- [ ] 5.4 提交合并（merge commit），推送后确认 PR #24 CI 恢复触发且全绿（含新 test-report job 的 JUnit 报告）

## 6. 收尾

- [ ] 6.1 更新 `docs/manual-test-checklist.md`：补充单例模式 / Markdown 更新日志 / 镜像下载三项的 Windows 实机验收项
- [ ] 6.2 确认无误后删除本地备份分支 `backup/pre-dev-merge`
