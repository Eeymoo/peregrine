# builtin-material-i18n Specification (Delta)

## ADDED Requirements

### Requirement: 内置物料展示文案按 locale 映射覆盖

前端消费 `MaterialInfo`（`list_materials` IPC 返回）时，MUST 对 `builtin === true` 的物料按当前界面语言查表覆盖展示文案；用户物料（`builtin === false`）MUST 原样展示脚本内文案，不做任何映射。

key 命名规范（`<name>` 为内置 id 去掉 `builtin.` 前缀，如 `builtin.cross` → `cross`）：

- `materials.<name>.name` — 物料显示名，覆盖 `display_name`
- `materials.<name>.params.<paramKey>` — 参数标签，覆盖 `schema[].label`
- `materials.<name>.options.<paramKey>.<value>` — 选项标签，覆盖 `schema[].options[].label`；`<value>` 为脚本原始值转字符串（数字 `4` → `"4"`）

查表未命中时 MUST 回退为脚本内原始文案（zh-CN），MUST NOT 显示原始 key 字符串。

#### Scenario: 英文界面下内置物料显示英文名称与标签

- **WHEN** 界面语言为 en，物料选择器列出 `builtin.cross`
- **THEN** 显示名为 `materials.cross.name` 的英文译文（如 "Cross"）
- **AND** 参数面板中 `size` 参数标签显示 `materials.cross.params.size` 的英文译文

#### Scenario: 选项标签本地化（数字 value）

- **WHEN** 界面语言为 ja-JP，`builtin.corner_dots` 的 `count` 参数渲染 select 控件
- **THEN** 选项 label MUST 取自 `materials.corner_dots.options.count.4` / `.6` / `.8` 的日文译文

#### Scenario: 查表未命中回退脚本原文

- **WHEN** 某内置物料的某个参数 key 在当前语言与 en 回退中均无对应 locale 条目
- **THEN** 该参数标签 MUST 显示脚本 `schema()` 中的原始 label
- **AND** MUST NOT 显示 `materials.xxx.params.yyy` 原始 key 字符串

#### Scenario: 用户物料不受影响

- **WHEN** 加载用户物料 `user.my_material`，其脚本内 `// Name:` 与 label 为任意语言
- **THEN** 前端 MUST 原样展示脚本文案，不查询 `materials.*` 映射

### Requirement: 内置物料 locale 条目 6 语齐全

`materials.*` 命名空间下的所有 key MUST 在 6 份 locale JSON（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）中同时存在，key 集合与既有 `i18n-audit` 规范一致（以 zh-CN 为基准，扁平化后 6 语集合完全相等）。zh-CN 条目文案 MUST 与脚本内原始文案保持一致（避免 zh-CN 界面出现文案漂移）。

#### Scenario: 6 语对齐可被审计工具验证

- **WHEN** 运行 i18n-audit 流程（扁平化 6 份 locale JSON 对比 key 集合）
- **THEN** `materials.*` 命名空间下零 per-locale 缺失、零 per-locale 冗余

#### Scenario: zh-CN 条目与脚本原文一致

- **WHEN** 界面语言为 zh-CN
- **THEN** 内置物料显示的名称/标签与直接读取脚本内 `// Name:` / `label` 的结果一致

### Requirement: 图层名回填使用本地化物料名

从物料创建/切换图层时回填的图层名 MUST 使用当前语言下的本地化物料显示名。回填完成后图层名即为用户数据，后续切换界面语言 MUST NOT 追溯改写已存在的图层名。

#### Scenario: 英文界面创建图层回填英文名

- **WHEN** 界面语言为 en，用户为图层选择物料 `builtin.cross`
- **THEN** 图层名输入框回填 "Cross" 而非 "准星"

#### Scenario: 切语言不追溯已有图层名

- **WHEN** 用户已有名为 "准星" 的图层，随后将界面语言切换为 en
- **THEN** 该图层名保持 "准星" 不变
