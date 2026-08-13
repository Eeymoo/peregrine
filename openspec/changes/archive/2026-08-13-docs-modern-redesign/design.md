## Context

文档站已完成 VitePress → Astro 7 + Starlight 0.41 迁移（见已归档的 documentation-output 变更），但样式层只搬运了品牌色（`src/styles/custom.css`，31 行），其余维持 Starlight 出厂皮肤。首页为 `template: splash` + frontmatter hero + 唯一自定义组件 `FeatureCards.astro`；`public/` 仅 logo 与 favicon，无任何产品截图。

约束：

- Starlight 的排版与骨架样式依赖其内部 class 与 `--sl-color-*` 变量体系，任何全局 CSS reset（如 Tailwind preflight）都会破坏它。
- Starlight 主题切换通过 `data-theme="dark"|"light"` 属性，与 Tailwind v4 默认的 `prefers-color-scheme` 暗色策略脱钩，必须显式桥接。
- 覆盖层与设置面板是 Windows-only 的 Tauri 应用，本开发机（headless Linux）无法真实运行；截图必须走"浏览器 mock IPC + headless Chromium"路径。
- 主程序前端的几何计算已收口到 Rust 侧（`build_shapes_ipc`，前端零几何逻辑），预览图元可由 Rust 一次性导出 JSON 复用。
- 设计执行遵循 `.agents/skills/design-taste-frontend`（Redesign - Preserve 模式：保留品牌蓝与 IA，做视觉现代化）。

## Goals / Non-Goals

**Goals:**

- 首页升级为现代化产品落地页（Hero + 特性网格 + 三步上手 + 下载区），en / zh-cn 双语一致。
- guide 页排版与配色精修，明暗双主题均精致。
- 产出真实的多图层设置界面截图用于 Hero，并固化为可复跑的截图脚本。
- 设计 token 单一事实源：品牌蓝色阶一处定义，Tailwind 与 Starlight 变量共同引用。

**Non-Goals:**

- 不改 URL / IA / sidebar / 正文内容；不引入交互 playground；不引入 GSAP / Three.js / React 岛（landing 组件保持纯 Astro + CSS 动效）。
- 不改主程序功能代码；截图 mock 只在浏览器注入层生效。
- 不在 CI 中重新生成截图（资产提交入库）。

## Decisions

### D1：Tailwind v4 经 `@tailwindcss/vite` + 官方兼容包 `@astrojs/starlight-tailwind` 接入

- 做法（Starlight 0.41 官方指南）：`npm install @astrojs/starlight-tailwind`；`astro.config.mjs` 加 `vite: { plugins: [tailwindcss()] }`；入口 CSS：
  ```css
  @layer base, starlight, theme, components, utilities;
  @import '@astrojs/starlight-tailwind';
  @import 'tailwindcss/theme.css' layer(theme);
  @import 'tailwindcss/utilities.css' layer(utilities);
  ```
  兼容包负责三件事：桥接 `dark:` 变体到 Starlight 暗色模式、恢复 preflight 的必要部分、让 Starlight UI 直接消费 Tailwind theme token。
- 理由：曾假设该兼容包是 v3 时代产物而计划手写 prefix + `@custom-variant` 方案，核对官方文档后推翻——手写方案与官方 cascade layer 管理重复，且前缀会改写 theme 变量名导致兼容包失效。
- 备选：`tw-` 前缀 + 全手写桥接（增加维护面、与官方 layer 管理打架）——否决；不接 Tailwind 纯手写 CSS——否决。

### D2：暗色桥接交由兼容包处理

- 做法：`@astrojs/starlight-tailwind` 已将 Tailwind `dark:` 变体绑定到 Starlight 的暗色模式实现，不另写 `@custom-variant`。
- 理由：Starlight 允许用户手动切主题，暗色必须跟随站点开关而非 `prefers-color-scheme`；官方兼容包已解决且随版本维护。

### D3：品牌 token 单一定义于 `@theme`，Starlight UI 直接消费

- 做法：`@theme` 中定义 `--color-accent-50..950`（品牌蓝，主色 `#2563EB` 对应 600）与 `--color-gray-50..950`（冷灰 zinc 系）；Starlight 0.41 原生将这两组 token 用于链接、导航高亮、背景与边框（官方 "Styling Starlight with Tailwind" 机制）。品牌别名 `--color-brand-*` 映射到同一色阶供 landing 组件语义化引用。`custom.css` 的手抄色值删除。
- 理由：Starlight 原生消费 Tailwind token，比手写 `--sl-color-*` 映射更稳定；现状两处手写同一蓝色，漂移只是时间问题。

### D4：首页 = Hero 覆写 + MDX body 组合 landing 组件（纯 Astro，无 React 岛）

- 做法：`starlight({ components: { Hero: './src/components/landing/LandingHero.astro' } })`。LandingHero 读 frontmatter `hero` 数据并完全自绘：左侧 eyebrow + 标题 + ≤20 词副标题 + 双 CTA，右侧真实截图（带深色边框的窗口框质感）。body 依次排布 `FeatureGrid`（6 卡，非等宽网格避免"三张一样卡"套路）、`HowItWorks`（下载 → 选窗口 → 调样式，动词命名不加"Step 1"前缀）、`DownloadCta`（x64 / x86 / ARM64 三入口 + GitHub Releases 链接）。
- 理由：覆写 Hero 可保留 Starlight 顶栏（语言切换 / 主题开关 / GitHub 链接 / 搜索），splash 模板已隐去侧边栏；纯 Astro 组件零 JS 运行时成本，动效用 CSS `transform/opacity` 过渡即可满足 MOTION_INTENSITY 4-5 的目标。
- 备选：绕过 Starlight 自建落地页（丢失 i18n 路由与主题同步，重复造轮）——否决；引入 React + Motion 岛（为几个渐显动画引入整个 React 运行时，不值）——否决，如后续需要再升级。

### D5：截图管线 = Rust 导出真实图元 + 浏览器 mock IPC + Playwright headless

- 数据流：
  1. 一次性 Rust 导出（`cargo test -- --nocapture` 或临时 example，不新增 crate）：用精心调制的多图层 `Profile` 调 `build_layers_shapes`，输出 shapes JSON 到 `docs/scripts/fixtures/`。
  2. `docs/scripts/mock-tauri.js`：在页面加载前注入，stub `@tauri-apps/api` 的 IPC 通道——`get_config` 返回该 Profile 对应的完整 `AppConfig`，`build_shapes_ipc` 返回 fixtures JSON，其余命令返回安全默认值。
  3. 根目录 `npm run dev` 起 Vite；Playwright（`docs/scripts/capture-screenshots.mjs`）以 addInitScript 注入 mock，打开设置面板，切到多图层编辑态，按既定视口（1600×1000，明暗各一）截图存入 `docs/public/img/screenshots/`。
- 理由：截图内容 = 真实 React UI + 真实 Rust 几何，WYSIWYG 字面成立；管线可复跑，发版本换 UI 后重新出图即可。
- 风险前置：mock 注入点取决于 `@tauri-apps/api` 在非 Tauri 环境的检测方式（`window.__TAURI_INTERNALS__`），实现时先用最小 spike 验证注入可行性，再写完整管线。

### D6：文档页精修只动 CSS，不覆写 Starlight 组件

- 做法：`starlight-polish.css` 全量校准 `--sl-color-*`（bg / hairline / text / accent 层次）、`.sl-markdown-content` 下的排版细节（行高、段落间距、code block 圆角与边框、aside 左侧强调条、表格斑马纹与边框、链接下划线偏移）、明暗双主题各一套。
- 理由：Starlight 组件覆写会绑定其内部 DOM 结构，升级易碎；CSS 变量是官方稳定的主题接口，维护成本最低。

### D7：视觉语言（design-taste-frontend 决策记录）

- Design Read：面向玩家的开源桌面工具落地页，dark-tech 极简语言，Tailwind v4 + 纯 CSS 动效。
- 三旋钮：`DESIGN_VARIANCE 6 / MOTION_INTENSITY 4 / VISUAL_DENSITY 4`——左文右图非对称 hero、克制的入场渐显、标准留白节奏。
- 模式：Redesign - Preserve。保留品牌蓝 `#2563EB`、logo、全部 IA 与文案骨架；中性色统一冷灰（zinc 系），单 accent 贯穿全站；圆角体系统一（卡 12px / 按钮 pill）。
- 字体：拉丁用自托管 display sans（Geist 或 Outfit），中文落系统 CJK 栈，不使用 serif；禁 em-dash（`—`）于一切可见文案（中文破折号亦不在 UI 文案使用）。

## Risks / Trade-offs

- [Tailwind v4 与 Starlight 0.41 的 cascade layer 细节随版本漂移] → 接入后第一件事跑 `astro build` + 目检明暗双主题；prefix/layer 写法以当时官方文档为准核对一次。
- [mock IPC 注入方式依赖 `@tauri-apps/api` 内部检测逻辑，版本升级可能失效] → spike 先行；注入脚本集中于 `docs/scripts/mock-tauri.js` 单文件，失效时只改一处。
- [Hero 覆写绑定 Starlight frontmatter `hero` 数据结构] → LandingHero 对缺省字段做防御性渲染（无 actions / image 时退化纯排版）。
- [截图资产增大仓库体积] → 单张 ≤ 300KB（pngquant / 缩放至 ≤1600 宽），总量控制在 4 张以内。
- [CSS 动效在 `prefers-reduced-motion` 下需全部塌缩] → 所有动画包在 `@media (prefers-reduced-motion: no-preference)` 内。

## Migration Plan

1. 接入 Tailwind v4 + token 迁移（构建不破坏现状为验收）。
2. 截图管线 spike → 出图 → 入库。
3. landing 组件 + Hero 覆写 + 双语 index.mdx。
4. `starlight-polish.css` 文档页精修。
5. Pre-Flight Check（design-taste-frontend §14）+ 明暗/双语/移动视口截图验证。
6. 更新 AGENTS.md。

回滚：全部改动限于 `docs/` 与 `AGENTS.md`，revert 即可；无运行时迁移。

## Open Questions

- 自托管 display 字体选 Geist 还是 Outfit（需下载 woff2 入库，~100KB）？实现时按实际渲染效果定。
- `DownloadCta` 的三个架构入口文案是否与 release.yml 产物命名保持一致（`windows-x64-setup.exe` 等）？实现时核对 snapshot/release 产物名。
