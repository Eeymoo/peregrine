## Context

前端设置面板（React + Tailwind + shadcn/ui）当前有三个待解决问题：

1. `src/components/LayersEditor.tsx` 右侧参数面板渲染「变换」区块（`LayerTransformEditor`，位移 / 缩放 / 旋转）。物料运行时已通过 `MATERIAL_RUNTIME_ENABLED = false` 软关闭（见 `src/lib/feature.ts`、`crates/peregrine/src/lib.rs`），图层编辑器整体虽被门控隐藏，但「变换」功能即使在未来恢复时也未就绪，用户要求暂时移除该区块。
2. `src/components/ProfileManager.tsx` 中，配置方案切换下拉框（`Select`，`w-40`）无条件渲染；进入新建 / 重命名编辑态时再追加「输入框（`w-32`）+ 确认 + 取消」三个控件。单图层模式右侧面板宽 320px（`w-80`，含 `p-4` 内边距后可用约 288px），编辑态整行约 400px+，必然溢出。
3. 前端 i18n（`src/lib/i18n.tsx` + `src/i18n/locales/{zh-CN,en}.json`）已落地，但没有工具化手段审查覆盖度：是否存在遗漏的硬编码文案、`t()` 引用的 key（如 `common.add` 一类）是否都有双语条目、双语 key 是否对齐。

约束：

- 注释与文档一律简体中文；用户可见文案一律走 `t()`。
- 本次改动仅涉及前端与 agent skill，不触碰 Rust 后端与配置 schema。
- 物料运行时软关闭约定：被禁用的功能保留源码与配置字段，仅隐藏入口，便于恢复。

## Goals / Non-Goals

**Goals:**

- 图层编辑器右侧不再渲染「变换」区块，且 `LayerTransformEditor` 组件、相关 i18n key、图层 `transform` 数据全部保留。
- ProfileManager 编辑态在 320px 宽面板内不溢出：编辑时隐藏切换下拉框，输入框占据其位置。
- 新增 `.agent/skills/i18n-audit/SKILL.md`，提供可重复执行的 i18n 审查流程（硬编码文案、缺失 key、双语对齐、冗余 key 四个维度）。
- 依据审查结果补齐缺失文案 / 修复已发现的未国际化位置，修复后复查四项清单为空（冗余 key 仅报告）。

**Non-Goals:**

- 不删除 `LayerTransformEditor` 组件源码、`layers.transformSection` 等 i18n key 与 `Transform2D` 类型定义。
- 不重构图层编辑器整体布局，不恢复物料运行时。
- 不改动 Rust 后端的 tray / 错误提示国际化（`src-tauri/src/lib.rs` 的后端 locale 机制保持不变）。
- 不引入新的 npm 依赖（审查流程基于已有工具：`rg` + Node 单行脚本）。
- 不清理「冗余 key」（仅报告，删除另议）。

## Decisions

### 决策 1：「变换」区块直接移除挂载点，而非加开关门控

在 `LayersEditor.tsx` 中删除「变换」区块的 JSX（标题 + `<LayerTransformEditor />`），并留简体中文注释说明「变换功能暂未就绪，已随物料运行时软关闭一并隐藏；恢复时参考此注释还原」。`LayerTransformEditor` 组件、`LayerEditors.tsx` 的导出、i18n key 全部保留。

- 备选 A：用 `MATERIAL_RUNTIME_ENABLED` 门控包裹该区块。否决理由：即使物料运行时恢复，变换功能本身也未就绪，两者不是同一门控条件；另立 flag（如 `LAYER_TRANSFORM_ENABLED`）属于过度设计，当前无人消费。
- 备选 B：连同组件源码一并删除。否决理由：违反物料运行时「保留源码、仅隐藏入口」的既定约定，恢复成本高。

### 决策 2：ProfileManager 编辑态条件渲染，输入框占满剩余宽度

`ProfileManager.tsx` 中将切换下拉框包裹为 `{!isEditing && <Select … />}`；编辑态输入框宽度从固定 `w-32` 改为 `flex-1 min-w-0`，使编辑态整行 = `input(flex-1) + 确认 + 取消`，在 288px 可用宽度内自适应。编辑态的外层容器保持 `flex items-center gap-2` 不变。

- 备选 A：保留下拉框、缩窄各控件宽度。否决理由：四个控件并行在 288px 内无论如何都拥挤，且编辑态下切换方案本身没有意义（重命名对象就是当前方案），隐藏交互上更合理。
- 备选 B：编辑态改为弹窗 / 对话框。否决理由：改动面大、交互路径变长，超出「修复溢出」的最小改动原则。

### 决策 3：i18n 审查以 skill 形式落地，位置 `.agent/skills/i18n-audit/`

项目级工作流 skill 已集中在 `.agent/skills/`（`bugfix`、`codebase-audit`、`feature`、`release` 等），`.opencode/skills/` 由 openspec 管理。因此新 skill 放在 `.agent/skills/i18n-audit/SKILL.md`，纯指令形式（无脚本文件），审查步骤基于：

- `rg` 正则扫描 `src/**` 中 JSX 文本节点与 `title=`/`placeholder=` 属性里的硬编码中英文文案（排除注释、`console.*`、`logAction`、className）。
- `rg -o 't\("[^"]+"\)'` 提取全部 `t()` 引用 key，与两个 locale JSON 扁平化后的 key 集合对比（Node 单行脚本复用 `src/lib/i18n.tsx` 的 flatten 逻辑思路），输出「引用但缺失」「zh/en 不齐」「存在但未引用」三张清单。
- 输出格式：分类清单 + 文件路径 + 行号 + 建议 key / 文案。

- 备选 A：写成 `scripts/i18n-audit.mjs` 独立脚本。否决理由：用户明确要求「写一个 skills」；且硬编码文案的判断需要语义甄别（注释 / 日志 vs 用户可见文案），纯脚本误报高，skill 指引代理判断更准确。
- 备选 B：接入 CI 强制检查。否决理由：误报风险高，先作为按需审查工具，成熟后再考虑 CI 化。

### 决策 4：先建 skill 再用其产出修复文案

实施顺序为：先落 `.agent/skills/i18n-audit/SKILL.md`，再按该 skill 的流程执行一次全量审查，依据结果补齐 `zh-CN.json` / `en.json` 缺失条目、迁移硬编码文案。这样既交付工具也交付一次实际修复，并验证 skill 可用。

## Risks / Trade-offs

- [硬编码文案扫描存在误报（注释、日志、错误信息）] → skill 明确排除规则与「需人工 / 代理判断」的灰区说明；审查结果分类标注置信度。
- [移除「变换」区块后，旧配置中 `transform` 非默认值与视觉表现可能让用户困惑（看不到改不了）] → 数据保留不清理；物料运行时整体已软关闭，图层编辑器入口本身隐藏，实际影响面为零。
- [ProfileManager 条件渲染下拉框可能影响使用它的其他页面] → 消费方仅 `ConfigApp.tsx` 与 `LayersEditor.tsx` 两处，编辑态行为一致，无差异化需求。
- [冗余 key 报告可能很长（如 `layers.*` 软关闭相关 key）] → skill 明确「仅报告、不强制删除」，并在报告中按命名空间分组。
