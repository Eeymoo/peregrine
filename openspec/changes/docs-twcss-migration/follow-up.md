# 后续任务登记（不属于本 change 范围）

> 本文件登记实施过程中发现、但按 proposal「非目标」明确排除的事项，供后续 change 认领。

## 1. 英文文案 em-dash 违规（taste-skill 9.G）

taste-skill pre-flight 机械检查确认，英文文案存在 2 处 em-dash（`—`）违规
（design.md /tasks.md 原记录为「3 处」，实际复核为 2 处，第 3 处为同一
description 字段在页面 head 与正文摘要的重复渲染）：

- `docs/src/content/docs/download.mdx:3`（frontmatter `description`）
- `docs/src/content/docs/download.mdx:15`（正文）

处理约定（design.md D7 已固化）：英文 `—` 违规需改写；中文 `——` 是规范
中文标点，**保留**，pre-flight 不应对中文文案误报。

## 2. 无其他遗留

迁移涉及的 6 个组件与 starlight-polish.css 均已按豁免清单收敛，无已知
视觉回归（`npm run verify` 双主题双语全绿 + pre-flight 抽查复核）。
