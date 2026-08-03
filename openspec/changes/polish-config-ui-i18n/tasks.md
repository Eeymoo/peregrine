## 1. 暂时移除图层编辑器「变换」区块

- [ ] 1.1 在 `src/components/LayersEditor.tsx` 中删除「变换」区块 JSX（`layers.transformSection` 标题 + `<LayerTransformEditor />`），并留简体中文注释说明该功能暂未就绪、随物料运行时软关闭隐藏及恢复方式
- [ ] 1.2 确认 `src/components/LayerEditors.tsx` 的 `LayerTransformEditor` 组件、导出与 `layers.transformSection` 等 i18n key 保留不删；若移除区块后 `LayerTransformEditor` import 变为未使用，清理该 import
- [ ] 1.3 运行 `npm run build`（或 `npx tsc --noEmit`）确认无类型 / 编译错误

## 2. 修复 ProfileManager 编辑态溢出

- [ ] 2.1 在 `src/components/ProfileManager.tsx` 中将切换下拉框改为 `{!isEditing && <Select … />}` 条件渲染
- [ ] 2.2 编辑态输入框宽度由 `w-32` 改为 `flex-1 min-w-0`，确保编辑态整行（输入框 + 确认 + 取消）在 320px 宽面板内不溢出
- [ ] 2.3 验证新建 / 重命名 / 确认 / 取消 / Escape 各路径后下拉框恢复正常显示，方案切换功能不受影响

## 3. 新增 i18n 审查 skill

- [ ] 3.1 创建 `.agent/skills/i18n-audit/SKILL.md`，包含：触发描述、四个审查维度（硬编码 UI 文案 / 缺失 key / 双语对齐 / 冗余 key）、具体 `rg` 与 Node 对比命令、排除规则（注释 / `console.*` / `logAction` / className）、分类输出格式（文件 + 行号 + 建议）
- [ ] 3.2 按 skill 中的流程实际执行一次全量审查，记录结果（缺失 key 清单、双语不齐清单、硬编码文案清单）

## 4. 补齐缺失文案与未国际化修复

- [ ] 4.1 依据 3.2 审查结果，在 `src/i18n/locales/zh-CN.json` 与 `en.json` 中补齐所有 `t()` 引用但缺失的 key（双语同时补）
- [ ] 4.2 修复审查发现的硬编码用户可见文案：迁移为 `t()` 调用并补充双语条目（`common.add` 等已在用的 key 重点核对）
- [ ] 4.3 修复双语 key 不一致的条目，使两个 locale 文件扁平化后 key 集合完全一致

## 5. 验证

- [ ] 5.1 重新执行 i18n 审查：缺失 key 清单为空、双语 key 集合一致、无未修复的硬编码用户可见文案（冗余 key 仅报告）
- [ ] 5.2 运行 `npm run build` 通过；运行 `npx tauri dev` 人工确认：图层编辑器无「变换」区块、ProfileManager 编辑态不溢出、切换 zh-CN / en 后本次补齐的文案正常显示
