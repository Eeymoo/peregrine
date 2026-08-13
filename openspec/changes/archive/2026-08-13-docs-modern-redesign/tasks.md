## 1. Tailwind v4 接入与设计 token

- [x] 1.1 `docs/` 安装 `tailwindcss@4` 与 `@tailwindcss/vite@4`（已完成 4.3.3），补装官方兼容包 `@astrojs/starlight-tailwind`，`astro.config.mjs` 注册 vite 插件
- [x] 1.2 新建 `src/styles/global.css`（官方配方）：`@layer base, starlight, theme, components, utilities` + `@import '@astrojs/starlight-tailwind'` + theme/utilities 两层
- [x] 1.3 `@theme` 定义 `--color-accent-50..950`（品牌蓝 `#2563EB` 为 600）与 `--color-gray-50..950`（zinc 系），品牌别名 `--color-brand-*` 同阶映射；`astro.config.mjs` 的 `customCss` 改为 `[global.css, starlight-polish.css]`
- [x] 1.4 新建 `src/styles/starlight-polish.css`，删除旧 `custom.css`（accent 已由 D3 机制覆盖）
- [x] 1.5 验证：`npm run build` 成功，guide 页排版无 reset 迹象，Starlight UI 已消费 accent/gray token（明暗各目检一页）

## 2. 截图资产管线

- [x] 2.1 spike：验证 `@tauri-apps/api` 在纯浏览器环境的 mock 注入点（`window.__TAURI_INTERNALS__` 或等价机制），根目录 `npm run dev` 起 Vite 后设置面板可在 Chromium 渲染（结论：ConfigApp 窗口（label=config）才是图层编辑器所在，SettingsApp 无预览）
- [x] 2.2 Rust 一次性导出：用精心调制的多图层 `Profile` 调 `build_layers_shapes`，输出 shapes JSON 到 `docs/scripts/fixtures/`（用 `cargo test -- --nocapture` 或临时 example，不新增 crate、不提交主程序改动）
- [x] 2.3 `docs/scripts/mock-tauri.js`：注入脚本，stub `get_config`（返回多图层 AppConfig）、`build_shapes_ipc`（返回 fixtures JSON）、其余命令安全默认值
- [x] 2.4 `docs/scripts/capture-screenshots.mjs`：Playwright headless 按既定视口与主题截取多图层设置界面，产物压缩（≤1600 宽、≤300KB）存入 `docs/public/img/screenshots/`（实测：主程序 UI 固定深色主题，只产单张 1600×1000、71KB）
- [x] 2.5 验证：截图中 UI 完整、预览图元与 fixtures 一致；脚本可重复执行（canvas 像素多样性断言 + 零 console error）

## 3. 首页落地页

- [x] 3.1 `src/components/landing/LandingHero.astro`：左文右图非对称 Hero，读 frontmatter `hero` 数据并防御性渲染，右栏引用 2.x 产出的真实截图；`astro.config.mjs` 注册 `components: { Hero: ... }`
- [x] 3.2 `src/components/landing/FeatureGrid.astro`：升级现有 FeatureCards，非等宽网格、单 accent、图标用 Phosphor（新增 `@phosphor-icons/react` 或自包含 SVG sprite 依赖）
- [x] 3.3 `src/components/landing/HowItWorks.astro`：三步上手（下载 → 选窗口 → 调样式），动词命名无 "Step N" 前缀
- [x] 3.4 `src/components/landing/DownloadCta.astro`：x64 / x86 / ARM64 三入口 + GitHub Releases 链接，产物命名与 release.yml 核对一致
- [x] 3.5 更新 `src/content/docs/index.mdx` 与 `zh-cn/index.mdx`：hero frontmatter + body 组合三个区块，双语文案对齐
- [x] 3.6 CSS 入场渐显（仅 transform/opacity，包在 `prefers-reduced-motion: no-preference` 内）；旧 `FeatureCards.astro` 删除

## 4. 文档页精修

- [x] 4.1 `starlight-polish.css` 全量校准 `--sl-color-*` 配色层次（bg / hairline / text / accent），明暗双主题
- [x] 4.2 `.sl-markdown-content` 排版细节：行高与段落间距、code block 圆角边框、aside 强调条、表格样式、链接下划线
- [x] 4.3 验证：明暗双主题 × 中英文各抽查 2 篇 guide，无裸默认样式（computed style 程序化断言全 PASS）

## 5. 验收与收尾

- [x] 5.1 design-taste-frontend §14 Pre-Flight Check 全项过一遍（em-dash 零出现、单 accent、CTA 对比度、eyebrow 计数等）
- [x] 5.2 Playwright 截图验收矩阵：{首页, guide} × {明, 暗} × {桌面, 移动视口} × {en, zh-cn}
- [x] 5.3 更新 `AGENTS.md` "Documentation Site" 一节（Astro/Starlight 现状、截图管线、脚本用法）
- [x] 5.4 `npm run build` 最终构建通过；根仓库 `git status` 确认改动仅限 `docs/`、`openspec/`、`AGENTS.md`、`.agents/skills/`
