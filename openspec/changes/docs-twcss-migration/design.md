# Design: docs-twcss-migration

## Context

文档站（`docs/`，Astro 7 + Starlight 0.41）已按官方配方接入 Tailwind CSS v4：`global.css` 声明层序 `base, starlight, theme, components, utilities`，品牌 token 单源在 `@theme`（accent = 品牌蓝 `#2563EB` 色阶，gray = zinc）。但 6 个自定义组件仍用手写 scoped CSS（`Header.astro` ~90 行、`LandingHero` 132、`DownloadTable` 145、`DownloadCta` 77、`FeatureGrid` 73、`HowItWorks` 45），另有 `starlight-polish.css` 173 行全局润色。同一站点两套样式体系并存。

已核实的关键事实（来自 `node_modules` 源码，非推测）：

- 兼容包 `@astrojs/starlight-tailwind@5.0.0` 的 `tailwind.css` 给出了 `--sl-color-*` ↔ `--color-*` 的**精确映射**（暗色默认、亮色 `[data-theme='light']` 翻转），且提供 `@custom-variant dark` 桥接 Starlight 主题开关。
- 兼容包**未对齐断点**：Starlight `md` = 50rem（`starlight/style/util.css`），Tailwind 默认 `md` = 48rem。
- 当前 docs 自建代码中 Tailwind 断点工具类使用量为 **0**（仅 Starlight 自带 `md:sl-flex`），对齐断点无存量影响。
- 层序中 `utilities` 在 `starlight` 之后，工具类天然能覆盖 Starlight 内建样式，不存在优先级回退问题。

## Goals / Non-Goals

**Goals:**

- 自定义组件样式统一以 Tailwind 工具类表达，手写 CSS 收敛到明确豁免清单。
- 在现有设计语言内做克制的视觉现代化（详见 D9），站点交互反馈与细节明显更现代。
- 验收：`npm run build` 通过 + `npm run verify` 全绿（现代化项断言同步更新，其余断言不变）。
- 沉淀可复用规约：断点对齐、`--sl-color-*` token 映射表、豁免清单、现代化边界。

**Non-Goals:**

- 整体重新设计：品牌色、信息架构、页面结构、双语内容、Hero 构图均保留；`MOTION_INTENSITY` 上限 4-5。
- 文案修正。taste-skill pre-flight 审出的文案问题（英文 em-dash 等）另起任务。
- Starlight 默认组件覆写结构的调整。
- 主应用 `src/` 的样式（已 ~99% twcss，剩余为真动态 inline 值）。

## Decisions

### D1：验收标准 = 迁移正确 + 现代化项生效

本 change 是「迁移 + 克制现代化」，不是纯机械重构。每个组件完成后必须：`npm run build` 通过 + `npm run verify` 全绿。断言的处理分两类：

1. **未被现代化项触及的样式**——断言保持不变，必须全绿（这部分等价于「计算样式不变」）；
2. **现代化项涉及的样式**——断言同步更新为新预期值，且 commit message 逐项列出视觉变更点。

verify 脚本未覆盖的迁移面（如 Header 导航区），先补断言再动手（断言先行，相当于给迁移上锁）。

### D2：迁移顺序 = 风险递进

`HowItWorks`（45 行，无断点/token 坑，先行趟通「断言 → 迁移 → verify」流程）→ `Header`（仅自定义区）→ `FeatureGrid` → `DownloadCta` → `LandingHero`（keyframes 迁移）→ `DownloadTable`（JS 状态类，最复杂）→ `starlight-polish.css`（部分）。

### D3：断点对齐 = `@theme` 声明 `--breakpoint-md: 50rem`

备选：A) `@theme` 对齐；B) 使用点写 `min-[50rem]:` 任意值。选 A：当前 docs 无任何 Tailwind 断点工具类存量，对齐零风险；B 会让每个断点使用点都冗长且易写错。`lg`（64rem）与 Starlight 一致，无需改。对齐后 `hidden md:flex` 与 `sl-hidden md:sl-flex` 语义等价，可互换。

### D4：`--sl-color-*` → 工具类映射表（以兼容包源码为准）

兼容包给出暗色默认 / 亮色翻转的映射，因此 Tailwind 侧写成「base = 亮色值 + `dark:` = 暗色值」的成对工具类：

| Starlight 变量 | Tailwind 工具类对 |
| --- | --- |
| `--sl-color-white` | `text-gray-900 dark:text-white` |
| `--sl-color-gray-1` | `text-gray-800 dark:text-gray-200` |
| `--sl-color-gray-2` | `text-gray-700 dark:text-gray-300` |
| `--sl-color-gray-3` | `text-gray-500 dark:text-gray-400` |
| `--sl-color-gray-4` | `text-gray-400 dark:text-gray-600` |
| `--sl-color-gray-5` | `text-gray-300 dark:text-gray-700` |
| `--sl-color-gray-6` | `text-gray-200 dark:text-gray-800` |
| `--sl-color-black` | `text-white dark:text-gray-900` |
| `--sl-color-text-accent` | `text-accent-600 dark:text-accent-200`（Starlight core：亮色取 `--sl-color-accent`，暗色取 `accent-high`） |

兼容包未覆盖的 Starlight core 变量（`--sl-color-hairline`、`--sl-color-bg-sidebar` 等）：用 v4 任意值简写直接引用变量（如 `border-(--sl-color-hairline)`），保持 token 单源，不在 `@theme` 重复定义。每条映射落地后必须经计算样式对比复核（唯一权威是渲染结果）。

### D5：豁免清单（以下内容保留手写 CSS）

1. **`Header.astro` 的 Starlight 复制区**：`.header` grid 布局（`--__sidebar-width` / `--__main-column-fr` calc）、`.title-wrapper`、`.right-group`、`.social-icons` 逐字复制自 Starlight 默认 Header，AGENTS.md 要求升级时对照 diff——转成任意值工具类会破坏 diff 工作流且不可读。仅迁移本站新增的 `.header-nav` / `.header-nav-link` / `.header-nav-icon`。
2. **JS 状态钩子类**：`DownloadTable` 的 `is-active` 等由客户端脚本切换的类保留为语义钩子，其**静态**样式仍迁为工具类（`is-active` 态样式可用 `&.is-active` 保留少量 CSS 或 `aria-pressed` 属性选择器，实施时以最小 CSS 残留为准）。
3. **`starlight-polish.css` 中的 Starlight 内部选择器润色**：级联层声明、`:global` / `.sl-markdown-content` 选择器必须留在 CSS 中，不属于迁移范围。

### D6：动效 keyframes 迁移为 `@theme --animate-*` token

`LandingHero` 的 `lh-rise` 转为 `global.css` 的 `@theme { --animate-rise: rise ...; @keyframes rise { ... } }`，组件侧用 `animate-rise` + 任意值 delay（`[animation-delay:80ms]`）。`prefers-reduced-motion` 塌缩逻辑保留少量 CSS（媒体查询无工具类等价物，属豁免）。

### D7：视觉变更必须显式枚举，文案变更不夹带

taste-skill（design-taste-frontend）是本 change 的现代化指南（Section 11「Redesign - Preserve / 定向演进」模式）：每个 commit 允许包含视觉变更，但**仅限 design.md D9 与对应任务中显式枚举的现代化项**，禁止顺手改未列出的东西；pre-flight 清单作为每个组件交付前的检查门禁。文案问题单独记为后续任务：英文文案的 `—`（download.mdx，3 处）按 9.G 属违规；中文 `——` 是规范中文标点，**保留**——此条作为正式决策固化，避免后续 pre-flight 对中文文案误报。

### D9：现代化边界（dials 与逐项清单）

现代化在现有设计语言内演进，dial 读数：`DESIGN_VARIANCE: 6`（不变，沿用现有非对称构图）、`MOTION_INTENSITY: 3 → 4-5`（新增克制的交互动效，全部 reduced-motion 塌缩）、`VISUAL_DENSITY: 3`（不变）。杠杆顺序按 taste-skill 11.D：排版/细节 → 交互反馈 → 动效层，不做 Hero 重组与布局替换。硬约束：单 accent（品牌蓝）锁死、圆角体系统一、无 em-dash 新增、无装饰性状态点、无 scroll cue。

**Header 导航区现代化项（首个实施，枚举如下）：**

1. **当前页 active 态**：Docs / Download 链接在对应路由下显示 active 指示（accent 色 + 下划线或等价处理；现无任何 active 反馈）；
2. **hover 反馈升级**：由瞬时变色升级为 200ms 过渡 + 下划线 scaleX 动画（或淡色 pill 底，实施时二选一并以 verify 断言锁定）；
3. **`focus-visible` 焦点环**：键盘可达性补齐（现无自定义焦点样式）。

**其余组件候选方向**（做到该组件时枚举确认，不在本 design 锁定）：FeatureGrid 图标品牌色淡染容器 + 卡片 hover 微反馈、HowItWorks 步骤连接线索、CTA `:active` 下压触感、LandingHero 截图框 hover 微浮起。

### D8：双语一致

index.mdx / download.mdx 双语页面共用同一组件，迁移天然双语同步，无需分语言处理；验收时两个 locale 页面都要过 verify。

### D10：产品家族设计语言统一（aukcraft design language）

经对 `github.com/aukcraft/website`（组织站，Astro + Tailwind v3）逐组件审查，将两站关系定义为**同一产品家族**：家族感由共享版式语法承载，产品个性由品牌色与站型承载。落地页须让用户一眼识别「这是 aukcraft 系列的产品」。

**家族签名元素（两站必须一致）：**

1. `SectionHeading` 复用组件——`01 ─ LABEL` 序号 + micro mono 标签 + hairline 贯穿线，落地页各区块统一以其开篇（文案沿用各区块现有标题，MDX props 传入，不改信息架构）；
2. mono 微标签体系——`font-mono` + uppercase + 宽字距（tracking 0.15em 量级），用于 eyebrow / 序号 / CTA label 等标签类文本；
3. hairline 分隔网格——`grid gap-px` + hairline 底色衬出分隔线（FeatureGrid 卡片采用）；
4. 滚入揭示——`IntersectionObserver` + `[data-reveal]`，600ms ease-out rise + 逐子级 70ms 错峰；`html.js` 门控保证无 JS 时可读；reduced-motion 全量塌缩；
5. 大留白单列叙事节奏——区块容器 `max-w-5xl` 量级居中，垂直段距对齐组织站的大留白尺度（`py-40 md:py-56` 量级，落地页区块按 Starlight 内容流等比收敛）。

**产品线内产品个性（允许不同，不跟）：** 品牌色（aukcraft teal vs Peregrine 品牌蓝 + zinc 双主题）、主题模式（深色单主题 vs 双主题）、站型与 Hero 构图（组织叙事长页 vs 产品文档站左文右图）、背景装饰与重动效（DotField / noise / Flight Line / scroll-snap / 玻璃拟态 / 衬线斜体关键词均不引入）。

**表达边界（硬约束）：** 家族一致性完全由视觉语言承载——不新增任何归属声明类文案或视觉标识（无 "by aukcraft" 角标、无联名标语、无交叉推广横幅），现有文案内容一字不动；SectionHeading 的标签文案沿用各区块既有标题。

**实现约束**：全部以 Tailwind 工具类表达（本 change 的既有约束不变）；`IntersectionObserver` 脚本为落地页唯一新增 JS，内联于组件级片段，不引入依赖。

## Risks / Trade-offs

- [断点对齐（D3）影响面估计错误，存在未发现的 Tailwind 断点使用] → 迁移前 `rg '(sm|md|lg|xl|2xl):' docs/src` 复核为空；verify 双主题全量跑。
- [`--sl-color-*` 映射表（D4）在个别变量上与 Starlight core 定义有出入（如 hairline 派生链）] → 每组件迁移时对该组件实际用到的变量逐一做计算样式对比，不以映射表代替验证。
- [工具类覆盖 Starlight 样式的优先级过强，未来 Starlight 升级后样式被意外压制] → 层序是官方配方的一部分；豁免清单（D5）本身就是对冲。
- [verify 脚本对 Header 区域无断言，迁移可能引入肉眼难查的偏差] → D1 要求断言先行。
- [`.astro` scoped `<style>` 删除后类名失去组件作用域，全局工具类可能与 Starlight 类名碰撞] → Tailwind 工具类无前缀冲突面；自定义钩子类（`header-nav` 等）保留原名。

## Migration Plan

按 D2 顺序逐组件进行，每个组件一个独立 commit（断言先行 + 迁移 + verify 全绿），任一环节失败即回退该 commit，不影响已完成的组件。全部完成后更新 AGENTS.md 文档站小节（迁移状态、豁免清单、D3/D4 规约）。无线上发布风险：纯静态样式重构，pages.yml 下次构建自然生效。

## Open Questions

- `starlight-polish.css` 中哪些规则可归入工具类、哪些必须豁免，需在做到第 7 步时逐条判定（当前预估大部分豁免）。
- `DownloadTable` 的 `is-active` 态样式最终形态（工具类 + 属性选择器 vs 少量残留 CSS），做到第 6 步时定。
