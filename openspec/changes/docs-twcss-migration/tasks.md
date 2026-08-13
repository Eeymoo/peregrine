# Tasks: docs-twcss-migration

## 1. 迁移基础设施

- [x] 1.1 在 `docs/src/styles/global.css` 的 `@theme` 块对齐断点：`--breakpoint-md: 50rem`（D3），并用 `rg '(sm|md|lg|xl|2xl):' docs/src` 复核无存量的 Tailwind 断点工具类受影响
- [x] 1.2 为 `HowItWorks` 区块在 `docs/scripts/verify-polish.mjs`（或 verify-landing.mjs，按现有归属）补充计算样式断言（断言先行），确认迁移前断言全绿

## 2. HowItWorks（趟通流程）

- [x] 2.1 将 `HowItWorks.astro` 的 scoped CSS（45 行）迁移为 Tailwind 工具类，`--sl-color-*` 按 design.md D4 映射表转换
- [x] 2.2 执行 `npm run build && npm run verify`，确认双主题、双语（en / zh-cn）断言全绿
- [x] 2.3 独立 commit（仅含本组件迁移）

## 3. Header（仅自定义导航区，迁移 + 现代化）

- [x] 3.1 为 Header 导航区（`.header-nav` / `.header-nav-link` / `.header-nav-icon`，桌面端 `hidden md:flex`）补充计算样式断言，确认迁移前全绿
- [x] 3.2 将导航区三个自定义类迁移为工具类（含 `[&_svg]:size-3.5` 形式的图标尺寸），Starlight 复制区（`.header` grid / `.title-wrapper` / `.right-group` / `.social-icons`）保持原样（D5）
- [x] 3.3 现代化项 1：Docs / Download 链接的当前页 active 态指示（accent 色 + 下划线或等价处理，路由判定逻辑最小化）
- [x] 3.4 现代化项 2：hover 反馈升级为 200ms 过渡 + 下划线 scaleX 动画（或淡色 pill 底，二选一），reduced-motion 塌缩
- [x] 3.5 现代化项 3：`focus-visible` 焦点环补齐
- [x] 3.6 将 3.3-3.5 的预期样式写入 verify 断言，`npm run build && npm run verify` 全绿后独立 commit（message 逐项列出视觉变更）

## 4. FeatureGrid / DownloadCta（含布局词汇对齐，D10）

- [x] 4.0 新增 `SectionHeading.astro` 复用组件（序号 + micro mono 标签 + hairline 贯穿线，全工具类实现），落地页区块（FeatureGrid / HowItWorks / DownloadCta）以 props 接入并统一以其开篇；断言先行
- [x] 4.1 为两个区块补充计算样式断言（若现有 verify 未覆盖）
- [x] 4.2 迁移 `FeatureGrid.astro`（73 行）为工具类；现代化项：卡片改 hairline 分隔网格（`gap-px` + hairline 底色）+ 图标品牌色淡染容器 + 卡片 hover 微反馈，写入断言，verify 全绿后 commit
- [x] 4.3 迁移 `DownloadCta.astro`（77 行）为工具类；现代化项：CTA label 改 mono 微标签形态 + CTA `:active` 下压触感，写入断言，verify 全绿后 commit
- [x] 4.4 落地页区块接入滚入揭示（`IntersectionObserver` + `[data-reveal]`，600ms rise + 70ms 错峰，`html.js` 门控 + reduced-motion 塌缩）；HowItWorks 序号同步改 mono 微标签形态；区块垂直节奏对齐家族大留白尺度（D10 签名元素 5）；补充断言，verify 全绿后 commit

## 5. LandingHero

- [x] 5.1 在 `global.css` 的 `@theme` 定义 `--animate-rise` token + `@keyframes rise`（D6），组件侧改 `animate-rise` + 任意值 delay；`prefers-reduced-motion` 塌缩块保留为豁免 CSS
- [x] 5.2 迁移 `LandingHero.astro` 其余 scoped CSS（132 行）为工具类，`--sl-color-hairline` 等兼容包未覆盖变量用 v4 任意值变量简写引用
- [x] 5.3 枚举该组件现代化项（候选：截图框 hover 微浮起）并写入断言，`npm run build && npm run verify` 全绿后独立 commit

## 6. DownloadTable

- [x] 6.1 判定 `is-active` 态样式的最终形态（工具类 + `aria-pressed` 属性选择器 vs 最小残留 CSS），记入 commit message
- [x] 6.2 迁移 `DownloadTable.astro`（145 行）静态样式为工具类，保留 JS 状态钩子类；枚举该组件现代化项（候选：筛选按钮态反馈细化）并补充/更新计算样式断言
- [x] 6.3 `npm run build && npm run verify` 全绿后独立 commit

## 7. starlight-polish.css 与收尾

- [x] 7.1 逐条判定 `starlight-polish.css`（173 行）规则归属：可归工具类的迁移，必须豁免（`:global` / `.sl-markdown-content` / 级联层声明）的保留并注释豁免理由
- [x] 7.2 全量 `npm run build && npm run verify`（双主题、双语）最终验收 + taste-skill pre-flight 逐页过一遍（D7 门禁）
- [x] 7.3 更新 AGENTS.md 文档站设计系统小节：迁移状态、豁免清单、D3 断点对齐与 D4 token 映射规约、D9 现代化边界
- [x] 7.4 将英文文案 em-dash 问题（download.mdx 3 处）登记为后续任务（不属于本 change）