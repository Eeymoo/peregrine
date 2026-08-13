# 迁移文档站到 Astro / Starlight

> **跟踪 issue：#51**（https://github.com/Eeymoo/peregrine/issues/51）

## Why

现有文档站（`docs/`）基于 VitePress 构建，但存在多个存量问题：`lastUpdated` 时间戳线上为空、`llms.txt` 含有幽灵条目与泄漏的内部 QA 文档、缺少 sitemap、依赖里塞了三个从未使用的插件（mermaid ×2、search-insights）。同时组织的自定义站点 `aukcraft/website` 已经采用裸 Astro + Tailwind，文档站继续保留 VitePress 会在组织内制造两套工具链。将文档站迁移到 Astro（Starlight 主题）可以在统一工具链的同时顺带修复上述问题。

## 目标 / Goals

- 用 Astro + Starlight 替换 VitePress，文档内容层零改动。
- 迁移后线上 URL 路径保持不变（`/guide/*`、`/zh-cn/guide/*` 等）。
- 保留并修复 `llms.txt` / `llms-full.txt`（移除幽灵条目与泄漏文档）。
- 顺带修复 4 个线上 bug：空 `lastUpdated`、脏 `llms.txt`、内部 QA 文档公网泄漏、缺 sitemap。
- 双语（en 根 locale + zh-cn）导航、sidebar、首页 hero 视觉不退化。
- 与 `aukcraft/website` 共享 Astro 工具链与 Node 22 CI 心智。

## 非目标 / Non-Goals

- 不迁移 `aukcraft/website`（保持裸 Astro + Tailwind 不变）。
- 不在文档站内引入 AI 聊天助手 / RAG 问答（本次只保留 llms.txt 级别的 AI 可读性）。
- 不重构文档内容本身（不重写、不重新组织现有 34 篇 Markdown）。
- 不引入新的站点功能（博客、评论等）。

## What Changes

- 删除 VitePress 及其全部依赖（含未使用的 `vitepress-plugin-mermaid`、`search-insights`）。
- 引入 `astro` + `@astrojs/starlight` + `starlight-llms-txt`。
- 将 34 篇 Markdown 从 VitePress 目录结构搬运到 Starlight 的 `src/content/docs/`。
- 将 `docs/.vitepress/config.mts` 翻译为 `astro.config.mjs`（含 locales、sidebar、`trailingSlash`）。
- 首页 `layout: home` hero 改写为 Starlight hero frontmatter，6 张特性卡片改为自定义组件。
- 修正 10 条带 `.md` 后缀的内部链接与 4 处 `::: tip` 语法差异。
- 更新 `pages.yml` CI（`fetch-depth: 0`、build 命令、产物路径）。
- **BREAKING**：部署产物路径从 `docs/.vitepress/dist` 变为 `docs/dist`（仅影响 CI 配置）。

## Capabilities

### New Capabilities

- `docs-site`: 文档站能力。定义迁移后文档站必须满足的规范级要求——静态构建、可部署到 GitHub Pages + Cloudflare、URL 路径保持、双语（en / zh-cn）、`llms.txt` / `llms-full.txt` 生成、全文搜索、`lastUpdated` 时间戳、sitemap 生成、以及排除内部 QA 文档。

### Modified Capabilities

<!-- 无现有 spec 的能力需求发生变化，此处留空。 -->

## Impact

- **受影响目录**：`docs/`（package.json、`.vitepress/` 删除、新增 `astro.config.mjs`、`src/`）。
- **受影响工作流**：`.github/workflows/pages.yml`（Node 22 部署）。
- **依赖**：删除 `vitepress`、`vitepress-plugin-mermaid`、`search-insights`；新增 `astro`、`@astrojs/starlight`、`starlight-llms-txt`。
- **线上行为**：URL 路径保持不变；新增 sitemap；修复 `lastUpdated` 与 `llms.txt`；隐藏内部 QA 文档。
- **不涉及**：Rust 后端、Tauri 应用、`src/` 前端、`crates/`。
