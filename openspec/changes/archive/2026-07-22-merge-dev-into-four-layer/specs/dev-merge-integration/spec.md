# dev 合并集成

## ADDED Requirements

### Requirement: 合并后代码库无冲突残留

`git merge origin/dev` 完成后，工作区 MUST NOT 存在任何冲突标记（`<<<<<<<` / `=======` / `>>>>>>>`），8 个冲突文件（`Cargo.toml`、`package.json`、`src-tauri/tauri.conf.json`、`src/lib/api.ts`、`Cargo.lock`、`package-lock.json`、`src/ConfigApp.tsx`、`src/SettingsApp.tsx`）MUST 全部解决并纳入提交。

#### Scenario: 冲突标记扫描

- **WHEN** 合并提交完成后在仓库根目录执行 `grep -rn "<<<<<<<" --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git`
- **THEN** 无任何匹配结果

#### Scenario: lock 文件一致性

- **WHEN** 删除并重新生成 `Cargo.lock` 与 `package-lock.json` 后执行 `cargo build` 与 `npm ci`
- **THEN** 两者均成功，且 `package-lock.json` 中包含 `react-markdown` 与 `@tailwindcss/typography`

### Requirement: 版本号统一为四层架构预发布版本

合并后 `Cargo.toml`（workspace.package.version）、`package.json`、`src-tauri/tauri.conf.json` 的版本字段 MUST 均为 `0.2.0-alpha.0`。

#### Scenario: 三处版本号一致

- **WHEN** 读取上述三个文件的版本字段
- **THEN** 三者均为 `0.2.0-alpha.0`

### Requirement: dev 的 v0.1.13 功能在合并后可用

合并后 MUST 保留 dev 引入的以下能力：单例模式（第二实例聚焦已有窗口）、Markdown 渲染的更新日志（`react-markdown` + `@tailwindcss/typography`）、中国大陆镜像前缀套用到安装包下载链接、更新检查 UI（AutoSwitchDialog / UpdateDialog / UpdateProgress / TargetWindowSelect 及 settings 五个 *Tab 组件）。

#### Scenario: 单例插件注册

- **WHEN** 检查 `src-tauri/src/lib.rs` 的 Tauri Builder 链
- **THEN** 存在 `tauri_plugin_single_instance::init` 注册，且回调聚焦已有窗口

#### Scenario: 镜像下载前缀

- **WHEN** 启用中国大陆镜像且安装包下载链接指向 github.com
- **THEN** `download_install_update` 将下载 URL 改写为镜像前缀 + 原始链接

#### Scenario: 设置窗口五标签页结构

- **WHEN** 检查 `src/SettingsApp.tsx`
- **THEN** 其渲染 `GeneralTab` / `OverlayTab` / `HotkeysTab` / `UpdateTab` / `AboutTab` 五个标签页组件，且通过 `src/hooks/` 管理状态

### Requirement: 四层架构功能零回归

合并后本分支的四层架构能力 MUST 全部保留：图层管理（LayersEditor）、多 profile 管理（ProfileManager）、开发者面板（DeveloperPanel）、单/多图层模式切换、图层颜色快捷键、`src/lib/api.ts` 中的图层与 profile IPC 封装。

#### Scenario: ConfigApp 集成本分支组件

- **WHEN** 检查合并后的 `src/ConfigApp.tsx`
- **THEN** 其包含 `LayersEditor`、`ProfileManager`、`DeveloperPanel` 的引用，且保留单/多图层模式切换能力

#### Scenario: api.ts 保留双方封装

- **WHEN** 检查 `src/lib/api.ts`
- **THEN** 同时存在 `getCurrentWebviewWindow` re-export 与图层 / profile IPC 封装函数（如 `listProfiles`、`setActiveProfile`、`listLayers` 等）

#### Scenario: 快捷颜色重置与关于信息 i18n 回补

- **WHEN** 检查合并后的 `OverlayTab` 与 `AboutTab`
- **THEN** `OverlayTab` 快捷颜色区块包含重置按钮，`AboutTab` 关于信息使用 i18n key（publisher / license / repository）

### Requirement: 合并后全量验证通过

合并提交前 MUST 通过本地全量验证：`cargo test`（3 crate）、`cargo clippy -D warnings`、`cargo fmt --check`、`cargo check --target x86_64-pc-windows-msvc`、`npx tsc --noEmit`、`npm run build`。推送后 PR #24 的 CI MUST 恢复触发且全绿。

#### Scenario: 本地测试套件

- **WHEN** 在合并结果上执行 `bash scripts/test.sh`
- **THEN** 全部 6 项检查通过

#### Scenario: PR CI 恢复

- **WHEN** 合并提交推送到 `feature/four-layer-customization`
- **THEN** PR #24 出现 CI 检查记录，且最终结论为 success
