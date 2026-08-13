## 1. 设计 token 与中性色 ramp

- [x] 1.1 在 `docs/src/styles/global.css` 的 `@theme` 中把 `--color-gray-50..950` 替换为蓝调灰阶（锚定 `#0B0E11`/`#14181D`/`#F5F7F9`，D1 表），保留 `--color-accent-*` 蓝阶与 `--color-brand-*` 别名
- [x] 1.2 校准 Starlight `--sl-color-*` 语义映射（D2）：浅色页面底色 `#F5F7F9`、抬升面 `#FFFFFF`，暗色 base `#0B0E11` / raised `#14181D`；grep `#fff`/`white` 定点清理硬编码白
- [x] 1.3 `cd docs && npm run build` 验收：Starlight 排版不破坏、明暗双主题背景/边框/文字层次正确；目检灰阶 300→400 跳变（Δ≈43，全阶最大）在 Starlight 中间阶消费位置（边框 / 次级文字）是否突兀

## 2. 表面语言：零阴影 + ≤4px 圆角 + hairline

- [x] 2.1 新增 hairline token（暗 `rgba(255,255,255,.08)` / 浅 `rgba(15,23,42,.08)`）并映射到 `--sl-color-hairline*`
- [x] 2.2 移除 `LandingHero.astro` 截图框的 `shadow-[...]` 与 hover 阴影放大，改为 hairline 边框
- [x] 2.3 全站 `rounded-full`（pill 按钮）→ 4px 方形；`rounded-xl`/`rounded-lg` → 4px；行内 `rounded-md` → 2px
- [x] 2.4 覆盖 Starlight 内容区阴影（`--sl-shadow-*` 相关变量），落地页 + 内容区零阴影

## 3. 指南页 polish 重写

- [x] 3.1 重写 `docs/src/styles/starlight-polish.css`：code block / aside / 表格 / 行内码 / kbd / 引用块 → ≤4px 圆角 + hairline 边框 + 去阴影（D7）
- [x] 3.2 在 `starlight-polish.css` 中定点覆写 Starlight 控件（侧栏 pill / 搜索框 / 主题切换）的 radius/shadow → ≤4px + 零阴影；每条覆写注释标注目标 Starlight 版本与内部 class 名（升级复查清单，D7 维护约定）
- [x] 3.3 明暗双主题下目检任一 guide 页，确认内容区与控件均零阴影、≤4px、对比度可读

## 4. 落地页签名组件

- [x] 4.1 新增 `docs/src/components/DotField.astro`：复用 skill 资产，teal 填色改蓝 `37,99,235`，暗/浅两种空心环变体，reduced-motion 下静态；监听 `<html data-theme>` 变化（MutationObserver）重建 baseLayer（主题切换后点阵颜色立即正确，不等 resize）；canvas z 层位经 `--z-field` token 低于 Starlight 内容/侧栏
- [x] 4.2 新增 `docs/src/components/FlightLine.astro`：复用 skill 资产，描边色改蓝，`prefers-reduced-motion` 塌缩
- [x] 4.3 新增 `.flight` / `.flight-line` / `.link-line` / `.micro` / `.serif-italic` / `.serif-zh` 等基元样式到 `global.css`（沿用 aukcraft skill 的 global.css 规则，去除 teal 引用），并补齐支撑 token：`--z-field` z 阶与 `--motion-line-btn` / `--ease-lock` / `--fl-duration` 动效 token
- [x] 4.4 新增 `.noise` 胶片颗粒 overlay 样式（2.5% 透明度，fixed + pointer-events-none，z 阶经 token），仅经落地页挂载（D10）

## 5. landing 组件改造

- [x] 5.1 改造 `DownloadCta.astro`：三架构 pill 按钮 → `.flight` 方形描边按钮（含 `<FlightLine />`，primary `text-accent` / secondary `text-ink`）；暗色主题下 primary 文字与描边动线改用 `accent-400` 亮阶（blue-600 在 `#0B0E11` 上对比度 ≈3.7:1，不足 4.5:1）
- [x] 5.2 改造 `DownloadTable.astro`：筛选/通道 pill 与「查看更多版本」按钮 → 方形描边，表格/控件圆角 ≤4px
- [x] 5.3 改造 `LandingHero.astro`：标题引入 serif-italic 关键词强调；截图框零阴影 hairline（配合 2.2）

## 6. 字体与标题强调

- [x] 6.1 `docs/package.json` 新增 `@fontsource-variable/newsreader` 并 `npm install`
- [x] 6.2 引入 Newsreader italic（`.serif-italic`）与 `.serif-zh` 系统 serif 栈（`'Songti SC', 'Noto Serif CJK SC', 'Source Han Serif SC', serif`，不引 CJK webfont）；Hero 标题英文关键词用 Newsreader italic、中文用 `.serif-zh`；italic 强调词若含 descender 字母（y/g/j/p/q），容器 `leading-[1.1]` 以上 + `pb-1` 防裁剪，逐词审计

## 7. 双语 MDX 挂载

- [x] 7.1 在 `docs/src/content/docs/index.mdx` 与 `zh-cn/index.mdx` body 挂载 `<DotField />` 与 `.noise` overlay（或经 Layout 注入），保持三区块结构不变；download 页不挂（D5/D10）

## 8. verify 断言更新

- [x] 8.1 更新 `docs/scripts/verify-landing.mjs`：新增「无 box-shadow」「圆角 ≤4px」「页面底色非纯白/纯黑」「DotField 存在」「暗色 CTA 文字对比度 ≥4.5:1」断言
- [x] 8.2 更新 `docs/scripts/verify-polish.mjs`：内容区 code block / aside / 表格零阴影 + ≤4px 断言
- [x] 8.3 `cd docs && npm run build && npm run verify` 全绿

## 9. 收尾

- [x] 9.1 明暗/双语/移动视口截图自检，确认零阴影、蓝调中性、描边动效与 reduced-motion 塌缩
- [x] 9.2 更新 `AGENTS.md` "Documentation Site" 一节的圆角/阴影/中性色描述
