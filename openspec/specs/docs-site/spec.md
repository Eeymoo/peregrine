# docs-site Specification

## Purpose

定义 Peregrine 文档站（`peregrine.aukcraft.org`）在迁移到 Astro / Starlight 后必须满足的规范级要求：纯静态构建、可部署到 GitHub Pages + Cloudflare、URL 路径保持不变、双语（en / zh-cn）、`llms.txt` / `llms-full.txt` 生成、全文搜索、`lastUpdated` 时间戳、sitemap 生成，以及排除内部 QA 文档。

## Requirements
### Requirement: 静态构建与部署

文档站 MUST 通过 Astro 构建为纯静态站点，产物可部署到 GitHub Pages 并经 Cloudflare 对外服务，不依赖任何服务端运行时。

#### Scenario: 静态构建产物

- **WHEN** 在 `docs/` 目录执行构建命令
- **THEN** 在 `docs/dist` 生成纯静态产物（HTML/CSS/JS/资产），不含服务端代码

#### Scenario: 部署兼容

- **WHEN** 产物上传到 GitHub Pages
- **THEN** 站点可通过 `https://peregrine.aukcraft.org/` 访问，首页返回 HTTP 200

### Requirement: URL 路径保持不变

迁移后站点 MUST 保持现有 URL 路径形态不变，现有内部页面路径与迁移前一致。

#### Scenario: 指南页面路径

- **WHEN** 访问 `/guide/usage`
- **THEN** 返回 HTTP 200 且展示「Usage Guide」页面

#### Scenario: 中文页面路径

- **WHEN** 访问 `/zh-cn/guide/usage`
- **THEN** 返回 HTTP 200 且展示中文「使用说明」页面

#### Scenario: 首页路径

- **WHEN** 访问站点根 `/` 与 `/zh-cn/`
- **THEN** 分别返回英文首页与中文首页，均 HTTP 200

### Requirement: 双语支持

文档站 MUST 支持双语（en 默认 locale 与 zh-cn locale），每种语言拥有完整的导航、sidebar 与页面集合。

#### Scenario: 语言导航

- **WHEN** 用户在英文页面切换语言到「简体中文」
- **THEN** 跳转到对应的 `/zh-cn/...` 路径，sidebar 与导航显示中文标签

#### Scenario: 语言页面完整性

- **WHEN** 遍历两种 locale 的指南页面
- **THEN** en 与 zh-cn 各有 16 篇指南页，一一对应且无缺失

### Requirement: LLM 可读文档生成

文档站 MUST 在站点根生成 `/llms.txt` 与 `/llms-full.txt`，内容为全站 Markdown 索引与完整内容。

#### Scenario: 生成干净索引

- **WHEN** 构建完成后访问 `/llms.txt`
- **THEN** 返回 HTTP 200，索引中不包含 `[Untitled]` 幽灵条目

#### Scenario: 排除内部文档

- **WHEN** 检查 `/llms.txt` 与 `/llms-full.txt` 内容
- **THEN** 不包含 `manual-test-checklist` 与 `v0.2.1-release-test-plan` 相关条目

### Requirement: 全文搜索

文档站 MUST 提供站内全文搜索能力，支持英文与中文关键词检索。

#### Scenario: 搜索命中

- **WHEN** 用户在搜索框输入一个存在于文档中的英文关键词（如 `telemetry`）
- **THEN** 返回相关页面结果列表，且点击可跳转

### Requirement: 最后更新时间

文档站 MUST 在页面显示基于 git 提交历史的最后更新时间，且值非空。

#### Scenario: 时间戳非空

- **WHEN** 打开任一指南页面
- **THEN** 页面底部「Last updated」后显示具体日期，而非空值

### Requirement: sitemap 生成

文档站 MUST 在构建时生成 sitemap，供搜索引擎索引。

#### Scenario: sitemap 可访问

- **WHEN** 访问 `/sitemap-index.xml` 或等价的 sitemap 入口
- **THEN** 返回 HTTP 200 且包含全站页面 URL 列表

