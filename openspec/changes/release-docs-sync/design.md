# Design：发版与文档同步固化

## 背景

本 change 收敛六类文档/发版欠账，全部在 explore 阶段实地核证（构建产物、workflow 源码、changelog 格式、分支保护状态均查过）。

## 核心洞察：事件时序决定架构

「先合 changelog、后打 tag」是唯一正确的时序，原因是事件链的先后关系：

```
PR 阶段  ── ci.yml 运行（可检查版本↔changelog 一致性，提前暴露）
merge    ── 版本号 bump + changelog 合入 main/dev
push tag ── release.yml 运行
            ├─ [新增] verify-docs 前置闸门：tag 版本 ↔ changelog 最新条目
            ├─ 构建 → assets → Create GitHub Release（最后一步）
            └─ release: published ── pages.yml 重建并部署 docs
                                     （此刻 main 已含新 changelog，零延迟）
```

**推论**：任何「发版后自动改文档 + 自动合 PR」的 workflow 都赶不上本次部署（tag 触发的自动化晚于 `release: published` 触发 pages.yml）。因此放弃该路线，改由 CI 闸门 + 人工前置编辑组成闭环。

## 决策记录

### D1 下载页数据源：构建时 GitHub API（L2-B）

- 不使用「手工维护版本号」（易过期）；不使用「改资产命名去版本号」（动 release.yml 产物 + updater 清单，影响存量）。
- 实现：docs 构建时 `fetch('https://api.github.com/repos/Eeymoo/peregrine/releases')`（或 `releases/latest`），解析 assets。
- 降级：API 失败（限流/网络）时，下载页只渲染「查看 GitHub Releases」外链，保证页面始终可用。
- 发版后 pages.yml 自动重建 → 下载页版本自动更新，**零版本号硬编码**。

### D2 加速/直连通道：仅 zh-cn 渲染，多代理候选

- 组件保持语言无关，通过 prop（如 `showProxy`）由双语 MDX 传入；zh-cn 传 `true`，en 不传。
- 加速实现：给 `browser_download_url` 加 gh-proxy 前缀。内置候选（用户可下拉切换）：
  - `https://ghfast.top/`
  - `https://gh-proxy.com/`
  - `https://ghproxy.net/`
- 不依赖单一前缀（公益代理存活期短）。

### D3 顶部导航：Header 覆写 + sidebar 顶层 link 双轨

- 桌面：覆写 Starlight `Header.astro`，在 site title 后插入文字链接（`AukCraft ↗` 外链、`Docs`、`Download`），CSS 在窄屏隐藏；移动端靠 sidebar 顶层 `link` 项（随抽屉展示）。
- **破例记录**：AGENTS.md 当前写「仅覆写 Hero，不增加其他覆写」；本 change 需同步更新该约定，注明 Header 为第二个必要覆写，且保持覆盖物最小化（只加导航区，不动其他内部结构）。

### D4 图标：删除 `set:html={''}` + 图标库统一为 lucide（用户批准）

- **修复渲染 bug**：`FeatureGrid.astro:55` 的 `<svg ... set:html={''}>` 删除该属性；`:57` 的 `<Fragment set:html={ICONS[f.icon]}/>` 保留。构建产物中 6 个 svg 全空的根因即此。
- **图标库统一为 lucide 系（用户决策，批准）**：docs 不再手写 SVG path，改用 `lucide-static`（lucide 官方纯 SVG 包，零运行时依赖）。
  - 背景核证：主应用（React 前端）用 `lucide-react` ^0.460.0；docs 项目无 React、无 `@astrojs/react`、landing 全纯 Astro。`lucide-react` 与 `lucide-static` 共享同一套 path 数据，视觉 100% 同源。
  - 决策：采用**理解 B**——docs 引 `lucide-static`，FeatureGrid / 下载页表格 / 导航链接图标全部从 lucide 取 path，**绝不手写**；不给纯静态文档站引入 React 栈，保留「纯 Astro + scoped CSS」架构。
  - 已排除理解 A（docs 装 `@astrojs/react` + `lucide-react`）：破坏 `docs-modern-redesign` 明确的 no-React-islands 约定，仅为几个图标引入整套 React 依赖链。
  - 注意：`FeatureGrid.astro` 现有注释"内联 Phosphor 风格"是错误描述（主应用根本没有 Phosphor），需同步更正。

### D5 落地页下载按钮：改指向站内下载页

- `DownloadCta.astro` 三架构按钮的 `href` 从 404 直链改为 `/download`（en）/`/zh-cn/download`（zh-cn），由下载页承接架构选择与加速；`releasesText` 外链保留。

### D6 release.yml 闸门：fail 硬拦（L1）

- 新增 `verify-docs` job（独立于 matrix build，作为 `release` job 或 build 的前置依赖）。
- 判定：解析 `docs/src/content/docs/guide/changelog.md`，取第一个 `## [vX.Y.Z]`（正则 `^## \[(v[\d.]+[^\]]*)\]`），与 `GITHUB_REF_NAME` 比对。
- 只对纯版本号 tag 生效（tag 含 `-` 时跳过，与 release.yml 通道判定一致）。
- 不等 → `exit 1`，中止发布。已确认不加 `[skip-docs]` 逃生口。

### D7 ci.yml 一致性 job：PR 阶段提前暴露（L3）

- 新增 `docs-consistency` job，仿 `i18n-check` 模式：比对 `package.json` 版本与 changelog 最新条目。
- 说明：该 job 的触达时机在 merge 前，可提前提醒；release 闸门（D6）才是最终强制。

### D8 editLink：零成本启用

- `astro.config.mjs` 加 `editLink: { baseUrl: 'https://github.com/Eeymoo/peregrine/edit/main/docs/src/content/docs/' }`，Starlight 自动适配双语子目录路径。

## 文件影响面

```
docs/
├── astro.config.mjs              # editLink + Header 覆写注册 + 下载页路由感知
├── package.json                  # 新增 devDependency: lucide-static
├── src/components/landing/
│   ├── FeatureGrid.astro          # 删 set:html={''}（修复图标）+ 改用 lucide-static path
│   ├── DownloadCta.astro          # 按钮改指站内下载页 + 图标改 lucide-static
│   ├── DownloadTable.astro        # [新] 下载页表格组件（通道/加速/筛选/列）
│   └── HeaderLink.astro 或等价    # [新] 或直接在 Header 覆写内联
├── src/components/Header.astro    # [新] 覆写：桌面顶栏导航链接
├── src/content/docs/
│   ├── download.mdx               # [新] en 下载页
│   └── zh-cn/download.mdx         # [新] zh-cn 下载页（showProxy=true）
├── index.mdx / zh-cn/index.mdx    # （可选）落地页文案微调
└── src/styles/starlight-polish.css # 导航链接/下载页表格样式（如需要）
.github/workflows/
├── release.yml                    # 新增 verify-docs 前置 job
└── ci.yml                         # 新增 docs-consistency job
AGENTS.md                          # 更新「仅覆写 Hero」约定 + 下载页说明
```

## 验证方案

- 本地：`cd docs && npm run build` 成功；下载页在 API 可用/降级两态均渲染；FeatureGrid 图标出现；editLink 链接指向正确文件。
- 落地页：三架构按钮不再指向 404 URL。
- CI：`docs-consistency` job 对「版本↔changelog 不一致」的样例提交 fail。
- release 闸门：构造 tag≠changelog 的 dry-run 校验脚本，确认 `exit 1` 路径。
- 视觉：zh-cn 下载页有加速通道；en 下载页无加速通道；明暗双主题检查。
