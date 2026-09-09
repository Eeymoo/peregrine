## Context

设置窗口（`src/SettingsApp.tsx`）现有 7 个 Tab：通用 / 覆盖层 / 物料（`MATERIAL_RUNTIME_ENABLED` 编译期开启时显示）/ 快捷键 / 更新 / 关于 / 开发（解锁后）。配置侧对应两块：

- 全局 `settings` 块（`AppSettings`，`crates/config/src/schema.rs`）：语言、自动切换、GPU 加速、渲染后端、抗锯齿、快捷颜色、热键绑定、遥测开关、更新通道、CN 镜像、物料设置等 15+ 字段
- Profile 级新格式 `layers`（多图层 + Rhai 物料），旧 `crosshair` 单格式由迁移逻辑兼容

文档站现状：`guide/config.md` 的 JSON 示例仍是旧格式（无 `settings` / 无 `layers`）；截图管线 `docs/scripts/capture-screenshots.mjs` 只产出一张 `settings-layers.png`（mock-tauri 桩 + headless Chromium + root dev server :5199）。

## Goals / Non-Goals

**Goals:**

- 新增双语 `guide/settings.md`，覆盖通用 / 覆盖层 / 物料 / 快捷键 / 更新五个 Tab，每 Tab 至少一张真实截图
- `config.md` 与 `schema.rs` 当前实现对齐：新格式示例优先 + `AppSettings` / `MaterialSettings` 字段表 + 旧格式降级为遗留章节
- 截图管线可重复执行（`npm run screenshots`），新 PNG 入库

**Non-Goals:**

- 不写「关于」Tab 与「开发者模式」；「更新」只细微介绍
- 不改任何 Rust / 前端应用代码
- 不重写 `layers.md` / `material-scripting.md`，仅互链

## Decisions

### D1 截图走真实截图管线（扩展 capture-screenshots.mjs），而非手绘

按用户决定采用真实截图。具体做法：在现有脚本中按 `role/trigger` 定位 Tab 按钮依次点击（通用 / 覆盖层 / 物料 / 快捷键 / 更新），每 Tab 等待渲染稳定后截图，产出 `settings-general.png` 等入既有截图目录。mock-tauri 桩已覆盖 IPC 面，无需新桩；「物料」Tab 需确保 mock 配置带 `settings.material`（默认即可）。
备选：手绘 SVG / Mermaid 图——被否，用户要真实 UI 图；但**逻辑语义**（两层与门、配置结构）截图表达不了，仍配 ASCII/Mermaid 示意图作为补充，两者不互斥。

### D2 环境不可行时的降级路径

截图依赖 root Vite dev server（:5199）+ headless Chromium。实施时先探测：起 dev server → 跑现有脚本验证 `settings-layers.png` 可再产出。若 Chromium 不可用，文档与字段表照常交付，截图任务标记阻塞并在 tasks 中保持未勾选，管线代码仍合入（本地/CI 可补跑）。这是文档变更，不因截图环境缺失而阻塞整个 change。

### D3 config.md 翻新策略：新格式优先，旧格式降级

正文示例改为 `settings` + `layers` 结构（与 `Profile::default_profile()` 一致：`builtin.edge_rect` 图层）；`Crosshair` 大字段表移入「遗留格式（Legacy）」章节并前置迁移说明（旧文件加载即自动迁移为 layers，`crosshair` 字段随后消失）。理由：旧示例继续当正文会误导手写用户。

### D4 settings.md 页面结构

总览（Tab 索引图）→ 通用 → 覆盖层 → 物料（重点，含两层与门 ASCII 图 + FPS 语义表）→ 快捷键（动作↔键位表）→ 更新（细微介绍）。双语镜像；sidebar 分页如受顶层 link 影响，按 `usage.md` 模式用 frontmatter `prev/next` 显式修正。

### D5 字段表以 schema.rs 为唯一事实源

`AppSettings` / `MaterialSettings` 字段表逐字段对照 `crates/config/src/schema.rs` 的 doc 注释与默认值函数编写，不改 schema。若发现文档与 schema 冲突，以代码为准并在 PR 中注明。

## Risks / Trade-offs

- [Chromium / dev server 环境不可用] → D2 降级：文档先行，截图任务留待补跑，不阻塞合入
- [截图与未来 UI 改版漂移] → 管线可重复执行；截图文件名带 Tab 名不带版本号，改版后重跑即覆盖
- [config.md 字段表与 schema 再次失同步] → 字段表旁注明「以 `crates/config/src/schema.rs` 为准」，降低读者对文档的绝对信任
- [双语页内容漂移] → en 页与 zh 页同 PR 产出，验收逐节对照

## Migration Plan

纯文档 + 脚本变更，无运行时迁移。回滚 = revert 单个 PR。

## Open Questions

无——范围已在探索阶段与用户确认（真实截图 / 落后内容一并升级 / 关于与开发者忽略 / 更新细微介绍）。
