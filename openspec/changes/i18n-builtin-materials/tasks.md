# i18n-builtin-materials 任务清单

## 1. key 清单与 zh-CN / en 语义定稿

- [x] 1.1 从 14 份 `crates/material/builtin/*.rhai` 提取全量 key 清单（脚本 id、`// Name:`、`schema()` 的每个 `key`+`label`、`options[].value`+`label`），生成 `materials.<name>.name` / `materials.<name>.params.<key>` / `materials.<name>.options.<key>.<value>` 完整条目列表（可用临时 Node 脚本解析，产物放 `/tmp` 不入库）
- [x] 1.2 在 `src/i18n/locales/zh-CN.json` 新增 `materials` 命名空间，条目文案与脚本原文逐字一致（含 `window.configTitle` / `window.settingsTitle` 与 `layers.comingSoonSuffix` 等非 materials 新 key）
- [x] 1.3 在 `src/i18n/locales/en.json` 补齐对应英文译文，术语与既有条目一致（参考 `styles.*` / `anchors.*` 现有译法）

## 2. 前端覆盖逻辑

- [x] 2.1 检查 `src/lib/i18n.tsx` 是否具备 key 存在性判断能力；若无则补 `has(key)`（不得依赖"t() 缺 key 返回 key 字符串"的比对hack）
- [x] 2.2 新建 `src/lib/material-i18n.ts`：`localizeMaterial(m: MaterialInfo, t/has)` 纯函数，仅对 `builtin === true` 按 D2 规范覆盖 `display_name` / `schema[].label` / `schema[].options[].label`（`String(opt.value)` 查表），未命中回退脚本原文
- [x] 2.3 接入 3 处消费点：`LayersEditor.tsx:279`（选中物料标题）、`LayerPanel.tsx:216/323/327`（名称显示与图层名回填）、`LayerPanel.tsx` schema 控件渲染（label / options label，含 select 数字 value）
- [x] 2.4 `LayerPanel.tsx:510` 的 `（开发中）` 后缀改为 `t("layers.comingSoonSuffix")`（实现前先查现有 key 是否已有可复用条目）

## 3. 后端窗口标题

- [x] 3.1 `create_config_window` / `create_settings_window` 按创建时 resolved locale 使用 `translate(locale, "window.configTitle" / "window.settingsTitle")` 设置 `.title(...)`（locale 获取与 `read_gpu_setting` 同模式），删除两处硬编码中文
- [x] 3.2 config 窗口 builder 补 `.visible(false)`，与 settings 窗口对齐
- [x] 3.3 扩展后端回归测试（`translate_tray_keys_exist_in_all_locales` 同类）覆盖 `window.configTitle` / `window.settingsTitle` 两个 key 在 6 语存在
- [x] 3.4 窗口标题实时本地化：新增共享 hook `src/hooks/useWindowTitle.ts`，`SettingsApp.tsx` / `ConfigApp.tsx` 接入（key 与后端创建时一致，语言切换后 `setTitle` 实时跟随；此前 ConfigApp 缺失运行时更新、SettingsApp 拼接来源与后端不一致）

## 4. 日 / 德 / 法 / 俄翻译

- [x] 4.1 以 en 为翻译源 + zh-CN 为校对源，产出 ja-JP 全部新增条目
- [x] 4.2 同上产出 de-DE
- [x] 4.3 同上产出 fr-FR
- [x] 4.4 同上产出 ru-RU

## 5. 验证

- [x] 5.1 重跑 i18n-audit 流程：6 语 key 集合完全一致、`t()` 引用零缺失、无新增硬编码用户可见文案（含动态 key 人工核对）
- [x] 5.2 `cargo test -p peregrine_material` + `cargo test`（后端新测试）通过，`cargo fmt --check` / `cargo clippy` 干净
- [x] 5.3 `npm run build`（tsc + vite）通过
- [ ] 5.4 【PR 合并前人工执行】Windows 实测：en/ja-JP 界面下物料选择器与参数面板全英文/日文；首启 ConfigApp 窗口正常显示且无标题闪烁；切换语言后物料文案与窗口标题随界面刷新
- [ ] 5.5（可选增强，留作后续）在 CI 或脚本中加"内置物料 params/options key 必须有对应 locale 条目"的防漂移校验
