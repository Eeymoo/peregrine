# i18n-builtin-materials 设计

## Context

i18n 审计（2026-08-15）结论：前端 i18n 体系健康——6 份 locale JSON 各 294 keys 完全对齐、`t()` 引用零缺失、JSX 零硬编码用户可见文案。剩余缺口全部在 i18n 体系**之外**：

1. **内置物料脚本**（`crates/material/builtin/*.rhai`，14 份）：`// Name: 准星` 与 `schema()` 的 `label` / `options[].label` 写死中文，经 `list_materials` IPC 原样送达前端，`LayerPanel.tsx` / `LayersEditor.tsx` 直接渲染 `display_name` / `entry.label` / `opt.label`。内置物料 id 为简单 slug（`cross` / `ring` / `corner_dots` / …，见 `BUILTIN_MATERIALS`）。选项值可能是数字（`count: 4/6/8`）或字符串（`style: "solid"`）。
2. **窗口标题**：`src-tauri/src/lib.rs:317` `.title("Peregrine 配置")`、`:345` `.title("Peregrine 设置")` 硬编码。后端已有数据驱动翻译表（`include_str!` 嵌入前端 locale JSON + `translate(locale, key)`，托盘菜单 4 个 `tray.*` key 在用，并有防 key 拼写回归测试）。前端挂载后用 `t("app.title") + t("settings.title")` 重设标题，但 config 窗口无 `visible(false)`（settings 窗口有），加载瞬间闪现中文标题。
3. **审计中顺手发现的漏网硬编码**：`src/components/LayerPanel.tsx:510` coming_soon 后缀 `` `${entry.label}（开发中）` `` 是 JSX 表达式里的中文字符串，不在 i18n-audit 的 `>中文<` 扫描模式内而漏网。

约束：

- 用户物料脚本（`~/.config/Peregrine/materials/*.rhai`）的文案由脚本作者自负责，类比 VSCode 扩展 displayName，**不在本 change 范围**。
- `MaterialInfo` IPC 结构不变；overlay 渲染路径不消费名称/label，不受影响。
- 14 份 `.rhai` 内中文文案**保留不动**，作为 zh-CN 默认值与查表失败的回退兜底。

## Goals / Non-Goals

**Goals:**

- 内置物料的名称、参数 label、选项 label 在全部 6 门界面语言下正确本地化。
- 窗口创建时标题即本地化（`window.*` key 走后端 `translate()`），消除 config 窗口标题闪烁。
- 补齐 `（开发中）` 后缀等漏网硬编码。
- 日/德/法/俄译文为 AI 认真翻译（en 为源 + zh-CN 校对），非英文占位。
- 6 语 key 集合保持完全一致（i18n-audit 复查通过）。

**Non-Goals:**

- 不改 Rhai 脚本格式 / 不支持脚本内多语言元数据（方案 B，见 Decisions D1）。
- 不翻译用户物料；不为用户物料提供 i18n API。
- 不清理审计报告的 ~15 条真冗余 key（`common.ok` 等预留条目，按 i18n-audit 原则仅报告）。
- 不引入 i18next 等外部库（沿用 in-house `src/lib/i18n.tsx`）。

## Decisions

### D1：内置物料文案走前端映射覆盖（方案 A），不改脚本格式

在 6 份 locale JSON 新增 `materials.*` 命名空间，前端消费 `MaterialInfo` 时按内置 id 查表覆盖。**备选方案**：B（脚本内 `// Name[en]:` / `labels(locale)` 多语言元数据）——把 i18n 复杂度推给脚本作者、registry 需引入 locale 概念、切语言要重拉 IPC，且 6 语对齐无法被现有 i18n-audit 工具校验；C（后端 `list_materials` 出口覆盖）——后端翻译表膨胀 ~110 条 key 且与"物料文案是产品自带内容"的归属不符。A 零 Rust 改动、复用现有 6 语对齐基建，用户物料天然不受影响（只对内置 id 白名单映射）。

### D2：key 命名规范——按物料全限定，不做跨物料去重

```
materials.<id>.name                        # 物料名，如 materials.cross.name = "准星"
materials.<id>.params.<paramKey>           # 参数 label，如 materials.cross.params.size = "臂长"
materials.<id>.options.<paramKey>.<value>  # 选项 label，value 为脚本原始值的字符串形式
                                           # 如 materials.corner_dots.options.count.4 = "4 点（四角）"
                                           #    materials.border_frame.options.style.solid = "实线"
```

**不按文案内容去重**（如多个物料共有的 `"大小"` 各自成 key）：翻译上下文相关（日文里"粗细"在准星与网格场景可能选词不同），且避免物料间隐式耦合。代价是 key 总量略增（估算 ~110 条/语 × 6 语），可接受。`value` 段统一用脚本原始值转字符串（数字 `4` → `"4"`），前端查表时 `String(opt.value)`。

### D3：覆盖点收敛为一个纯函数 `localizeMaterial()`

新增 `src/lib/material-i18n.ts` 导出 `localizeMaterial(m: MaterialInfo, t: TFunction): MaterialInfo`——仅对 `m.builtin === true` 的物料，按 D2 规范查 `t()`，命中则覆盖 `display_name` / `schema[].label` / `schema[].options[].label`，未命中保留脚本原文（`t()` 本身已有"缺 key 返回 key 字符串"行为，需用 `has()` 式存在性判断或比对回退，具体实现时以 `src/lib/i18n.tsx` 现有能力为准；若 i18n 上下文无存在性 API，则为其补一个 `has(key)`）。消费点仅 3 处：`LayersEditor.tsx:279`（选中物料标题）、`LayerPanel.tsx:216/323/327`（名称显示与回填）、`LayerPanel.tsx` schema 控件渲染（label / options）。`LayerPanel.tsx:323` 的 `setName(m.display_name)` 回填的是**图层名**，应使用本地化后的名称（用户可见），接受"切语言后旧图层名不追溯"（图层名一旦回填即用户数据）。

### D4：窗口标题走后端翻译表 + config 窗口补 `visible(false)`

- locale JSON 新增 `window.configTitle` / `window.settingsTitle`（6 语）。
- `create_config_window` / `create_settings_window` 签名增加 resolved locale 参数（或读取当前配置 locale 的辅助函数，与 `read_gpu_setting` / `read_developer_mode` 同模式），`.title(translate(&locale, "window.configTitle"))`。
- config 窗口 builder 补 `.visible(false)`，与 settings 窗口对齐，由既有"就绪后显示"流程统一控制可见性，消除标题闪烁。
- 前端挂载后 `setTitle(t(...))` 逻辑**保留**，作为语言切换后的权威更新路径（后端创建时只保证首次展示正确）。
- 后端测试 `translate_tray_keys_exist_in_all_locales` 扩展覆盖 `window.*` 两个 key。

### D5：漏网硬编码顺手收编

`LayerPanel.tsx:510` 的 `（开发中）` 后缀提取为 `layers.comingSoonSuffix`（或复用现有语义 key，实现时先查重），6 语补齐。实施阶段按 i18n-audit 流程全量复查（含 `t\(` 模板串动态 key 的人工核对）。

### D6：翻译生产策略——双源 + 审计结构化清单驱动

以 en 为翻译源、zh-CN 为校对源（项目惯例）。先把 zh-CN / en 的 `materials.*` 语义定稿，再批量产出 ja-JP / de-DE / fr-FR / ru-RU；术语保持与现有 locale 用词一致（如"准星"已有 `styles.cross` 译文可复用措辞）。完成后重跑 i18n-audit：6 语 key 集合一致、零缺失。

## Risks / Trade-offs

- [key 与脚本参数名漂移：脚本 `schema()` 改 key 后 locale 里的 `materials.<id>.params.<key>` 变成死 key] → 审计工具已能报告冗余 key；在 `crates/material` 的脚本测试或 CI i18n 检查中加一条"内置物料 params/options key 必须存在对应 locale 条目"的校验脚本（tasks 中列为可选增强项）。
- [key 总量增加 ~110×6，locale JSON 体积上涨] → 每条均短文案，体积影响 <50KB/语，可忽略。
- [图层名回填本地化名称后，切语言不追溯旧图层名] → 接受：图层名回填后即用户数据，与"配置即数据"原则一致；在 spec 中明确该语义。
- [config 窗口补 `visible(false)` 改变窗口生命周期表现] → settings 窗口已验证同模式；实施后在 Windows 实测首启（ConfigApp 首启引导路径）确认窗口仍能正常显示。
- [AI 翻译的日/德/法/俄译文质量非母语级] → 接受：双源策略保证语义准确，发布说明中欢迎母语者贡献修订；不为本 change 阻塞项。

## Open Questions

- 无阻塞性问题。`t()` 存在性判断（`has()`）若需补 API，实现时一并落入 `src/lib/i18n.tsx`，属于实现细节。
