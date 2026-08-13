# Proposal：发版与文档同步固化

> 跟踪 issue：#58（https://github.com/Eeymoo/peregrine/issues/58）

> 探索产出：本文档由 explore 会话（docs 优化 + 发版同步）沉淀而成。

## Why

当前文档站与发版流程存在多处手动维护与失效点，探索中逐一定位：

1. **下载入口是坏的**：落地页 `DownloadCta.astro` 的三个按钮指向 `releases/latest/download/peregrine-windows-x64-setup.exe`，但 release.yml 实际产物名带版本号（`peregrine-v0.2.4-windows-x64-setup.exe`），`latest/download/` 需精确匹配 → **三个按钮全部 404**。
2. **FeatureGrid 图标缺失**：`FeatureGrid.astro:55` 的 svg 上有一个多余的 `set:html={''}`，把内部 `<Fragment set:html={ICONS[f.icon]}/>` 覆盖为空 → 落地页「Everything you need to stay oriented」下方 6 张卡片**图标全部渲染为空**（已在构建产物中确认）。
3. **没有独立下载页**：下载入口直接外链 GitHub Releases，中文字面无法给出「GitHub 加速」选项；也没有「查看更多版本」的站内承载。
4. **顶部无导航**：Starlight 没有原生 navbar-links，只有 site title + 社交图标，AukCraft 主站 / Docs / Download 三个入口无处安放。
5. **没有「在 GitHub 上编辑此页面」**：Starlight 原生 `editLink` 未启用，贡献者无法一键跳转源码。
6. **发版与 changelog 靠自觉**：「先合 changelog、后打 tag」目前只是 release skill 里的经验流程，没有任何 CI 强制；漏写 changelog 也能发版，文档落后一版。

## 目标

- 落地页三个下载按钮**恢复可用**（修复资产命名不匹配）。
- FeatureGrid 图标**恢复渲染**。
- 新增**独立下载页**（easytier 式），构建时拉取 GitHub API 动态取最新版本资产，下载页**零版本号硬编码**；中文 locale 提供「GitHub 加速」选项，en locale 仅直连。
- 顶部导航新增 **AukCraft ↗ / Docs / Download**（桌面顶栏 + 移动侧栏）。
- 启用 Starlight **`editLink`**（「在 GitHub 上编辑此页面」）。
- 用 **CI 强制**「先合 changelog、后打 tag」时序：
  - release.yml 增加前置闸门（tag 版本 ≠ changelog 最新条目 → fail，构建不启动）。
  - ci.yml 增加文档一致性 job（版本号 ↔ changelog，PR 阶段提前暴露）。

## 非目标

- **不做「Agent 编辑」按钮**：依赖 `opencode.yml` 工作流的 secret 与权限改动（`issues: opened` 事件、`contents: write`），属于独立 CI 改造，探索判定可行性存疑（`OPENCODE_API_KEY` 无法从仓库验证），本 change 不包含。仅做纯静态 `editLink`。
- **不改 release.yml 的资产命名**：下载页走构建时 API（路径 B），不动产物名与 updater 清单，避免影响存量 release 与 `latest.json`。
- **不做「发版后自动改文档 + 自动合 PR」的 workflow**：架构上 tag 触发的一切自动化都晚于文档部署，无法赶上本次发版；changelog 是创作性工作（叙事 / 作者 / issue 引用），AI 写 + 人审 + CI 强制时序是正解。
- **不引入 React island**：下载页交互（加速通道 / 版本筛选）用 vanilla `<script>`，符合 landing 组件「纯 Astro + scoped CSS」惯例。
- **不新增「Agent 编辑」、不改 ci.yml 的既有 job 语义**。

## What Changes

### 下载页（新）

- 新增独立页面（en `/download` + zh-cn `/zh-cn/download`），easytier 式结构：
  - 顶部简介 + GitHub Releases 总入口。
  - **切换下载通道**：稳定版 / 预发布版（仅当仓库存在对应 release 时渲染）。
  - **GitHub 加速 / 直连**（仅 zh-cn locale 渲染）：加速 = 请求经 gh-proxy 前缀；直连 = 原 URL。
  - **按硬件架构筛选**：x64 / x86 / ARM64。
  - 表格列：操作系统 / 硬件架构 / 安装包（NSIS exe）/ 便携版（zip）/ 注意事项。
  - 「查看更多版本」按钮 → 打开 GitHub Releases 页面。
- **数据源（决策 L2-B）**：构建时 GET `https://api.github.com/repos/Eeymoo/peregrine/releases`（或 `releases/latest`），从 `assets[].browser_download_url` 动态生成下载链接；构建失败时降级为「查看 Releases」外链，保证页面始终可渲染。
- 版本号 **不硬编码**：页面展示的版本号来自 API 返回，发版后 pages.yml 重建即自动更新。

### 顶部导航

- 桌面：覆写 Starlight `Header`（破例一次，记录到 AGENTS.md 的「仅覆写 Hero」约束处），在 site title 右侧插入文字链接 **AukCraft ↗ / Docs / Download**。
- 移动：sidebar 顶层增加对应 `link` 项（AukCraft 外链、Download 内链、Docs 内链），随移动端抽屉展示。
- AukCraft → `https://www.aukcraft.org/`（外链，`target="_blank"`）。
- Docs → `/guide/intro`（en）/ `/zh-cn/guide/intro`（zh-cn）。
- Download → `/download`（en）/ `/zh-cn/download`（zh-cn）。

### FeatureGrid 图标修复 + lucide 统一

- 删除 `FeatureGrid.astro:55` 的 `set:html={''}`，修复 6 张卡片图标渲染为空。
- **图标库统一为 lucide 系（用户批准）**：docs 引入 `lucide-static`（lucide 官方纯 SVG 包，零运行时依赖），FeatureGrid / 下载页 / 导航图标全部从 lucide 取 path，**不手写 SVG**。
- 背景：主应用用 `lucide-react` ^0.460.0，docs 无 React；`lucide-static` 与 `lucide-react` 共享同一套 path 数据，视觉同源。docs 引 `lucide-static` 保留纯 Astro 架构，不给文档站引入 React 栈。

### 落地页下载按钮修复

- `DownloadCta.astro`：三个架构按钮改为指向**下载页**（站内），或保留直链但改用 API 动态 URL；探索推荐改为站内下载页 + 下载页承接「GitHub 加速」。

### editLink

- `astro.config.mjs` 启用 `editLink.baseUrl = https://github.com/Eeymoo/peregrine/edit/main/docs/src/content/docs/`（含双语子目录自动适配）。

### CI 闸门（L1 fail + L3 job）

- **release.yml 前置闸门**：新增 `verify-docs` job（`needs` 链在 build 之前或作为独立前置）：解析 `docs/guide/changelog.md` 最新 `## [vX.Y.Z]` 条目版本，与 `GITHUB_REF_NAME` 的 tag 版本比对，不一致 → `exit 1` 中止发布。
- **ci.yml 文档一致性 job**：新增 `docs-consistency`（类似 i18n-check 模式）：比对 `package.json` / `Cargo.toml` 版本号与 changelog 最新条目，不一致 → fail（PR 阶段提前暴露）。

## Capabilities

### New Capabilities

- `docs-download-page`: 独立下载页的交互与数据规范——API 动态版本、加速/直连通道、架构筛选、表格列、查看更多入口。
- `release-docs-sync`: 发版与文档同步的强制时序规范——release 闸门、ci 一致性 job、changelog 最新条目格式约定。

### Modified Capabilities

- `docs-site`: 新增「顶部导航」「editLink」「下载页路由」需求；「URL 路径保持不变」约束需为新增 `/download` 页面扩展。

## Open Questions / Risks

- **changelog 版本条目格式**：当前为 `## [v0.2.4] — 2026-08-13`（含日期），闸门解析用正则 `^## \[(v[\d.]+[^\]]*)\]`。预发布版（`v0.2.3-alpha.0`）是否也须出现在 changelog？探索建议：稳定版强制，预发布可选（闸门仅对纯版本号 tag 生效，与 release.yml 通道判定一致）。
- **构建时 API 限流**：GitHub 未认证 60 次/小时，docs 只在发版时构建，通常足够；若触发限流，降级路径保证页面仍可渲染。
- **加速代理存活期**：gh-proxy 类公益代理不定期失效；下载页应内置 2~3 个候选（如 ghfast.top / gh-proxy.org 系）并允许用户选择，不依赖单一前缀。
- **`Header` 覆写风险**：Starlight 升级时 Header 内部结构可能变动；需在 AGENTS.md 与代码注释中记录，尽量保持覆盖物最小化。
- **release 闸门的逃生口**：已确认不加 `[skip-docs]`；紧急热修若需跳过 changelog，先合 changelog 再发 tag（遵守流程）。
- **lucide-static 的引入位置**：已批准用 `lucide-static`（理解 B），确认引为 docs 的 devDependency，仅取 path 数据不引入运行时；`lucide-react`（理解 A）因破坏 no-React-islands 约定被排除。
