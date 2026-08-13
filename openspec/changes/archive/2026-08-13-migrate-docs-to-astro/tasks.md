# 任务：文档站迁移到 Astro / Starlight

## 1. 脚手架与依赖

- [x] 1.1 重写 `docs/package.json`：删除 `vitepress`、`vitepress-plugin-mermaid`、`vitepress-plugin-llms`、`search-insights`、`mermaid`，新增 `astro`、`@astrojs/starlight`、`starlight-llms-txt`
- [x] 1.2 删除 `docs/.vitepress/` 整个目录
- [x] 1.3 新建 `docs/astro.config.mjs`：配置 `title`、`base: '/'`、`defaultLocale: 'root'`、`locales`（root + zh-cn）、`trailingSlash: 'never'`、`build.format: 'file'`、`lastUpdated: true`、`integrations: [starlight(...), starlightLllmsTxt()]`
- [x] 1.4 新建 `docs/src/` 目录骨架（`content/docs/`、`components/`、`styles/`）

## 2. 内容搬迁（不改正文）

- [x] 2.1 搬运 `docs/guide/*.md` → `docs/src/content/docs/guide/`（16 篇，内容原样）
- [x] 2.2 搬运 `docs/zh-cn/guide/*.md` → `docs/src/content/docs/zh-cn/guide/`（16 篇，内容原样）
- [x] 2.3 改写 `docs/index.md` → `docs/src/content/docs/index.md`：`layout: home` frontmatter 改为 Starlight `template: splash` + `hero`
- [x] 2.4 改写 `docs/zh-cn/index.md` → `docs/src/content/docs/zh-cn/index.md`：同上
- [x] 2.5 确认 `docs/public/`（CNAME、logo.svg、favicon）保留原位且被复制到产物根

## 3. 配置翻译

- [x] 3.1 在 `astro.config.mjs` 中显式写出 en sidebar（16 项，标签与现 config.mts 完全一致，含 `Getting Started`/`Configuration`/`Motion Sickness Relief` 等）
- [x] 3.2 在 `astro.config.mjs` 中显式写出 zh-cn sidebar（16 项）
- [x] 3.3 配置 `social`（GitHub 链接）与 favicon/head 元信息

## 4. 机械替换

- [x] 4.1 10 条带 `.md` 后缀内部链接去后缀：`privacy.md`、`motion-sickness.md`、`contributing.md`、`development.md`（en + zh-cn 各对应位置）
- [x] 4.2 4 处 `::: tip` → `:::tip`
- [x] 4.3 搬运 `docs/.vitepress/theme/custom.css` → `docs/src/styles/custom.css`，品牌变量 `--vp-c-brand-*` 映射为 Starlight `--sl-color-*`（保持 #2563EB）
- [x] 4.4 在 `astro.config.mjs` 注册 `customCss` 指向 `src/styles/custom.css`

## 5. 首页特性卡片

- [x] 5.1 新建 `docs/src/components/FeatureCards.astro`：渲染 6 张特性卡片，复用品牌蓝
- [x] 5.2 将 `FeatureCards` 接入首页 hero 下方

## 6. 本地构建验证

- [x] 6.1 `cd docs && npm install`
- [x] 6.2 `npm run build` 成功，产物在 `docs/dist`
- [x] 6.3 对比迁移前后 URL 集合（`find dist -name '*.html'`），确认 34 个页面一一对应
- [x] 6.4 验证 `/llms.txt` 与 `/llms-full.txt` 产物存在，且不含 `Untitled`、`manual-test-checklist`、`v0.2.1-release-test-plan`
- [x] 6.5 验证 sitemap 产物存在
- [x] 6.6 验证 zh-cn 首页产物路径（`/zh-cn/` 目录形态是否保持，见设计 Open Questions）

## 7. CI 改造

- [x] 7.1 更新 `.github/workflows/pages.yml`：`actions/checkout` 加 `fetch-depth: 0`
- [x] 7.2 更新 build 命令 `docs:build` → `build`
- [x] 7.3 更新产物路径 `docs/.vitepress/dist` → `docs/dist`

## 8. 线上回归（部署后）

- [x] 8.1 `/guide/usage` 与 `/guide/usage.html` 均返回 200
- [x] 8.2 `/zh-cn/` 返回 200，`/zh-cn` 301 到 `/zh-cn/`
- [x] 8.3 页面 `Last updated` 显示非空日期
- [x] 8.4 `/llms.txt` 干净（无幽灵条目、无 QA 文档）
- [x] 8.5 `/sitemap-index.xml` 可访问
- [x] 8.6 搜索命中英文与中文关键词
- [x] 8.7 6 条 `#...` 锚点链接（含 `#动态输入-api`、`#7-配置文件`）逐一可跳转
