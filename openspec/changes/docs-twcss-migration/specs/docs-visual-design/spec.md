# docs-visual-design Delta

## ADDED Requirements

### Requirement: 自定义组件样式 Tailwind 工具类优先

文档站自定义组件（`Header.astro` 与 `src/components/landing/` 下组件）的样式 MUST 优先以 Tailwind v4 工具类实现；手写 scoped CSS MUST 仅出现在豁免清单内：Starlight 默认组件的逐字复制区、由客户端脚本切换的 JS 状态钩子类、以及必须依赖 `:global` / Starlight 内部选择器 / 媒体查询的全局润色规则。迁移 MUST 与显式枚举的现代化项同步交付，且现代化项之外的样式计算值保持不变，`npm run verify` 全绿为验收门禁。

#### Scenario: 组件样式以工具类表达

- **WHEN** 检查任一已迁移自定义组件的模板
- **THEN** 布局、颜色、字号、间距等样式以 Tailwind 工具类表达，`<style>` 块为空或仅含豁免清单内容

#### Scenario: 未触及部分计算样式不变

- **WHEN** 任一组件完成迁移并执行 `npm run build && npm run verify`
- **THEN** 除显式枚举的现代化项外，Playwright 计算样式断言全部通过，页面视觉与迁移前一致

#### Scenario: 现代化项可追踪

- **WHEN** 检查任一迁移 commit 的 message 与对应 verify 断言
- **THEN** 视觉变更点逐项列出，且断言已更新为现代化后的预期样式

#### Scenario: 断点语义与 Starlight 对齐

- **WHEN** 检查 `src/styles/global.css` 的 `@theme` 块
- **THEN** Tailwind 断点与 Starlight 断点语义对齐（`md` = 50rem），`hidden md:flex` 与 `sl-hidden md:sl-flex` 可互换使用

#### Scenario: Header 复制区豁免

- **WHEN** 检查 `src/components/Header.astro`
- **THEN** 逐字复制自 Starlight 默认 Header 的结构与 CSS 保持原样（保留升级 diff 工作流），仅本站新增的导航区样式使用 Tailwind 工具类

#### Scenario: Starlight token 引用单源

- **WHEN** 检查已迁移组件中对 `--sl-color-*` 变量的引用方式
- **THEN** 通过 Tailwind 工具类（含 v4 任意值变量简写）引用，无新增手写色值或重复 token 定义

#### Scenario: 产品家族风格统一可感知

- **WHEN** 检查落地页区块（FeatureGrid / HowItWorks / DownloadCta）并与组织站（aukcraft.org）对照
- **THEN** 家族签名元素两站一致：各区块以共享 `SectionHeading` 组件开篇（序号 + micro mono 标签 + hairline 贯穿线）、特性卡片采用 hairline 分隔网格、标签类文本以 mono + uppercase + 宽字距工具类表达、区块接入 `IntersectionObserver` 滚入揭示（reduced-motion 全量塌缩）、垂直节奏为大留白单列叙事；产品个性保持独立——品牌蓝 + zinc 双主题与 Hero 左文右图构图不随组织站改变；一致性完全由视觉语言表达，无新增归属声明类文案或标识
