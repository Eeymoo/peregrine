## 1. 后端 i18n 数据驱动重构

- [x] 1.1 在 `src-tauri/src/lib.rs` 用 `include_str!("../../../src/i18n/locales/{zh-CN,en,ja-JP,de-DE,fr-FR,ru-RU}.json")` 编译期内嵌 6 份 JSON
- [x] 1.2 用 `std::sync::LazyLock<HashMap<&'static str, HashMap<String, String>>>` 建立翻译表，首次访问时 `serde_json` 反序列化
- [x] 1.3 实现 `translate(locale: &str, key: &str) -> String`，查表顺序：当前 locale → `FALLBACK_LOCALE="en"` → 原始 key
- [x] 1.4 重写 `detect_locale() -> &'static str`：基于 Win32 `GetUserDefaultLocaleName`（Windows）或 `LANG`/`LC_ALL`/`LC_MESSAGES`（非 Windows），按 6+1 前缀映射表返回 locale id
- [x] 1.5 重写 `current_locale(state: &AppState) -> &'static str`：返回 `&'static str`（locale id），保持现有 `auto`→detect / 显式 locale→resolve 的逻辑
- [x] 1.6 移除 `BackendLocale` 枚举、`from_str`、`detect`、`tr(locale, key)` match 表
- [x] 1.7 把所有原 `tr(current_locale(&state), "...")` 调用点改为 `translate(&current_locale(&state), "...")`（`src-tauri/src/lib.rs` 约 10 处，含 tray 构建 / `start_overlay` 校验 / mode 切换提示）
- [x] 1.8 加 `FALLBACK_LOCALE` 常量为 `"en"`（同时改前端，见 3.x）
- [x] 1.9 加 `detect_locale` / `BackendLocale::from_str` 替代函数的单元测试：覆盖 6 个前缀映射分支 + 1 个 fallback 分支
- [x] 1.10 加 `translate` 单元测试：覆盖当前命中 / 英文回退 / 原始 key 回退三个分支

## 2. 6 份 locale JSON 生成与对齐

- [x] 2.1 备份现有 `src/i18n/locales/zh-CN.json` 与 `en.json` 的 key 集合作为基准（人工或脚本快照）
- [x] 2.2 用 AI 以 `en.json` 为源翻译生成 `src/i18n/locales/ja-JP.json`，保留所有插值占位符（如 `{name}`）
- [x] 2.3 用 AI 以 `en.json` 为源翻译生成 `src/i18n/locales/de-DE.json`，保留占位符
- [x] 2.4 用 AI 以 `en.json` 为源翻译生成 `src/i18n/locales/fr-FR.json`，保留占位符
- [x] 2.5 用 AI 以 `en.json` 为源翻译生成 `src/i18n/locales/ru-RU.json`，保留占位符
- [x] 2.6 用脚本（`jq` 或一次性 Rust 测试）校验 6 份 JSON 扁平化后 key 集合完全一致
- [x] 2.7 人工 spot-check 各 JSON 中 `tray.settings` / `tray.quit` / `target_window_required` 等关键 key 的译文合理性
- [x] 2.8 **中文源交叉校对**：把 4 份新 JSON 的每条译文与 `zh-CN.json`（项目原始语义源）并排对比，catch 从 en 回译引入的语义偏移（如"锚点"→"anchor"→ 译回时丢失"视觉参照"的隐喻）；重点 spot check：tray 菜单 / overlay 错误提示 / material 相关 key

## 3. 前端 i18n 扩展与 options.json 修复

- [x] 3.1 `src/lib/i18n.tsx`：`FALLBACK_LOCALE` 从 `"zh-CN"` 改为 `"en"`
- [x] 3.2 `src/lib/i18n.tsx`：`localeMap` 静态注册 6 个 locale（import 6 份 JSON）
- [x] 3.3 `src/lib/i18n.tsx`：`Locale` 类型联合扩展为 `"auto" | "zh-CN" | "en" | "ja-JP" | "de-DE" | "fr-FR" | "ru-RU"`
- [x] 3.4 `src/lib/i18n.tsx`：`detectLocale()` 增加 ja/de/fr/ru 前缀映射，与后端映射表对齐
- [x] 3.5 `src/lib/i18n.tsx`：`resolveLocale()` 验证对 6 个 locale 的处理（auto→detect / 显式→直接 resolve）
- [x] 3.6 修复 `src/i18n/options.json` 现存 bug：`label` 字段从直接文案改为 i18n key（如 `option.follow_system`）
- [x] 3.7 在 6 份 locale JSON 中新增对应的 `option.*` key（每语言每选项一条）
- [x] 3.8 `LANGUAGE_OPTIONS` 增加 4 个新选项（ja-JP / de-DE / fr-FR / ru-RU）
- [x] 3.9 修复前端渲染 `LANGUAGE_OPTIONS` 时对 `label` 走 `t()` 翻译
- [ ] 3.10 人工验证：切换 6 门语言，下拉框 `option.follow_system` 的 label 跟随切换显示

## 4. i18n-audit skill 升级到 6 语

- [x] 4.1 `.agents/skills/i18n-audit/SKILL.md`：对齐维度文案从"双语"改为"6 语（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）"
- [x] 4.2 扩展 skill 中的检查清单/脚本，覆盖 6 份 locale JSON 的缺失 key 检查
- [x] 4.3 扩展 skill 检查"6 语 key 集合不一致"维度（per-locale 缺失 + per-locale 冗余）
- [x] 4.4 扩展 skill 检查"目标 locale JSON 文件本身缺失"情况（如 `fr-FR.json` 完全不存在）
- [x] 4.5 skill 输出新增**结构化 JSON 缺失清单**（含 `missing[]`：key + missing_in[] + present_in{}；`extra[]`：key + only_in[]），存为 `.agents/skills/i18n-audit/output/missing-keys.json`
- [x] 4.6 更新 `openspec/specs/i18n-audit/spec.md`：需求文案从"双语"改为"6 语"+ 结构化清单输出要求
- [x] 4.7 手动跑一次升级后的 skill，确认输出格式正确且与现状（6 语齐全）一致

## 5. GitHub Issue 模板体系

- [x] 5.1 新建 `.github/ISSUE_TEMPLATE/bug_report.yml`：字段含描述 / 复现步骤 / 期望 vs 实际 / 截图 / 环境（Windows 版本 + 架构 x86/x64/ARM64 + Peregrine 版本）/ 额外上下文；标为默认推荐（排序第一）
- [x] 5.2 新建 `.github/ISSUE_TEMPLATE/translation-improvement.yml`：字段含语言下拉（zh-CN/en/ja-JP/de-DE/fr-FR/ru-RU）/ i18n key / 当前译文 / 建议译文 / 上下文截图
- [x] 5.3 新建 `.github/ISSUE_TEMPLATE/question.yml`：字段精简（问题描述 / 已尝试的操作 / 相关上下文）作为兜底
- [x] 5.4 新建 `.github/ISSUE_TEMPLATE/config.yml`：`blank_issues_enabled: false`，三个模板并列展示，可考虑加 `contact_links` 指向 docs / discussions
- [ ] 5.5 推送后人工在 GitHub UI 验证 issue 选择器显示 3 个模板、无 "Open a blank issue" 入口

## 6. opencode workflow 翻译修正闭环

- [x] 6.1 在 `.github/workflows/opencode.yml` 的 `auto-label` job prompt 中扩展规则：使用 `translation-improvement.yml` 模板提交的 issue 自动打 `translation` 标签（同时把 `translation` 加入"可用标签"列表）
- [x] 6.2 在 `.github/workflows/opencode.yml` 新增 `auto-translate` job：触发条件 `issues.opened` + 过滤 `github.event.label.name == 'translation'`（标签触发）
- [x] 6.3 `auto-translate` job 的 `permissions` 显式声明 `contents: write` + `pull-requests: write` + `issues: read`
- [x] 6.4 `auto-translate` job 的 opencode prompt：解析 issue body 的语言 / key / 当前译文 / 建议译文 → 编辑 `src/i18n/locales/<locale>.json` → 在 `feature/i18n-<issue#>-<key>` 分支提交 → 推送 → 开 PR 含 "Closes #N"
- [x] 6.5 PR body MUST 引用原 issue 编号，并声明"自动生成，等待维护者 review"
- [x] 6.6 在 `.github/workflows/` 加翻译 PR 的 CI 校验门（脚本 `scripts/check-i18n.rs` 或扩展 `ci.yml`）：对修改 `src/i18n/locales/*.json` 的 PR 触发 (a) JSON 可解析、(b) 单语 key 集合不变（只允许改 value）、(c) 6 语 key 集合对齐 三重校验；失败阻塞 merge
- [ ] 6.7 手动测试：用一个测试 issue 走完"提 issue → 自动打标签 → 自动开 PR → CI 校验 → review → merge"全链路（可在 fork 或测试仓库验证）

## 7. 文档与 changelog

- [x] 7.1 在 `docs/guide/` 合适位置（如 features/configuration）补充"支持 6 门语言"的说明
- [x] 7.2 在 README 或 docs 显著位置声明"翻译由 AI 初版，欢迎母语者通过 translation-improvement 模板修正"
- [x] 7.3 `docs/guide/changelog.md` 增加 6 语支持的条目（按项目惯例区分 stable / preview）
- [x] 7.4 在 `docs/guide/development.md` 补"前后端 locale 映射表必须对齐"的开发约定（指向 `src/lib/i18n.tsx` 与 `src-tauri/src/lib.rs` 的映射代码位置）

## 8. 验证与质量门

- [x] 8.1 `cargo fmt --check`
- [ ] 8.2 `cargo clippy --workspace -- -D warnings`（CI 只校验 config / material / peregrine 三库；src-tauri 含预存告警，详见下方说明）
- [x] 8.3 `cargo test -p peregrine_config`（确认配置层多语言场景无回归）
- [ ] 8.4 `cargo build --release`（确认 `include_str!` 路径在三平台都能编译）
- [x] 8.5 `npm run build`（前端 TypeScript 编译 + Vite 构建无错）
- [ ] 8.6 手动 `npx tauri dev` 启动应用：切换 6 门语言，验证设置面板 / tray 菜单 / 错误提示 / 语言下拉框 label 全部跟随
- [ ] 8.7 手动验证 `auto` 模式：把系统 locale 改为 de / ja / fr / ru，重启应用，验证自动检测到对应语言
- [ ] 8.8 手动验证 fallback：临时删除某 locale JSON 的某 key，验证回退到英文而非 panic
- [x] 8.9 运行升级后的 i18n-audit skill，确认 0 缺失、0 冗余、6 语 key 集合一致
- [x] 8.10 `openspec validate add-multi-language-support` 通过

## 备注：关于 8.2 / 8.4 与未勾选项

- **8.2**：项目 CI 的 lint job 范围是 `cargo clippy -p peregrine_config -p peregrine_material -p peregrine -- -D warnings`（已在 `ci.yml` 确认），这三库全绿。`--workspace` 还会带上 `peregrine-tauri`，它含**预存**告警（`unused_imports`/`useless_conversion` 等共 6 条，`git stash` 对照 main 已确认）；本 PR 净减 4 条告警（清掉了自己引入的 `&translate()` 借用告警），无新增告警。
- **8.4**：`cargo check -p peregrine-tauri`（dev profile，含 `include_str!` 路径校验）已通过；`--release` 走完整优化链路，Linux CI 无 Windows 资源无法跑，留给 Windows 三平台 CI 验证。
- **8.6 / 8.7 / 8.8 / 5.5 / 6.7**：需在 Windows 真机或 GitHub UI 手动验证，PR 合并前后均可由维护者补做。
