> **状态：待实施（Ready）**
> 0.2.0 正式版本已发布（2026-08），暂缓条件解除。本 change 的 design / specs / tasks 已于 2026-08-12 补齐，现可进入实施阶段。运行 `/opsx-apply` 启动。
>
> 跟踪 issue：#41

## Why

Peregrine 目前仅支持中文与英文两门语言，且后端 `src-tauri/src/lib.rs` 的 i18n 是硬编码 `match (locale, key)`、`BackendLocale` 枚举只有 `ZhCN | En`、`detect_locale()` 只认 `zh` 前缀。前端虽然已有数据驱动的 JSON i18n（`src/lib/i18n.tsx` + `src/i18n/locales/*.json`），但前后端两套系统不对称，导致新增任何一门语言都要改 Rust 代码。

为了让 Peregrine 服务更广泛的国际用户，并消除前后端 i18n 的不对称（"前端加语言是加文件，后端加语言是改代码"），需要一次性补齐主流语言，并把后端 i18n 改成与前端共用的数据驱动模型。

## 目标

- **新增 4 门语言**：日本語（ja-JP）、Deutsch（de-DE）、Français（fr-FR）、Русский（ru-RU）。加上现有的 zh-CN / en，共计 **6 门语言**。
- **后端 i18n 数据化**：废除 `src-tauri/src/lib.rs` 中硬编码的 `BackendLocale` 枚举与 `tr()` match 表，改为从 JSON 加载翻译，与前端共用同一份 locale JSON（`src/i18n/locales/*.json`），通过 `include_str!` 编译期嵌入。
- **统一 locale 标识与检测**：前端、后端、配置文件三处使用同一套 locale id；`detect_locale()` 改为多语言映射表（基于 `navigator.language` / Win32 `GetUserDefaultLocaleName` / 环境变量）。
- **Fallback 改为英文**：`FALLBACK_LOCALE` 由 `"zh-CN"` 改为 `"en"`。当某语言缺失某 key 时回退到英文，保证国际用户的兜底体验。
- **AI 翻译一次性生成**：4 门新语言的翻译由 AI 一次性生成并提交，接受"有翻译不保证完全地道"的权衡，后续由社区 PR 修正。
- **修复 `options.json` label 未本地化的现存 bug**：语言下拉框中 "跟随系统" 标签在所有语言下都显示中文，应改为根据当前 locale 显示对应翻译。
- **扩展 i18n-audit skill 到 6 语**：现有 skill 只覆盖 zh-CN / en 双语对齐，扩展为 6 语对齐报告，并配套脚本自动生成结构化缺失清单（含每条缺失 key 在 6 语中的分布），供 AI agent 直接消费修复。
- **建立社区翻译贡献通道**：新增 GitHub Issue 模板（`.github/ISSUE_TEMPLATE/translation-improvement.yml`），鼓励 6 门目标语言（尤其 ja/de/fr/ru 4 门 AI 生成初版）的母语用户提交翻译改进建议；模板含语言下拉、涉及的 i18n key、当前译文、建议译文、上下文截图位等字段，让 AI 生成的"够用但不地道"的初版有结构化的修正入口。
- **顺手补齐项目缺失的 Issue 模板体系**：当前仓库 `.github/ISSUE_TEMPLATE/` 完全不存在，新建 issue 是空白页；借本 change 建立 ISSUE_TEMPLATE 目录的时机，新增一个通用 **Bug 反馈模板**（`.github/ISSUE_TEMPLATE/bug_report.yml`）作为**默认推荐模板**——它是 issue 选择器中排在第一位的入口，覆盖最常见的反馈场景（描述 / 复现步骤 / 期望 vs 实际 / 截图 / 环境信息：Windows 版本 + 架构 x86/x64/ARM64 + Peregrine 版本）。翻译改进模板作为第二入口并列展示。
- **翻译修正闭环自动化**：扩展 `.github/workflows/opencode.yml`，新增一个 job——当 issue 以 `translation-improvement.yml` 模板提交时（或被自动打上 `translation` 标签时），触发 opencode agent 读取模板表单字段（语言 / i18n key / 当前译文 / 建议译文），自动编辑对应的 `src/i18n/locales/<locale>.json` 并提交 PR，让维护者只需 review 而不必手改 JSON。这把"AI 生成初版 → 社区 PR 修正"的两步链路压缩为"AI 初版 → 社区提 issue → opencode 自动 PR → 维护者 merge"的闭环，社区贡献门槛从"会写 PR"降到"会描述更好的译文"。

## 非目标

- **不做阿拉伯语及 RTL 布局**：阿拉伯语（以及任何 RTL 语言）不在本次范围内；RTL 是独立的布局工程，留待后续 change 处理。
- **不做翻译协作平台接入（Weblate / Crowdin）**：6 门语言全由维护者通过 AI 翻译维护，不接入外部翻译平台。
- **不追求翻译完美地道**：AI 翻译接受一定程度的生硬，优先保证"有"而非"精"。
- **不改变 `t()` API 形态**：前端 `useI18n()` / `translate()` / `resolveLocale()` 的对外签名保持不变，仅扩展内部支持的语言集合。
- **不调整配置文件结构**：`AppConfig.settings.locale` 仍是 `String`，仍接受 `"auto"`；只是合法取值集合扩大。
- **不做动态语言切换的额外 UX 优化**：沿用现有的 `peregrine:locale-changed` 事件广播机制，不新增切换动效或过渡。

## What Changes

- **BREAKING（仅对开发者约定）**：`FALLBACK_LOCALE` 由 `"zh-CN"` 改为 `"en"`。对用户无感（仅影响缺失 key 的回退路径），但对国际化定位是产品级决策。
- **后端 i18n 重构**：`src-tauri/src/lib.rs` 中 `BackendLocale` 枚举与 `tr()` match 表替换为从编译期内嵌的 JSON 读取的 `translate(locale, key)` 函数；`detect_locale()` 改为基于映射表的多语言检测。
- **新增 4 份 locale JSON**：`src/i18n/locales/ja-JP.json`、`de-DE.json`、`fr-FR.json`、`ru-RU.json`，结构与现有 `zh-CN.json` / `en.json` 完全一致（key 集合相同），由 AI 翻译生成。
- **扩展 `LANGUAGE_OPTIONS` 与 `options.json`**：新增 4 个语言选项；同时修复 `options.json` 中 label 未本地化的问题（label 改为根据当前 locale 显示）。
- **扩展 `resolveLocale` / `detectLocale` 映射**：前端 `src/lib/i18n.tsx` 的 `detectLocale()` 增加 ja/de/fr/ru 前缀映射；`Locale` 类型联合扩大。
- **后端 `detect_locale` / `BackendLocale::from_str` 映射扩展**：对应增加 5 条前缀映射（含现有 zh/en 共 6 条）。
- **扩展 i18n 审查 skill**：`.agent/skills/i18n-audit/` 的"双语对齐"维度扩展为"6 语对齐"。
- **新增翻译改进 Issue 模板**：`.github/ISSUE_TEMPLATE/translation-improvement.yml`，含语言下拉（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）、i18n key、当前译文、建议译文、上下文截图字段。
- **新增 Bug 反馈 Issue 模板（默认推荐）**：`.github/ISSUE_TEMPLATE/bug_report.yml`，作为 issue 选择器第一位入口；字段含问题描述、复现步骤、期望行为、实际行为、截图、环境信息（Windows 版本 / 架构 / Peregrine 版本）、额外上下文。
- **新增 Question Issue 模板（兜底）**：`.github/ISSUE_TEMPLATE/question.yml`，作为非 bug / 非翻译改进类反馈的兜底入口（用法咨询、功能讨论、杂项）；因 `blank_issues_enabled: false` 关闭了空白 issue 入口，本模板替代"空白 issue"承担自由形态提问的功能，字段精简（问题描述、已尝试的操作、相关上下文）。
- **新增 Issue 模板选择器配置**：`.github/ISSUE_TEMPLATE/config.yml`，将 `bug_report` 设为默认推荐、`translation-improvement` 并列展示，`blank_issues_enabled: false`（强制走模板，避免空白 issue 绕过结构化信息收集与翻译自动化闭环）。
- **扩展 `.github/workflows/opencode.yml`**：新增一个 `auto-translate` job，触发条件 `issues.opened` + 过滤模板名为 `translation-improvement.yml`（或新增 `translation` 标签后由 `auto-label` 触发本 job）；prompt 指示 opencode 解析 `issue.form_data` 提取语言 / key / 当前译文 / 建议译文 → 编辑 `src/i18n/locales/<locale>.json` → 在 `feature/i18n-<issue-number>-<key>` 分支提交 → 推送并以 "Closes #N" 开 PR，body 引用原 issue。

## Capabilities

### New Capabilities

- `backend-i18n`: 后端 i18n 数据驱动能力——从编译期内嵌的 JSON 加载翻译、统一 locale 检测、英文 fallback。

### Modified Capabilities

- `i18n-audit`: 审查维度从"双语对齐（zh-CN / en）"扩展为"6 语对齐（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）"，配套脚本生成结构化缺失清单（每条 key 在 6 语中的分布），供 AI agent 消费后批量翻译修复；并对缺失语言 JSON 文件的情况给出报告。

## Impact

- **`src-tauri/src/lib.rs`**：`BackendLocale` 枚举、`from_str`、`detect`、`detect_locale`、`tr`、`current_locale` 全部重写为数据驱动；`include_str!` 引用 `../../../src/i18n/locales/*.json`。
- **`src/lib/i18n.tsx`**：`Locale` 类型联合扩展；`detectLocale()` / `resolveLocale()` 增加映射分支；`FALLBACK_LOCALE` 改 `"en"`；`localeMap` 静态注册 6 个 locale。
- **`src/i18n/locales/`**：新增 4 份 JSON（AI 翻译产物）。
- **`src/i18n/options.json`**：新增 4 个语言选项，并修复 label 本地化问题。
- **`.agent/skills/i18n-audit/`**：审查脚本/skill 指引中对齐维度从 2 语扩到 6 语，脚本输出结构化缺失清单（含每条 key 在 6 语中的分布）供 AI agent 直接消费修复。
- **`openspec/specs/i18n-audit/spec.md`**：需求文案从"双语"改为"6 语"。
- **`.github/ISSUE_TEMPLATE/`**：新增四个文件——`bug_report.yml`（默认推荐模板）、`translation-improvement.yml`（翻译改进模板）、`question.yml`（兜底提问模板）、`config.yml`（选择器配置，`blank_issues_enabled: false`）；当前仓库尚无任何 issue 模板，本 change 会一并建立该目录与整套入口。
- **`.github/workflows/opencode.yml`**：新增 `auto-translate` job（触发 `issues.opened` + 模板过滤）；现有 `opencode`（评论触发）与 `auto-label`（标签触发）job 保留；如选择标签触发路径，则 `auto-label` 的 prompt 需扩展 `translation` 标签规则。
- **`crates/config/src/schema.rs`**：`default_locale()` 保持 `"auto"`（已是），但单元测试可能需要补多语言场景。
- **无新依赖**：不引入 i18n 库（如 `fluent` / `rust-i18n`），继续用 `serde_json` 反序列化 + `include_str!`，与前端 flatten 逻辑对齐。
- **bundle 体积**：6 份 JSON 各约 11KB，总计约 66KB 编译期内嵌，对 release 包大小影响可忽略。
