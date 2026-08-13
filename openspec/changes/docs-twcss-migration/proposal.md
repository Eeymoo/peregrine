# Proposal: docs-twcss-migration

> Status: proposed
> **跟踪 issue：#61**（https://github.com/Eeymoo/peregrine/issues/61）

## Why

文档站已接入 Tailwind CSS v4（`@tailwindcss/vite` + `@astrojs/starlight-tailwind`，token 单源在 `@theme`），但 6 个自定义组件（`Header.astro` + 5 个 landing 组件）仍共保留约 470 行手写 scoped CSS，另有 173 行全局 `starlight-polish.css`。同一站点存在两套样式体系，新增样式时无统一入口，token 单源约束也无法约束手写 CSS 里的 `var(--sl-color-*)` 直引。借迁移到 Tailwind 工具类之机，同时对这些组件做**克制的视觉现代化**（交互反馈、状态表达、动效细节），让站点更现代但不改头换面。

同时，组织站（aukcraft.org）与产品文档站（peregrine.aukcraft.org）当前视觉语言各自为政，用户无法感知二者是同一团队出品的系列产品。aukcraft.org 已建立一套鲜明的版式语法（SectionHeading 序号标签、mono 微标签、hairline 分隔网格、滚入揭示节奏、大留白单列叙事），本 change 让文档站成为这套**aukcraft 产品家族设计语言**的第二个实践者——家族版式语法完全对齐，产品个性（品牌色、主题模式、站型构图）保持各自独立。

## What Changes

- 将文档站自定义组件的 scoped `<style>` 逐步迁移为 Tailwind v4 工具类，迁移顺序：`HowItWorks`（先行趟通流程）→ `Header`（仅自定义导航区）→ `FeatureGrid` → `DownloadCta` → `LandingHero` → `DownloadTable` → `starlight-polish.css`（部分）。
- **迁移 + 现代化同步进行**：每个组件除样式实现方式迁移外，附带一组**显式枚举**的现代化项（写入 design.md D9 与各组件任务），现代化项之外的部分计算样式保持不变。
- **Header 导航区现代化**（首个重点）：当前页 active 态指示（现在完全没有）、hover 反馈升级为带过渡的交互（下划线动画或淡色 pill 底）、`focus-visible` 焦点环补齐。
- 其余组件的现代化项在做到该组件时枚举确认，候选方向：FeatureGrid 图标容器品牌色淡染 + 卡片 hover 反馈、HowItWorks 步骤间连接线索、CTA `:active` 触感反馈、LandingHero 截图框 hover 微浮起。
- 验收标准：`npm run build` 通过 + `npm run verify` 全绿（断言同步更新为现代化后的预期样式），且视觉变更逐项可追溯。
- `@keyframes` 动效迁移为 Tailwind v4 的 `@theme --animate-*` token + `animate-*` 工具类。
- `Header.astro` 中逐字复制自 Starlight 默认 Header 的结构与 CSS **保持原样**（保留升级 diff 工作流），仅迁移并现代化本站新增的 `.header-nav*` 自定义区。
- `DownloadTable` 的 JS 状态钩子类（如 `is-active`）保留，仅迁移静态样式。
- **建立 aukcraft 产品家族统一版式语法**（对齐 aukcraft.org）：新增 `SectionHeading.astro` 复用组件（序号 + micro 标签 + hairline 贯穿线），落地页各区块（FeatureGrid / HowItWorks / DownloadCta）统一以其开篇；特性卡片采用 hairline 分隔网格（`gap-px` + hairline 底色）；eyebrow/标签类文本统一为 mono + uppercase + 宽字距的微标签形态；落地页区块接入滚入揭示（`IntersectionObserver` + `[data-reveal]`，错峰 70ms 递增）；区块垂直节奏对齐组织站的大留白单列叙事（`max-w-5xl` 量级容器 + 大段距）。**家族签名是版式语法**——产品个性（品牌蓝 + zinc 双主题、Hero 左文右图构图、圆角体系）保持独立，不跟 aukcraft 的深色单主题 + teal accent / Flight Line / DotField / scroll-snap。一致性**完全由视觉语言表达**：不新增任何归属声明类文案或标识（如 "by aukcraft" 角标、联名标语），文案内容一字不动。
- **非本 change 范围**（另起任务）：文案层 em-dash 等 taste-skill pre-flight 文案问题（英文 `—` 违规、中文 `——` 属规范标点予以保留）。

## 目标

- 文档站自定义组件样式统一以 Tailwind 工具类表达，手写 scoped CSS 收敛到豁免清单内。
- 站点交互与细节在现有设计语言内明显现代化：导航有状态反馈、卡片/CTA 有触感、动效克制且 `prefers-reduced-motion` 全量塌缩。
- 形成可复用的迁移与现代化规约（断点对齐、token 映射、豁免清单、现代化边界），写入 design.md。
- 落地页与组织站（aukcraft.org）形成可感知的产品家族统一风格：共享版式语法（section heading / hairline 网格 / mono 微标签 / 滚入揭示 / 大留白节奏），产品个性（品牌色、主题模式、构图）各自独立。

## 非目标

- 不做整体重新设计：品牌蓝（`#2563EB`）、zinc 中性色、信息架构、页面结构、双语内容、Hero 左文右图构图全部保留。
- 不改动 Starlight 默认组件的覆写结构（`Header` 复制区、`LandingHero` 的覆写注册方式）。
- 不处理文案问题（em-dash 等另起任务）。
- 不引入重型动效（无滚动劫持、无视差、无 GSAP）；`MOTION_INTENSITY` 从 3 提升到 4-5 为止。
- 不迁移主应用 `src/`（其已 ~99% twcss，剩余 inline style 为真动态值）。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `docs-visual-design`: 新增「自定义组件样式实现方式」要求——自定义组件样式 MUST 优先以 Tailwind 工具类实现，手写 CSS 仅限豁免清单（Starlight 复制区、JS 状态钩子、Starlight 内部选择器全局润色）。

## Impact

- **代码**：`docs/src/components/Header.astro`、`docs/src/components/landing/{HowItWorks,FeatureGrid,DownloadCta,LandingHero,DownloadTable}.astro`、`docs/src/styles/starlight-polish.css`、`docs/src/styles/global.css`（可能新增 `--animate-*` token 或断点对齐）。
- **验证**：`docs/scripts/verify-landing.mjs` / `verify-polish.mjs` 为每步验收门禁；可能需要为 Header 导航区补充计算样式断言。
- **文档**：AGENTS.md 中文档站设计系统小节需在迁移完成后同步（豁免清单、迁移状态）。
- **无运行时 / 无后端影响**：纯文档站静态样式重构，不影响主应用、overlay 与发布流程。
