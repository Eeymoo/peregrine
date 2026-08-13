## Context

docs 站已完成两轮视觉改造：`docs-modern-redesign`（Tailwind v4 接入 + 品牌 token + 落地页化 + 截图管线）与 `docs-twcss-migration`（组件样式迁移到 Tailwind 工具类）。当前视觉语言仍偏向 Starlight 出厂皮肤的「soft / 圆角 / 阴影」：按钮 `rounded-full`、截图框 `rounded-xl` + `shadow-[...]`、中性色为 zinc 冷灰、浅色主题近纯白。

本次把 `.agents/skills/aukcraft-site-design` 的壳层设计系统落到 docs 站。约束不变：

- Starlight 排版依赖内部 class 与 `--sl-color-*` 变量，任何全局 reset 都会破坏它；主题切换经 `data-theme="dark"|"light"` 属性，与 `prefers-color-scheme` 脱钩，由 `@astrojs/starlight-tailwind` 桥接 `dark:` 变体。
- Starlight 0.41 原生消费 `--color-accent-*` / `--color-gray-*` 两组 token（D3），改这两组即可全站换色。
- 品牌主色锁定 `#2563EB`（blue-600），明暗双主题都保留（用户明确要求），与 aukcraft 技能的「单暗色 + teal」不同，需自研浅色端与蓝系主色的对应关系。
- 设计执行目标：在不脱离 Starlight 的前提下，把 aukcraft 的「零阴影 / ≤4px 圆角 / hairline 分层 / 蓝调中性 / mono 微标签 / serif-italic 强调 / 签名动效组件」落进来。

## Goals / Non-Goals

**Goals:**

- 全站表面语言切换到 aukcraft 壳层：零阴影、≤4px 圆角、hairline 分层、蓝调近黑/近白双主题（非纯黑/纯白）。
- 落地页引入 DotField（点阵动态背景）与 FlightLine（边框描边按钮），替换 pill 按钮；Hero 保留真实截图、引入 serif-italic 标题强调。
- 品牌蓝 `#2563EB` 唯一主色且「节制使用」：仅链接 / CTA / 描边动线 / 焦点环 / 画布光标填色。
- 动效遵守 `prefers-reduced-motion`，签名动效（点阵 / 描边 / reveal）全部塌缩。

**Non-Goals:**

- 不脱离 Starlight：sidebar / 搜索 / 主题开关 / 语言切换 / TOC / llms.txt / sitemap 不变。
- 不改 URL / IA / 正文内容；不引入 HeroCanvas（点阵字标营销 hero）、teal、单暗色。
- 不把 Starlight 自带控件（侧栏 pill、搜索框、主题切换）做 radius/shadow 覆写——换皮深度停在 token + 内容区 polish。
- 不引入 GSAP / Three.js / React 岛；DotField / FlightLine 用原生 canvas / SVG + rAF。

## Decisions

### D1：中性色 ramp 重定义——蓝调灰阶取代 zinc，锚定 aukcraft 的 base/raised

- 做法：`@theme` 中 `--color-gray-50..950` 由 zinc 改为 aukcraft 蓝调灰阶（B 分量 ≥ G ≥ R 的冷蓝灰），暗色端精确锚定 `#0B0E11`（base）/ `#14181D`（raised），浅色端锚定 `#F5F7F9`（base）；`--color-accent-*` 保持不变（蓝）。完整阶表：

  ```
  50  #F5F7F9   浅色 base（页面底色，蓝调近白，非纯白）
  100 #EBEFF2
  200 #DDE3E8
  300 #C6CFD7
  400 #9BA6B1
  500 #7C8894   ≈ 暗色 muted
  600 #5A6472   浅色 muted
  700 #454E5A
  800 #2C333C
  900 #14181D   暗色 raised（表面）
  950 #0B0E11   暗色 base（页面底色，蓝调近黑，非纯黑）
  ```

  aukcraft 的 ink `#EDEDED`（暗色正文）落在 gray-50 附近（Starlight 暗色下正文映射到最亮阶），muted `#8A9199` ≈ gray-500；文字级色值不单独建 token，由 gray 阶承载。
- 理由：Starlight 原生消费 `--color-gray-*` 生成明暗两套背景/边框/文字层次，改这一处即全站换色；aukcraft 只给 4 个锚点，需在两端锚点间插值出完整 11 阶才能喂给 Starlight。
- 备选：直接手写 `--sl-color-*` 映射（绕过 Tailwind token，丢失单一事实源，且 landing 组件无法用 `bg-gray-*` 工具类）——否决。

### D2：浅色端「raised = 纯白」的边界裁定

- 做法：浅色页面底色 `#F5F7F9`（非纯白），抬升面（卡片 / 侧栏 / 代码块 / 截图框）用 `#FFFFFF`；「不要纯白」约束只作用于**大面积页面底色**，不作用于小面积抬升面。
- 理由：aukcraft 的分层核心是「表面亮度差」——暗色下 base `#0B0E11` 之下 raised `#14181D` 更亮；浅色镜像必须让 raised 比 base 更亮，而 base 已是 `#F5F7F9`，唯一更亮档位是纯白。若连抬升面也禁纯白，则浅色只能退化为「同亮度 + 靠 hairline 边框分层」，卡片质感偏平。
- 实现注意：Starlight 浅色默认 `--sl-color-bg`（内容区）= 白、`--sl-color-bg-sidebar` = 更暗灰；本设计**反转**为内容区/页面 = `#F5F7F9`、抬升面 = `#FFFFFF`，实现时以 `--sl-color-*` 语义变量逐一校准而非依赖 gray 阶自动映射。

### D3：圆角体系收紧到 ≤4px（容器 4px / 行内 2px）

- 做法：全站容器级圆角由 0.75rem（12px）收紧为 4px；行内小元素（inline code / kbd / badge）由 0.375rem 收紧为 2px；所有 `rounded-full`（pill 按钮）移除，改为 4px 方形；`rounded-xl`/`rounded-lg`（截图框、图标容器、proxy select）改为 4px。**取代 `docs-twcss-migration` 的 0.75rem 容器圆角决策。**
- 理由：aukcraft 硬规则「圆角 ≤4px 全站」；信息密度高的文档页在极窄圆角下更「编辑」、更「工具」。
- 备选：保留极窄梯度（容器 4px / 行内 2px）已是最保守落地；彻底 0 圆角（直角）在 code block / aside 上观感生硬——不采用。

### D4：阴影移除 → hairline 分层

- 做法：删除 `LandingHero` 截图框的 `shadow-[...]` 与 hover 阴影放大；`starlight-polish.css` 中所有依赖 `--sl-shadow-*` 的容器阴影改为 `1px` hairline 边框（`var(--sl-color-hairline)` 及对应 light 变体）。layering 全靠「表面亮度差（D2）+ hairline 边框」表达。
- 理由：aukcraft 硬规则「零阴影，分层靠表面亮度 + 1px hairline」。
- 风险：Starlight 部分组件的阴影经 `--sl-shadow-sm/md/lg` 变量定义，改变量即可全站去阴影；个别硬编码阴影需在 polish 中定点覆盖（实现时 grep 核对）。

### D5：DotField 仅落地页；砍掉 HeroCanvas

- 做法：新增 `src/components/DotField.astro`（复用 skill 资产，改造为蓝 fill `37,99,235` 替代 teal，light 变体用深点 `rgba(17,21,26,.06)` 空心环），**仅挂载于双语 `index.mdx`**；`download.mdx` 不挂（它是带侧栏的普通 doc 页，与本条理由一致）；HeroCanvas 不引入。
- 主题适配：skill 资产把空心环**预渲染进 baseLayer** 且仅 `resize` 时重建——必须额外监听 `<html data-theme>` 变化（MutationObserver）触发 baseLayer 重建，否则用户切换明暗主题后点阵颜色错误，直到窗口 resize。
- 层位：fixed canvas 的 z 阶经 `--z-field` token 承载，低于 Starlight 内容 / 侧栏 / 顶栏、高于页面底色；`pointer-events-none` 不拦截交互。
- 理由：DotField 是 `position:fixed` 全屏点阵，穿透 Starlight 侧栏/正文会牺牲文档可读性，故只用于落地页；HeroCanvas 是营销化点阵字标 + 唯一循环动画（idle wave），与「真实截图 Hero」冲突且违反「克制动效」目标。
- 备选：DotField 全站（含指南页 / download 页）——牺牲可读性，否决；保留 HeroCanvas 以完整复刻 aukcraft 首页——偏营销、引入循环动画，否决。

### D6：FlightLine 替换 pill 按钮；serif-italic 标题强调

- 做法：新增 `src/components/FlightLine.astro`（复用 skill 资产，描边色改为蓝）。`DownloadCta` 三架构按钮、`DownloadTable` 的「查看更多版本」、Hero 双 CTA 由 pill 按钮改为 `.flight` 方形 hairline 描边按钮（内含 `<FlightLine />`），primary = `text-accent`、secondary = `text-ink`；hover/focus 触发边框描边动效，`prefers-reduced-motion` 下塌缩为瞬时描边常亮。**暗色主题下 primary 的文字与描边动线改用 `accent-400 #60A5FA` 亮阶**——blue-600 在 `#0B0E11` 上对比度 ≈3.7:1，不满足小字（`font-mono text-xs` CTA 标签）WCAG AA 4.5:1；浅色主题保持 blue-600（在 `#F5F7F9` 上 ≈7.6:1）。Hero 标题对英文关键词加 Newsreader serif-italic 强调（如 "Craft the *anchor*."），中文用 `.serif-zh` 系统 serif 栈。
- 理由：FlightLine 是 aukcraft 的 CTA 签名语言；serif-italic 强调是签名排版元素（仅关键词，不随机撒 serif）。

### D7：换皮深度 = token + 内容区 polish + 控件 CSS 级覆写；仍不新增组件覆写

- 做法：`starlight-polish.css` 重写——code block / aside / 表格 / 行内码 / kbd / 引用块全部改为 ≤4px + hairline 边框 + 去阴影；**侧栏 pill、搜索框、主题切换等 Starlight 自带控件也在 customCss 中定点覆写 radius/shadow**（目标：≤4px 圆角、零阴影、hairline 边框），使 aukcraft「≤4px 全站」规则覆盖控件层。
- 边界：只允许 L1（customCss 定点覆写内部 class）；**不新增 `components:` 组件覆写**（L2 维持禁止，Hero / Header 之外不加第三个）——组件覆写要「拥有」DOM 拷贝，每个都是升级 diff 负担，控件换皮用 CSS 足够。
- 理由：用户明确要求控件也纳入壳层换皮；CSS 级覆写是达到该目标的最低风险手段。代价是绑定 Starlight 内部 class 名（官方不承诺稳定），升级 Starlight 时需人工复查这些选择器是否仍命中。
- 维护约定：polish 中每条控件覆写规则必须注释标注目标 Starlight 版本与所覆写的内部 class，升级 Starlight 时按注释清单逐条核对（与 `Header.astro` 的 diff 工作流同级）。

### D8：serif 字体引入（Newsreader）+ 中文系统 serif 栈

- 做法：`package.json` 新增 `@fontsource-variable/newsreader`（wght + italic）；`.serif-italic` / `.serif-zh` 工具类或 `@theme` 字体 token 承载强调。mono 沿用 Starlight 既有 mono 栈（JetBrains Mono 由 Starlight 提供）。
- 中文方案：`.serif-zh` 用**系统 serif 回退栈**（`'Songti SC', 'Noto Serif CJK SC', 'Source Han Serif SC', serif`），**不引入 CJK webfont**——Google Fonts 在国内可达性差，全量 CJK 子集体积数 MB；代价是不同系统字形略有差异，可接受。
- 裁剪防护：italic 强调词若含 descender 字母（`y g j p q`），容器须 `leading-[1.1]` 以上并加 `pb-1` 预留，防止 `leading-none` 切掉下伸部；实现时逐词审计 Hero 标题。
- 理由：serif-italic 强调需要正文字体之外的第二字体；Newsreader 是 aukcraft 家族指定 serif（英文）；中文走系统栈是可达性与体积的权衡。

### D10：`.noise` 胶片颗粒仅落地页

- 做法：新增 `.noise` 样式（2.5% 透明度，`position: fixed` + `pointer-events-none`，z 阶经 token 承载），仅随落地页（双语 `index.mdx`）挂载，与 DotField 同范围；guide 页 / download 页一律不加。
- 理由：fixed 噪声层压在文档正文上会轻微降低文字锐度、干扰截图管线；收益主要在落地页氛围。性能形态（fixed + pointer-events-none 单层）本身合法，问题在于覆盖面。
- 备选：全站生效（proposal 初稿）——可读性代价大于氛围收益，否决。

### D9：verify 断言同步更新

- 做法：`docs/scripts/verify-landing.mjs` / `verify-polish.mjs` 的中性色、圆角、阴影相关 computed-style 断言随新设计更新（如「无 `box-shadow`」「`border-radius ≤ 4px`」「页面底色非纯白/纯黑」）。
- 理由：既有 verify 脚本断言的是上一代设计，不改会 CI 红。

## Risks / Trade-offs

- [Starlight 浅色「反转底色」后，某些内建组件仍硬编码白色背景] → 实现时 grep `#fff` / `white` / `--sl-color-bg`，逐个校准到 `#F5F7F9` / `#FFFFFF` 语义。
- [`--color-gray-*` 全阶替换影响 Starlight 的明暗语义映射，某阶插值不当导致对比度不足] → 两端锚点锁定后逐阶目检明暗双主题；`npm run verify` 增加对比度断言兜底。注意灰阶 300→400 跳变（`#C6CFD7` → `#9BA6B1`，Δ≈43）是全阶最大落差，目检时重点核对 Starlight 消费中间阶的位置（边框 / 次级文字）是否突兀。
- [暗色下 blue-600 文字对比度不足（≈3.7:1 < 4.5:1）] → CTA / 描边动线在暗色主题用 `accent-400` 亮阶（D6），verify 断言兜底。
- [去阴影后 Starlight 某些分层（悬浮菜单、mobile 抽屉）失去纵深] → 用 hairline 边框替代；控件层经 customCss 定点覆写（D7），悬浮类组件可保留极淡阴影作为功能性纵深（hairline 无法表达浮起），实现时逐案裁定并在 polish 注释说明。
- [控件 CSS 覆写绑定 Starlight 内部 class，升级后静默失效] → D7 维护约定：每条控件覆写注释标注目标版本与 class 名，升级时逐条核对；失效的最坏结果是控件回到默认圆角/阴影，不影响功能。
- [DotField 在 `prefers-reduced-motion` 下需塌缩为静态] → 脚本内 `matchMedia('(prefers-reduced-motion: reduce)')` 直接跳过 rAF，只画静态点阵（skill 资产已实现）。
- [Newsreader 字体体积] → 仅引入 variable wght + italic 子集，`@fontsource-variable` 按需分包。

## Migration Plan

1. 中性色 ramp（D1/D2）替换 `@theme`，跑 `npm run build` 验收不破坏 Starlight 排版。
2. 圆角/阴影收紧（D3/D4）+ `starlight-polish.css` 重写（D7），明暗双主题目检。
3. 新增 DotField / FlightLine / `.noise`（D5/D6/D10，均仅落地页），改造 landing 组件与双语 index.mdx。
4. 引入 Newsreader 与 `.serif-zh` 系统栈（D8），Hero 标题 serif-italic（含 descender 裁剪审计）。
5. 更新 verify 断言（D9），`npm run verify` + 明暗/双语/移动视口截图验证。
6. 更新 AGENTS.md "Documentation Site" 一节。

回滚：全部改动限于 `docs/` 与 `AGENTS.md`，revert 即可；无运行时迁移。

## Open Questions

- 浅色「raised = 纯白」是否接受？若最终坚持「连卡片也不纯白」，需回退 D2 为「同亮度 + hairline 分层」方案。
- serif-italic 强调的具体落点（Hero 标题英文关键词 vs 全站标题）以实现时视觉为准，暂定为 Hero 标题一处。
