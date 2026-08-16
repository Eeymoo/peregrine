# i18n-builtin-materials 提案

> 跟踪 issue：#78（https://github.com/Eeymoo/peregrine/issues/78）

## Why

产品已支持 6 门界面语言（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU），前端 i18n 体系本身健康（294 keys × 6 语完全对齐、零硬编码 JSX 文案），但**两套面向用户的文案仍游离在 i18n 体系之外**：

1. **14 份内置物料脚本**（`crates/material/builtin/*.rhai`）把展示文案写死在脚本里——14 个物料名（`// Name: 准星`）+ 约 70 个去重后的参数/选项 label（`"臂长"`、`"实线"/"虚线"`、`"4 点（四角）"` 等）。前端 `MaterialInfo.display_name` / `schema[].label` 拿到什么显示什么，导致非中文界面下物料选择器和参数面板全是中文，体验非常不合理。
2. **窗口标题**在 `src-tauri/src/lib.rs:317/345` 硬编码为 `"Peregrine 配置"` / `"Peregrine 设置"`。前端挂载后会用 `t()` 重设标题掩盖了问题，但 config 窗口创建时未设 `visible(false)`，加载瞬间会闪现中文标题；且后端已有的 `translate(locale, key)` 翻译表基础设施（托盘菜单在用）并未覆盖窗口标题。

## What Changes

- **内置物料文案接入前端 i18n（方案 A：前端映射覆盖）**：在 6 份 locale JSON 中新增 `materials.<id>.name` 与 `materials.<id>.params.<key>`（含选项 label 的 `materials.<id>.options.<key>.<value>`）命名空间；前端在消费 `MaterialInfo` 时按 builtin 物料 id 查表覆盖 `display_name` 与 `label`，查不到 key 时回退脚本原文（用户物料天然不受影响）。
- **AI 认真翻译**：以 en 为翻译源 + zh-CN 为校对源的双源策略，产出日/德/法/俄可用译文（非英文占位）。
- **窗口标题走后端翻译表**：locale JSON 新增 `window.configTitle` / `window.settingsTitle` 两个 key；`create_config_window` / `create_settings_window` 创建时使用 `translate(&initial_locale, ...)`；并为 config 窗口补 `visible(false)` 与 settings 窗口对齐，消除标题闪烁。
- **审计基线固化**：将本次 i18n 审计结论（前端 0 硬编码、6 语 294 keys 对齐）写入设计文档作为回归基线。

## Capabilities

### New Capabilities

- `builtin-material-i18n`: 内置物料展示文案（名称、参数 label、选项 label）的 6 语本地化能力——前端映射覆盖方案、key 命名规范、回退语义、与用户物料的边界。

### Modified Capabilities

- `backend-i18n`: 新增需求——窗口标题 MUST 通过 `translate()` 走翻译表（`window.*` key），窗口创建时按初始 locale 设置本地化标题，禁止硬编码。
- `material-runtime`: 修订内置物料脚本中 `// Name:` 与 schema `label` 的定位——脚本内中文文案仅作为 zh-CN 默认值与回退兜底，UI 展示 MUST 以前端 locale 映射为准。

## Impact

- **前端**：`src/i18n/locales/*.json`（6 份，每份新增 ~84 条 key）；新增物料文案覆盖逻辑（预计 `src/lib/material-i18n.ts` 或并入 `LayersEditor.tsx` 消费处）。
- **后端**：`src-tauri/src/lib.rs`（窗口创建两处 + config 窗口 `visible(false)`）；`translate()` 无需改动（数据驱动，新 key 自动生效）。
- **物料脚本**：14 份 `.rhai` 中的中文 `Name:` / `label` **保留不动**（作为 zh-CN 默认与回退）。
- **测试**：后端 `translate_tray_keys_exist_in_all_locales` 同类测试扩展覆盖 `window.*` key；前端 i18n-audit 流程复查 6 语对齐。
- **不影响**：用户物料（脚本作者自负责文案）、IPC 接口形状（`MaterialInfo` 结构不变）、overlay 渲染路径。
