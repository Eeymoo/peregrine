> 跟踪 issue：#64

## Why

文档站已接入 Tailwind v4 品牌 token（`docs-modern-redesign` / `docs-twcss-migration` 两轮改造），但视觉语言仍停留在 Starlight 出厂皮肤的「soft / 圆角 / 阴影」质感：按钮 `rounded-full`、截图框 `rounded-xl` + 阴影、浅色主题近纯白、中性色为 zinc 冷灰。这与 aukcraft 家族站（aukcraft.org）的暗色编辑风——hairline 分层、≤4px 圆角、零阴影、蓝调近黑/近白、mono 微标签、serif-italic 强调——明显不协调。本次将 `.agents/skills/aukcraft-site-design` 的壳层设计系统落到 docs 站，同时按项目既定约束保留 Starlight 架构、品牌蓝 `#2563EB` 与明暗双主题。

## 目标

- 全站（落地页 + 16 指南页 × 2 locale）切换到 aukcraft 壳层的表面语言：零阴影、≤4px 圆角、hairline 分层；明暗双主题为「蓝调近黑 `#0B0E11` / 蓝调近白 `#F5F7F9`」，均非纯黑/纯白。
- 落地页引入签名组件：`DotField`（点阵动态背景）、`FlightLine`（边框描边按钮），替换现有 pill 按钮；保留真实截图 Hero，仅换底色并引入 serif-italic 标题强调。
- 品牌蓝 `#2563EB` 作为唯一主色并「节制使用」——仅链接 / CTA / 描边动线 / 焦点环 / 画布光标填色，不用于背景或装饰。
- 动效遵守 `prefers-reduced-motion`：点阵、描边、reveal 全部塌缩为静态或瞬时变色。

## 非目标

- 不脱离 Starlight：保留 sidebar / 搜索 / 主题开关 / 语言切换 / TOC / `llms.txt` / sitemap（`docs-site` 既有需求全部不变）。
- 不改变任何 URL 路径、页面集合、sidebar 结构、信息架构，不改写文档正文内容。
- 不引入 `HeroCanvas`（点阵字标营销 hero）、不引入 teal 主色、不做单暗色主题。
- 不为 Starlight 自带控件新增 `components:` 组件覆写（Hero / Header 之外不加第三个）；但控件（侧栏 pill、搜索框、主题切换）**允许 customCss 级覆写** radius/shadow，纳入 ≤4px + 零阴影体系。
- 不触碰主程序（`src/`、`crates/`、`src-tauri/`）。
- 不引入 GSAP / Three.js 等重型动效栈；DotField / FlightLine 为原生 canvas / SVG + rAF 实现。

## What Changes

- **中性色 ramp 重定义**：`@theme` 中 `--color-gray-*` 由 zinc 冷灰改为 aukcraft 蓝调灰阶（暗色 base `#0B0E11` / raised `#14181D` / ink `#EDEDED` / muted `#8A9199`；浅色 base `#F5F7F9` / raised `#FFFFFF` / ink `#11151A` / muted `#5A6472`），hairline 由 `rgba(255,255,255,.08)` / `rgba(15,23,42,.08)` 提供。**取代 `docs-modern-redesign` 的 zinc 中性色决策。**
- **圆角体系收紧**：全站容器级圆角由 0.75rem、按钮由 `rounded-full` 统一收紧到 ≤4px（容器 4px、行内 2px）。**取代 `docs-twcss-migration` 的 0.75rem 容器圆角决策。**
- **阴影移除**：`LandingHero` 截图框 `shadow-[...]` 及 Starlight 内容区相关阴影全部改为 hairline 边框分层（`rgba(...)` 1px）。
- **落地页签名组件**：新增 `DotField.astro`（仅落地页，点阵光标填色，蓝 fill）与 `FlightLine.astro`（边框描边 SVG）；`DownloadCta` / `DownloadTable` 的 pill 按钮 → `.flight` 方形描边按钮（暗色下文字与描边动线用 `accent-400` 亮阶，保证小字对比度 ≥4.5:1）；Hero 标题引入 Newsreader serif-italic 强调（中文用系统 serif 栈，不引入 CJK webfont）。
- **噪声颗粒**：新增 `.noise` 胶片颗粒 overlay（2.5% 透明度，fixed + pointer-events-none），**仅落地页生效**（与 DotField 同范围），不穿透 guide 页正文。
- **指南页精修**：重写 `starlight-polish.css`，将 code block / aside / 表格 / 行内码的圆角 + 阴影统一为 ≤4px + hairline 边框；侧栏 pill / 搜索框 / 主题切换等 Starlight 控件也在 customCss 层定点覆写为 ≤4px + 零阴影（每条覆写注释标注目标版本与内部 class，供升级复查）。
- **AGENTS.md 同步**：更新 "Documentation Site" 一节的圆角/阴影/中性色描述。

## Capabilities

### New Capabilities

<!-- 无：新元素（DotField / FlightLine / 中性色 ramp）均属视觉设计系统，归入 docs-visual-design。 -->

### Modified Capabilities

- `docs-visual-design`: 设计系统要求由「品牌蓝 + zinc 冷灰 + 0.75rem 圆角 + 阴影分层」调整为 aukcraft 壳层语言——蓝调近黑/近白双主题、≤4px 圆角、零阴影 + hairline 分层、DotField / FlightLine 签名组件、serif-italic 标题强调、reduced-motion 塌缩。

## Impact

- **代码**：全部集中在 `docs/`：`src/styles/global.css`（中性色 ramp 重写）、`src/styles/starlight-polish.css`（重写）、`src/components/landing/`（Hero / DownloadCta / DownloadTable 改造）、`src/components/`（新增 `DotField.astro` / `FlightLine.astro` / `noise` 样式）、`src/content/docs/index.mdx` + `zh-cn/index.mdx`（挂载新组件）、`src/lib/icons.ts`（如需新增图标）。
- **依赖**：docs 独立 package；`package.json` 新增 `@fontsource-variable/newsreader`（中文 serif 强调走系统字体栈，无 CJK webfont 依赖）。
- **CI/CD**：`pages.yml` 无需改动；`npm run verify`（`verify-landing.mjs` / `verify-polish.mjs`）断言需随新设计更新。
- **文档**：`AGENTS.md` "Documentation Site" 一节同步。
