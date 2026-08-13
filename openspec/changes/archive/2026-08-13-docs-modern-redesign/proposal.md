> 跟踪 issue：#55

## Why

文档站从 VitePress 迁移到 Astro + Starlight 后，样式层几乎归零：只有 31 行品牌色映射，其余全是 Starlight 出厂皮肤。首页（产品门面）观感"简陋"，缺少官网应有的视觉表现力；guide 页面排版也停留在默认状态。同时官网一直没有真实的产品截图素材，首页 Hero 区无图可用。

## 目标

- 首页从"Starlight splash 模板 + 6 张朴素卡片"升级为现代化产品落地页，双语（en / zh-cn）一致。
- guide 文档页在不动 Starlight 组件的前提下完成排版与配色精修，明暗双主题均有精致观感。
- 建立可复用的截图资产管线，产出真实的多图层设置界面截图用于首页 Hero。
- 设计与实现遵循 `.agents/skills/design-taste-frontend` 的反套路规范，并通过其 Pre-Flight Check。

## 非目标

- 不改变任何 URL 路径、页面集合、sidebar 结构、信息架构（`docs-site` 既有需求全部保持）。
- 不做可交互 playground（WASM / TS 几何重写均已明确放弃）。
- 不改写文档正文内容（视觉现代化 ≠ 内容重写）。
- 不引入 GSAP / Three.js 等重型动效栈，不动 Starlight 内建组件（除 Hero 覆写外）。
- 不触碰主程序（`src/`、`crates/`、`src-tauri/`）的任何功能代码；截图 mock 仅在 docs 目录内闭环。

## What Changes

- **接入 Tailwind CSS v4**：`@tailwindcss/vite` 插件 + 显式 cascade layers，剥离 preflight、工具类 `tw-` 前缀，避免与 Starlight 内建样式冲突；`@custom-variant dark` 对齐 Starlight 的 `data-theme` 主题开关。
- **设计 token 单一事实源**：品牌蓝 `#2563EB` 色阶迁入 `@theme`，Starlight `--sl-color-*` 变量引用同一 token，消除两处手抄色值。
- **首页落地页化**：覆写 Starlight `Hero.astro` 为自定义 `LandingHero.astro`（左文右图非对称布局），body 组合新 landing 组件：`FeatureGrid`（升级现有 FeatureCards）、`HowItWorks`（三步上手）、`DownloadCta`（三架构下载入口）。全部文案走 props，双语 MDX 各自传入。
- **文档页精修**：新增 `starlight-polish.css`，全量校准 `--sl-color-*` 配色层次、正文排版、code block / aside / 表格 / 链接样式，明暗双主题。
- **截图资产管线**：在 `docs/` 内构建一次性截图工具链——mock Tauri IPC（浏览器内 stub `get_config` / `build_shapes_ipc`，后者使用 Rust 侧一次性导出的真实 shapes JSON）+ Playwright headless 截图，产出多图层设置界面真实截图，存入 `docs/public/img/screenshots/` 供 Hero 使用。
- **AGENTS.md 同步**：更新已过期的 "Documentation Site" 一节（仍写着 VitePress / mermaid / `npm run docs:build`）。

## Capabilities

### New Capabilities

- `docs-visual-design`: 文档站的视觉设计系统与落地页体验要求——Tailwind v4 设计 token、首页落地页构成（Hero / 特性 / 上手步骤 / 下载区）、真实截图资产、文档页排版质量、明暗双主题一致性。

### Modified Capabilities

<!-- 无：docs-site 的基础设施需求（URL / 双语 / 搜索 / llms.txt / sitemap）全部保持不变 -->

## Impact

- **代码**：全部集中在 `docs/`：`astro.config.mjs`、`package.json`（新增 tailwindcss / @tailwindcss/vite / playwright 相关 devDependencies）、`src/styles/`、`src/components/`（新增 landing/ 子目录）、`src/content/docs/index.mdx` + `zh-cn/index.mdx`、`public/img/screenshots/`（新增二进制资产）。
- **截图管线（一次性工具）**：`docs/scripts/`（mock IPC 注入脚本、Playwright 截图脚本）；根目录 `src/` 前端仅在 mock 注入时被加载运行，源码零改动；Rust 侧通过既有 `cargo test` / 示例方式导出 shapes JSON，不新增 crate。
- **CI/CD**：`pages.yml` 无需改动（截图资产提交进仓库，构建时不重新生成）。
- **依赖**：docs 独立 package，新增 `tailwindcss@4`、`@tailwindcss/vite`、`playwright`（dev）。
- **文档**：`AGENTS.md` "Documentation Site" 一节同步更新。
