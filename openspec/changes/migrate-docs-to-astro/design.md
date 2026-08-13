# 设计：文档站迁移到 Astro / Starlight

## Context

当前文档站位于仓库根目录 `docs/`，基于 VitePress 1.5 构建，部署在 `peregrine.aukcraft.org`（Cloudflare CDN 前置 GitHub Pages）。实测现状（2026-08-13）：

- **内容**：34 篇 Markdown（16 en guide + 16 zh-cn guide + 2 首页），约 280KB，纯标准 Markdown（GFM 表格 30 处、代码块 bash/rhai/json/jsonc/ts、`::: tip` 4 处）。
- **URL 形态**：VitePress 以 `cleanUrls: false` 产出 `guide/usage.html`，但 GitHub Pages 的 pretty-URL 特性让 `/guide/usage`（干净）与 `/guide/usage.html` 两种地址都返回 200。zh-cn 首页 `/zh-cn/`（目录斜杠形态），`/zh-cn` 会 301 到 `/zh-cn/`。
- **配置**：`docs/.vitepress/config.mts` 133 行，`locales`（root=en + zh-cn）nav/sidebar 全量重复。
- **主题**：`docs/.vitepress/theme/index.ts`（7 行）+ `custom.css`（品牌蓝 #2563EB 覆盖）。
- **线上 bug**（已确认）：`lastUpdated` 日期为空（CI 缺 git 历史）；`llms.txt` 含 `[Untitled](/zh-cn.md)` 幽灵条目与两个内部 QA 文档；无 sitemap；QA 文档公网可访问。
- **约束**：组织内 `aukcraft/website` 已是裸 Astro 4 + Tailwind 3；文档站点需保持现有 URL、双语、llms.txt。

## Goals / Non-Goals

**Goals:**
- VitePress → Astro + Starlight，内容零改动，URL 保持，双语与视觉不退化。
- 修复 4 个线上 bug 并新增 sitemap。
- 与组织 Astro 工具链 / Node 22 CI 统一。

**Non-Goals:**
- 不动 `aukcraft/website`；不引入 AI 聊天助手；不重构文档内容；不加新站点功能。

## Decisions

### D1. 采用 Starlight 而非裸 Astro

文档站需要 sidebar、搜索、i18n、lastUpdated —— 这些裸 Astro 需全手搓，而 Starlight 原生提供。内容层是纯标准 Markdown，Starlight 的文档模型完全够用。`aukcraft/website`（营销站）才是裸 Astro 的主场，两者共享 Astro 工具链即可。

备选：裸 Astro + content collections（设计自由但成本 2~3 倍）——放弃，理由是对 34 页文档站收益过低。

### D2. 目录结构与 i18n 映射

Starlight 的 i18n 目录结构与 VitePress 同构：

| 现状（VitePress） | 迁移后（Starlight） |
|---|---|
| `docs/index.md` | `src/content/docs/index.md`（en 默认 locale） |
| `docs/guide/*.md` | `src/content/docs/guide/` |
| `docs/zh-cn/index.md` | `src/content/docs/zh-cn/index.md` |
| `docs/zh-cn/guide/*.md` | `src/content/docs/zh-cn/guide/` |
| `docs/public/` | 保持原位（`public/`，CNAME/logo/favicon 原样） |

`astro.config.mjs` 用 `defaultLocale: 'root'` + `locales: { root, 'zh-cn' }`，URL 前缀与现在一致。

### D3. URL 保持：`trailingSlash: 'never'` + `format: 'file'`

关键决策。实测发现 GitHub Pages 对 `.html` 文件同时响应干净 URL 与 `.html` URL：

- Astro `build.format: 'file'` 使 `guide/usage.md` 产出 `guide/usage.html` 文件（与 VitePress 一致）。
- `trailingSlash: 'never'` 使内部链接指向 `/guide/usage`（干净形态，与当前线上干净 URL 一致）。
- 结果：新旧两种 URL 在 GitHub Pages 上继续都返回 200，**无需 301 重定向**。

备选：`trailingSlash: 'always'` + 目录格式 —— 会产出 `/guide/usage/`，改变现有干净 URL 且让 30 条无后缀相对链接断链，放弃。

**遗留验证点**：zh-cn 首页当前是 `/zh-cn/`（目录形态）。Astro `format: 'file'` 可能将 `zh-cn/index.html` 变成 `zh-cn.html`，破坏 `/zh-cn/`。需在实现时验证 Starlight 是否对 locale 根保留目录形态；若否，需额外处理（见 Open Questions）。

### D4. 内部链接重写（精确清单）

实测全站内部链接 40 条，其中 30 条无后缀（`./config` 等）在 `trailingSlash: 'never'` + 相对同目录下天然正确，**仅 10 条带 `.md` 后缀需去后缀**（Astro 不会自动剥 `.md`）：

| 文件（en + zh-cn 各一份） | 链接 |
|---|---|
| `development.md` | `./privacy.md` → `./privacy` |
| `recommendations.md` | `./motion-sickness.md` → `./motion-sickness` |
| `intro.md` | `./contributing.md` → `./contributing`、`./development.md` → `./development` |

### D5. sidebar 显式配置（放弃 autogenerate）

实测 sidebar 标签 ≠ 页面 H1（如 `getting-started` 的 H1 是 "Quick Start"，sidebar 标签是 "Getting Started"）。`autogenerate` 会以 H1 作标签导致退化，故显式写出 16 + 16 项，与现 config.mts 标签逐项对齐。

### D6. 首页 hero + 特性卡片

- hero（name/text/tagline/image/3 个 actions）→ Starlight `template: splash` + `hero` frontmatter，零代码。
- 6 张特性卡片 → Starlight 无内置卡片网格，新增 `src/components/FeatureCards.astro`（~30 行，复用品牌蓝）。

### D7. llms.txt 用 `starlight-llms-txt`

直接替代 `vitepress-plugin-llms`。它只收录 Starlight docs collection，自动排除 `manual-test-checklist.md` / `v0.2.1-release-test-plan.md`，并按 locale 组织，顺带修复幽灵条目与文档泄漏。生成 `/llms.txt` 与 `/llms-full.txt`。

### D8. lastUpdated 与 sitemap

- `lastUpdated: true` 依赖 git 历史，CI 的 `actions/checkout` 必须加 `fetch-depth: 0`（否则日期仍为空，即线上现状）。
- Starlight 自动生成 sitemap（`sitemap-index.xml`），补上当前缺失。

### D9. 语法差异与品牌色

- 4 处 `::: tip`（带空格）→ `:::tip`（无空格），Starlight Asides 语法。
- `custom.css` 的 VitePress 品牌变量（`--vp-c-brand-*`）→ Starlight 变量（`--sl-color-*`），保持 #2563EB 蓝。

### D10. CI 改造（pages.yml）

| 变更 | 内容 |
|---|---|
| checkout | 加 `fetch-depth: 0`（喂 lastUpdated） |
| build 命令 | `docs:build` → `build` |
| 产物路径 | `docs/.vitepress/dist` → `docs/dist` |

## Risks / Trade-offs

- **[zh-cn 首页 URL 形态改变] → 迁移后验证 `/zh-cn/` 与 `/zh-cn` 行为；必要时为 locale 根保留目录形态或补重定向。**
- **[`.html` 后缀 URL 失效] → 实测 GH Pages pretty-URL 已覆盖，但需在部署后回归确认 `/guide/usage.html` 仍 200；若失效则用 Cloudflare 重定向规则兜底。**
- **[中文搜索命中率] → Starlight 的 Pagefind 默认按空格分词，中文弱；迁移后实测中文关键词，必要时自定义分词配置。**
- **[中文锚点 slug 漂移] → 6 条 `#...` 锚点（含 `#动态输入-api`、`#7-配置文件`）需逐一点击验证；两侧均用 github-slugger，理论上一致但需实测。**
- **[CI 首次构建需拉全历史] → `fetch-depth: 0` 增加少量 checkout 时间，可接受。**

## Migration Plan

1. `docs/package.json` 重写依赖；删除 `.vitepress/`；新增 `astro.config.mjs` 与 `src/`。
2. 搬迁 34 篇 Markdown（不改内容），仅首页 frontmatter 改写为 Starlight hero。
3. 执行 10 条链接去后缀、4 处 `:::tip`、品牌色变量映射。
4. 新增 `FeatureCards.astro` 与 hero frontmatter。
5. 本地 `npm run build` 验证产物：URL 集合与迁移前一致、`llms.txt` 干净、sitemap 生成。
6. 更新 `pages.yml`；合并后由 `pages.yml` 部署，线上回归 URL / lastUpdated / 搜索 / llms.txt。
7. 回滚：`pages.yml` 由 Git 历史可回退到 VitePress 版本，`docs/dist` 产物可随时重建。

## Open Questions（实现后确认）

- **zh-cn 首页 `/zh-cn/` 目录形态**：已确认 `format: 'file'` 下内容集合会剥离 `index` slug，`zh-cn/index.mdx` 产出 `zh-cn.html`（非 `zh-cn/index.html`）。`'preserve'` 与 Astro 静态 `redirects` 均无法生成 `zh-cn/index.html`（redirect 会与内容路由 `/zh-cn` 冲突）。**方案**：保留 `format: 'file'`，由 Cloudflare 重定向规则将 `/zh-cn/`（带斜杠）转发到 `/zh-cn`，作为部署侧兜底（见 Risks）。
- **内部 QA 文档**：仅排除、保留源文件。两个 QA 文档（`manual-test-checklist.md`、`v0.2.1-release-test-plan.md`）保留在 `docs/` 根目录，因其不在 `src/content/docs/` 下，构建与 `llms.txt` 均自动排除。

## 实现记录（与设计假设的偏差）

实现过程中发现以下必须的结构性改动，补充记录（不影响线上 URL 行为）：

- **每个页面必须带 `title` frontmatter**：Starlight 的 `docsSchema` 强制要求 `title`，而原 VitePress 页面以首行 `# H1` 作为标题。已为 32 篇 guide 页各补 `title`（取原 H1 文本），并删除原 `# H1` 行以避免标题重复（Starlight 会自动以 `title` 渲染 H1）。
- **新增 `docs/src/content.config.ts`**：Starlight 内容集合需显式注册 `docs` 集合（`docsLoader()` + `docsSchema()`），否则构建报 "collection docs does not exist"。
- **首页改为 `.mdx`**：为在首页 hero 下方引入 `FeatureCards` 组件，`index.md` 需为 MDX 才能 import 组件（`.md` 不支持组件）。URL 不受影响（`index.mdx` 仍产出 `index.html`）。
- **`::: tip X` → `:::tip[X]`**：Starlight 的自定义 aside 标题必须用方括号（`:::tip[标题]`），仅去空格（`:::tip 标题`）无法正确渲染标题。设计 D9 的「仅去空格」已修正为方括号形式。
