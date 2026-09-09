> **跟踪 issue：#80**（https://github.com/Eeymoo/peregrine/issues/80）

## Why

设置窗口已演化出 7 个 Tab（通用 / 覆盖层 / 物料 / 快捷键 / 更新 / 关于 / 开发），配置文件新增了全局 `settings` 块与多图层 + 物料运行时，但文档站完全没有对应的使用说明：`guide/config.md` 的 JSON 示例仍是旧版单 `crosshair` 格式（缺 `settings`、缺 `layers`），全站只有一张截图（`settings-layers.png`）。用户照旧文档手写配置会产出被迁移逻辑改写的文件，"物料设置"（动态开关 + FPS 档位 + 两层与门语义）更是无从知晓。

## What Changes

- 新增文档页 `guide/settings.md`「设置详解」（zh + en 双语镜像）：
  - 设置窗口总览（Tab 结构索引图）
  - **通用设置**逐项说明（语言 / 开始覆盖自动切换 / GPU 加速 / 遥测 / CN 镜像 / 快捷颜色）
  - **覆盖层设置**（渲染后端 CPU vs SVG 对比表、抗锯齿、拖拽实时预览）
  - **物料设置**（重点：动态物料两层与门示意图、FPS 节拍语义、静态 / 动态行为差异）
  - **快捷键设置**（动作 ↔ 键位表、录制交互）
  - **更新设置**细微介绍（stable / prerelease 通道 + 镜像加速，一两段）
  - **关于 / 开发者模式：忽略不写**（明确非目标）
- 扩展截图管线 `docs/scripts/capture-screenshots.mjs`：逐 Tab 点击切换，产出真实 UI 截图 `settings-general.png` / `settings-overlay.png` / `settings-material.png` / `settings-hotkeys.png`（入 `docs` 静态资源目录），双语页面共用
- 翻新 `guide/config.md`（zh + en）：
  - JSON 示例升级为新格式（`settings` + `layers` 优先）
  - 新增 `AppSettings` 字段表（15 字段逐一说明）
  - 新增 `MaterialSettings` 字段表
  - 旧 `crosshair` 单格式降级为「遗留兼容」章节并补迁移说明
- 文档内嵌 Mermaid / ASCII 示意图：物料两层与门逻辑图、配置结构图（截图之外的逻辑语义表达）

## 目标

- 用户能仅凭文档站完成设置窗口每个 Tab 的每一项设置的理解与操作（关于 / 开发者除外）
- 用户手写 `config.json` 时有与 `schema.rs` 当前实现一致的字段参考（含 `settings` 全局块）
- 每个设置 Tab 至少配一张真实截图（管线可重复执行）

## 非目标

- 不记录「关于」Tab 与「开发者模式」（用户明确要求忽略）
- 「更新」设置只做细微介绍，不展开更新器内部机制
- 不修改任何 Rust / 前端应用代码（纯文档 + 截图管线脚本变更）
- 不覆盖 `layers.md` / `material-scripting.md` 已有内容，只互相链接

## Capabilities

### New Capabilities

- `settings-guide`: 文档站「设置详解」页——设置窗口各 Tab（通用 / 覆盖层 / 物料 / 快捷键 / 更新）的逐项使用说明、真实截图与逻辑示意图的内容要求

### Modified Capabilities

- `docs-site`: 新增 guide/settings.md 双语页面 + 侧边栏导航项；config.md 内容翻新为新格式优先（`settings` 块 / `layers` 优先，旧 `crosshair` 降级为遗留章节）+ 新增 AppSettings / MaterialSettings 字段表

## Impact

- `docs/src/content/docs/guide/settings.md`（新增）+ `zh-cn/guide/settings.md`（新增）
- `docs/src/content/docs/guide/config.md` + `zh-cn/guide/config.md`（翻新）
- `docs/scripts/capture-screenshots.mjs`（扩展：逐 Tab 截图）+ 产出的 4 张 PNG
- 侧边栏导航（Starlight frontmatter / 自动导航，若需显式 prev/next 修正则同 usage.md 模式）
- 不影响应用代码、不改变任何运行时行为
