## Context

Peregrine 当前的国际化能力是**前后端不对称**的：

```
┌──────────────────────────────┐    ┌──────────────────────────────┐
│       前端（数据驱动）         │    │      后端（硬编码）           │
├──────────────────────────────┤    ├──────────────────────────────┤
│ src/lib/i18n.tsx             │    │ src-tauri/src/lib.rs          │
│  · localeMap: {locale: JSON} │    │  · enum BackendLocale {       │
│  · detectLocale()  ← navigator│    │      ZhCN | En               │
│  · resolveLocale()           │    │    }                          │
│  · translate(key) ← 运行时查表│    │  · BackendLocale::from_str    │
│  · FALLBACK_LOCALE = "zh-CN" │    │    ← 仅识别 "zh" 前缀          │
│                              │    │  · tr(locale, key)            │
│ src/i18n/locales/*.json      │    │    ← match (locale, key) 表    │
│  · zh-CN.json                │    │  · detect_locale()            │
│  · en.json                   │    │    ← Win32/环境变量             │
│                              │    │                               │
│ src/i18n/options.json        │    │  共 7 条硬编码翻译，2 门语言   │
│  · LANGUAGE_OPTIONS          │    │                               │
└──────────────────────────────┘    └──────────────────────────────┘
        加一门语言 = 加一份 JSON              加一门语言 = 改 Rust 代码
```

**关键约束**：

- 配置层 `AppConfig.settings.locale` 是 `String`，接受 `"auto"` 或具体 locale id；本 change 不改配置结构。
- 前后端都各自实现了一套 `detect_locale`，必须保持 locale id 一致。
- Tauri tray 菜单文案、错误提示等用户可见字符串都走 `tr()`，目前仅 7 条、2 语。
- 后端代码 `tr()` 调用点散落在 `src-tauri/src/lib.rs` 多处（tray 构建、`start_overlay` 校验、mode 切换提示）。
- 仓库已有 `i18n-audit` skill 与对应 capability spec，只覆盖 zh-CN / en 双语对齐。
- 仓库 `.github/ISSUE_TEMPLATE/` 完全不存在；`.github/workflows/opencode.yml` 已有 `opencode`（评论触发）和 `auto-label`（issue 打标签）两个 job。
- `crates/config`、`crates/material`、`crates/peregrine` 不直接做 UI 翻译；后端 i18n 仅属于 `src-tauri` 二进制层。

**现状代码位置**（实施时的精确坐标）：

| 文件 | 现有内容 |
|---|---|
| `src-tauri/src/lib.rs:68-153` | `BackendLocale` 枚举、`from_str`、`detect`、`detect_locale`、`tr`、`current_locale` |
| `src-tauri/src/lib.rs:124-142` | `tr()` 的 7 条硬编码 match 分支（zh/en × 6 key + `overlay_active_cannot_change_mode`） |
| `src/lib/i18n.tsx:18` | `FALLBACK_LOCALE = "zh-CN"` |
| `src/lib/i18n.tsx:21-25` | `localeMap` 静态注册 zh-CN/en |
| `src/lib/i18n.tsx:46-62` | `detectLocale` / `resolveLocale` |
| `src/i18n/options.json` | `LANGUAGE_OPTIONS` 来源，且 `label` 字段未走本地化（现存 bug） |
| `openspec/specs/i18n-audit/spec.md` | 双语对齐需求文案 |

## Goals / Non-Goals

**Goals:**

- 把后端 i18n 从"硬编码枚举 + match 表"改造为"编译期内嵌 JSON + 数据驱动查表"，使前后端共用同一份 locale 数据源。
- 把语言集合从 2 门扩到 6 门（zh-CN / en + ja-JP / de-DE / fr-FR / ru-RU），翻译初版由 AI 一次性生成。
- 让 `FALLBACK_LOCALE` 从 `"zh-CN"` 切到 `"en"`，确立"英文为国际兜底"的产品定位。
- 建立"社区提 issue → opencode 自动改 JSON → 维护者 review merge"的翻译修正闭环。
- 顺手补齐仓库缺失的 issue 模板体系（bug 反馈为默认推荐，translation-improvement 触发自动化，question 兜底）。
- 把 i18n-audit skill 的对齐维度从 2 语扩到 6 语，并产出结构化缺失清单。

**Non-Goals:**

- 不做阿拉伯语或任何 RTL 语言；RTL 布局是独立工程。
- 不接入 Weblate / Crowdin 等翻译协作平台。
- 不追求 AI 初版翻译完美地道；接受"够用，等社区 PR 修正"。
- 不改前端 `useI18n()` / `translate()` / `resolveLocale()` 的对外签名。
- 不改 `AppConfig.settings.locale` 的字段类型；仅扩大合法取值集合。
- 不改动态语言切换的 UX（沿用现有 `peregrine:locale-changed` 事件）。
- 不引入 `fluent` / `rust-i18n` 等第三方 i18n 库；继续用 `serde_json` + `include_str!`。

## Decisions

### 决策 1：后端翻译数据用 `include_str!` 编译期内嵌，运行时按 locale 查表

**选择**：在 `src-tauri/src/lib.rs` 中通过 `include_str!("../../../src/i18n/locales/<locale>.json")` 把 6 份 JSON 在编译期嵌入二进制；运行时用 `serde_json` 反序列化为 `HashMap<String, String>`（与前端 flatten 后的形态一致），按 `(locale, key)` 查表。

**为什么这样**：

- **单一数据源**：前后端共用 `src/i18n/locales/*.json`，新增语言只改 JSON，Rust 代码零修改。彻底消除"前端加语言是加文件、后端加语言是改代码"的不对称。
- **零运行时文件 IO**：编译期内嵌意味着 release 包不需要携带 locale JSON 作为资源文件；与现有 Tauri 打包流程不冲突。
- **零新依赖**：复用已有的 `serde_json`；不引入 `fluent`（语义化、复数规则强，但 Peregrine 的后端字符串极少且无复数场景，杀鸡用牛刀）或 `rust-i18n`（自带宏 DSL，但锁定一种工作流）。
- **与前端 flatten 对齐**：前端 `localeMap[locale][key]` 是扁平 key→string；后端直接反序列化成相同形态，行为对称，缺 key 回退逻辑可共用同一思路。

**被否决的方案**：

| 方案 | 否决理由 |
|---|---|
| 保留 `BackendLocale` 枚举，只是把 `tr()` match 表写得更长 | 没解决"加语言要改 Rust 代码"的根本问题；6 门语言 × N key 的笛卡尔积 match 不可维护 |
| 用 `fluent` crate | 引入语义化翻译 DSL + 复数规则 + 资源文件加载层，对 7 条字符串过度工程；前后端要同时迁移到 fluent，工作量翻倍 |
| 用 `rust-i18n` crate | 自带 `t!` 宏 + 编译期萃取；但前后端两套 DSL，反而加深不对称 |
| 运行时从磁盘读 JSON | 需要 release 包携带 locale 资源、处理路径与权限；Tauri 的 resource 机制会拖累打包流程 |
| 把 JSON 放进 `crates/config` | 违反分层原则——`peregrine_config` 必须保持纯逻辑、不依赖 UI/平台代码；翻译数据属于应用层 |

### 决策 2：用模块级 `LazyLock<HashMap<&'static str, HashMap<String, String>>>` 承载翻译表

**选择**：用 `std::sync::LazyLock` 在首次访问时反序列化嵌入的 JSON，结构为 `HashMap<locale_id, HashMap<key, value>>`。`translate(locale, key)` 查表顺序：`table[locale][key]` → `table[FALLBACK_LOCALE][key]` → `key.to_string()`。

**为什么这样**：

- **线程安全零成本**：`LazyLock` 是 std 库方案，避免 `once_cell` 依赖；Tauri 多线程访问安全。
- **回退路径明确**：二级回退（locale → 英文 → 原始 key）与前端 `localeMap[resolved][key] ?? localeMap[FALLBACK_LOCALE][key] ?? key` 行为完全一致。
- **避免在 `tr()` 调用点散落回退逻辑**：回退集中在 `translate()` 内部，调用点保持简洁。

**与现有代码的关系**：

```
现有：  tr(current_locale(&state), "target_window_required")
                       ↑ BackendLocale 枚举值
                       
改后：  translate(&current_locale(&state), "target_window_required")
                       ↑ &'static str (locale id)
                       
        current_locale() 返回类型从 BackendLocale 改为 &'static str
```

`BackendLocale` 枚举、`from_str`、`detect`、`tr`、`current_locale`、`detect_locale` **全部移除**，由新的数据驱动函数替代。所有调用点（`lib.rs` 中约 10 处）同步改为新签名。

### 决策 3：locale 检测基于前缀映射表，前后端各自实现但映射对齐

**选择**：

- 后端 `detect_locale()`：取系统 locale（Windows 用 `GetUserDefaultLocaleName`，非 Windows 用 `LANG`/`LC_ALL`/`LC_MESSAGES`）→ 按前缀映射到 6 个合法 locale id → 映射不到则回退到 `FALLBACK_LOCALE = "en"`。
- 前端 `detectLocale()`：取 `navigator.language` → 同样的前缀映射 → 同样的回退。
- 前缀映射表（前后端必须一字不差对齐）：

| 前缀（小写） | locale id |
|---|---|
| `zh` | `zh-CN` |
| `en` | `en` |
| `ja` | `ja-JP` |
| `de` | `de-DE` |
| `fr` | `fr-FR` |
| `ru` | `ru-RU` |
| 其它 | `en`（fallback） |

**为什么这样**：

- **稳定优先于精细**：用户系统是 `de-AT` 也映射到 `de-DE`，避免给 6 语维护 6×N 地区变体。Peregrine 是工具，不是地区敏感型应用。
- **前后端各自实现而非共享代码**：前端是 TS、后端是 Rust，跨语言共享代码不现实；但映射表是数据，可以靠 review + 单元测试保证对齐。
- **单测兜底**：后端 `detect_locale` 加单元测试覆盖所有 7 个分支（6 个映射 + 1 个 fallback）；前端 `detectLocale` 加 vitest（如已配置）或人工 review。

### 决策 4：`FALLBACK_LOCALE` 改为 `"en"`（BREAKING for 开发约定）

**选择**：前端 `src/lib/i18n.tsx:18` 与后端 `src-tauri/src/lib.rs` 的 fallback 常量从 `"zh-CN"` 改为 `"en"`。

**为什么**：

- 这是**产品定位声明**：Peregrine 面向国际用户，英文是最低共同语言。一个 ja-JP 用户在 ja 缺某 key 时看到英文比看到中文友好得多。
- 对终端用户无感（只影响缺 key 时的回退路径，正常情况下所有 key 都齐全）；仅对开发者约定的"缺 key 时显示哪种语言"有变化。

**风险与缓解**：

- 风险：如果 ja/de/fr/ru JSON 由 AI 生成时漏了某些 key，用户会看到英文 fallback。这恰好是可接受的（英文比中文友好，且 i18n-audit 升级后会立即报告缺失）。

### 决策 5：`options.json` label 本地化（修复现存 bug）

**选择**：`src/i18n/options.json` 当前的 `label` 字段（如 `"跟随系统"`）在所有语言下显示中文。修改方案：

- 把 `options.json` 的 `label` 改为 **i18n key**（如 `option.follow_system`），不是直接展示文本。
- 前端 `LANGUAGE_OPTIONS` 渲染时对 `label` 走 `t()`。
- 新增 6 语对应的 `option.*` key 到各 locale JSON。

**为什么**：

- 单一数据源：`options.json` 是结构性数据（locale id + 顺序），label 文案应走和其他 UI 字符串相同的 i18n 通道。
- 避免 `options.json` 自身膨胀成 `options.<locale>.json`。

### 决策 6：i18n-audit skill 升级为 6 语对齐 + 结构化缺失清单

**选择**：

- `.agents/skills/i18n-audit/SKILL.md` 中"双语对齐"维度改为"6 语对齐"：检查 `zh-CN.json` / `en.json` / `ja-JP.json` / `de-DE.json` / `fr-FR.json` / `ru-RU.json` 扁平化后 key 集合是否一致。
- 审查结果除了人类可读报告外，**额外输出一份结构化 JSON**（如 `.agents/skills/i18n-audit/output/missing-keys.json`），格式：

```json
{
  "missing": [
    {
      "key": "common.add",
      "missing_in": ["ja-JP", "fr-FR"],
      "present_in": { "zh-CN": "添加", "en": "Add", "de-DE": "Hinzufügen", "ru-RU": "Добавить" }
    }
  ],
  "extra": [
    { "key": "legacy.foo", "only_in": ["zh-CN"] }
  ]
}
```

- 该 JSON 是 opencode agent（无论是本地还是 CI 中的 `auto-translate` job）直接消费修复的输入。

**为什么**：

- 6 语对齐比双语对齐膨胀 N 倍，人类可读报告不够用；AI agent 需要结构化输入才能批量改 JSON。
- 结构化输出同时服务于两个自动化路径：i18n-audit 跑完 → AI 自动补齐缺失 key；社区提翻译 issue → AI 自动改 JSON 提 PR。

### 决策 7：翻译修正闭环走"issue 模板标签触发 opencode job"

**选择**：`.github/workflows/opencode.yml` 新增 `auto-translate` job：

```
┌─────────────────────────────────────────────────────────────┐
│   触发链路                                                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  用户提交 translation-improvement.yml issue                  │
│              │                                               │
│              ▼                                               │
│  issues.opened 事件                                          │
│              │                                               │
│              ├──── auto-label job 给 issue 打 'translation'  │
│              │      （auto-label 的 prompt 扩展此规则）        │
│              │                                               │
│              ▼                                               │
│  auto-translate job                                          │
│    if: label == 'translation'                                │
│    steps:                                                    │
│      - 读 issue body 解析表单字段                             │
│      - opencode prompt:                                      │
│          "改 src/i18n/locales/<locale>.json 的 <key>，        │
│           把 <当前译文> 替换为 <建议译文>，                    │
│           开分支、提交、开 PR，Closes #N"                     │
│      - PR body 引用原 issue                                  │
│              │                                               │
│              ▼                                               │
│  维护者 review & merge                                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**为什么走标签触发而不是直接模板名过滤**：

- `auto-label` job 已经存在，扩展它的 prompt 规则成本最低（加一行"含 translation-improvement 表单字段 → 加 `translation` 标签"）。
- `auto-translate` 的 `if` 条件用标签比用模板名稳定（GitHub Actions 直接拿 `github.event.label.name`，不用解析 issue body）。
- 未来如果想加别的自动化（比如 `bug` 标签触发别的 opencode 任务），同样的标签模式可复用。

**`auto-label` 扩展**：现有 prompt 的"可用标签"列表新增 `translation`，规则新增一条：

```
- translation：使用 translation-improvement 模板提交的翻译改进建议
```

### 决策 8：`blank_issues_enabled: false` + `question.yml` 兜底

**选择**：

- `.github/ISSUE_TEMPLATE/config.yml` 设置 `blank_issues_enabled: false`。
- 新增 `question.yml` 作为非 bug / 非翻译类的兜底入口（用法咨询、功能讨论、杂项）。
- 字段精简：问题描述、已尝试的操作、相关上下文。

**为什么**：

- 强制走模板保证所有 issue 都带结构化信息，不破坏翻译自动化闭环（翻译自动化强依赖表单字段）。
- 兜底模板避免"用户想提问但找不到入口"的死路。
- 不在本次加 `feature_request.yml`：目前 feature 走 OpenSpec change，不走 GitHub issue；如果将来需要再加。

### 决策 9：4 份新 locale JSON 由 AI 一次性生成、中文源交叉校对、提交后人工 review

**选择**：

- 实施阶段用 AI 翻译为 ja-JP / de-DE / fr-FR / ru-RU。
- 翻译策略：**以 `en.json` 为翻译源**（英文歧义少于中文，AI 翻译准确率最稳定），**以 `zh-CN.json` 为原始语义源做交叉校对**——因为中文是项目的原始语言，承载最初的产品语义。
- 保留 key 集合完全一致，保留插值占位符（如 `{name}`）原样。
- 提交后由维护者 review（不强制请母语者），接受"有翻译不保证完全地道"，留待社区通过 translation-improvement 模板修正。

**为什么"以 en 为翻译源 + 以 zh-CN 为校对源"双源策略**：

- **en 为翻译源**：英文作为源语言时，AI 翻译的语种覆盖与准确率最稳定（主流 LLM 的训练语料英文占比最高）；与决策 4 的 `FALLBACK_LOCALE = "en"` 心智一致。
- **zh-CN 为校对源**：中文是项目的原始语言（`zh-CN.json` 是最初撰写的文案），承载产品意图；AI 从英文回译时可能丢失中文原文的细微语义（如"锚点"→"anchor"→ 译回时丢失"视觉参照"的隐喻）。校对阶段把每条 en 源译文与 zh-CN 原始文案并排对比，catch 英文回译引入的语义偏移。
- 校对是实施期的一次性步骤，不是持续流程；后续社区修正走 translation-improvement 模板。

### 决策 10：翻译 PR 加 CI 校验门

**选择**：在 `.github/workflows/` 新增（或扩展 `ci.yml`）一个轻量 job，对所有修改 `src/i18n/locales/*.json` 的 PR 触发校验：

1. **JSON 可解析**：每份被修改的 locale JSON MUST 能被 `serde_json::from_str` 成功反序列化。
2. **Key 集合不变**（针对翻译修正 PR）：PR 修改前后该 locale JSON 的扁平化 key 集合 MUST 完全一致——只允许改 value，不允许加/删 key（加新 key 必须先动 `zh-CN.json` + `en.json` 并配套所有 6 语）。
3. **6 语 key 集合对齐**：PR 合并前后，6 份 locale JSON 扁平化后的 key 集合 MUST 仍完全一致。

校验失败则 CI 红灯，阻塞 PR 合并（维护者本来就手动 review，CI 只是兜底机械检查）。

**为什么**：

- **翻译 PR 已经走手动 review**，加 CI 不增加任何流程延迟——维护者点 merge 前多一道机器校验，是纯增益。
- **opencode 自动 PR 是 AI 生成**，可能改错 key 名、破坏 JSON 结构、误删相邻 key——CI 能在 review 前就把这些机械错误挡掉，节省维护者时间。
- **Key 集合不变约束**防止"只改一门语言漏了其它 5 门"的常见 i18n 失误，与 i18n-audit skill 的 6 语对齐维度互补（skill 是人工触发，CI 是 PR 触发）。

**实现选择**：

- 用一个小 Rust 测试或独立脚本 `scripts/check-i18n.rs`，输入 PR diff 前后的 JSON，跑三个校验。
- 不引入第三方 i18n CI 工具——校验逻辑简单，自维护脚本足够。

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| **AI 翻译不地道** 引发用户反感（尤其 ja/de 母语者） | 接受权衡；issue 模板 + opencode 闭环让修正成本低；在 README/docs 显著位置声明"翻译由 AI 初版，欢迎母语者修正" |
| **前后端 locale 映射表不同步** 导致用户选 ja 前端显示日文、后端 tray 显示英文 | 后端补单元测试覆盖 7 个映射分支；前端补对照测试；前后端映射表写注释互指对方位置 |
| **`BackendLocale` 全量替换** 漏改调用点导致编译失败或运行 panic | Rust 编译器是保障——枚举删除后所有调用点会立即报错；用 `cargo check` 与 `cargo clippy` 兜底，CI 已有 `-D warnings` |
| **`include_str!` 路径相对 crate 根**，构建目录结构变动会导致编译失败 | 路径以 `src-tauri/` 为锚点（`../../../src/i18n/locales/...`），加注释说明；CI 三平台构建会立刻发现路径问题 |
| **opencode 自动 PR 改错 JSON**（比如改错 key、破坏 JSON 结构、误删相邻 key） | 维护者必须 review，不直接 merge；PR 描述引用 issue 让用户可见；**CI 加 JSON 可解析 + key 集合不变 + 6 语对齐三重校验**（见决策 10），机器兜底挡住机械错误 |
| **用户提翻译 issue 但 opencode API 失败/超时** | job 失败时留下 issue 不处理，维护者手工兜底；不阻塞用户（issue 仍然在） |
| **强制模板 (`blank_issues_enabled: false`)** 让某些用户感到受限 | 提供 `question.yml` 兜底；config.yml 可加 `contact_links` 指向 docs / discussions |
| **i18n-audit skill 升级** 后老报告格式不再兼容 | 老报告是一次性产物；升级时直接重跑即可，不需要迁移 |
| **bundle 体积 +66KB** | 可忽略（已评估）；release profile 已开 `opt-level="z"` + lto |

## Migration Plan

本 change 不涉及用户配置迁移——`AppConfig.settings.locale` 仍是 `String`，老配置的 `"auto"` / `"zh-CN"` / `"en"` 继续合法。

**部署步骤**（一次性发版完成）：

1. 合并本 change 的实施 PR（含 6 份 JSON、后端重构、前端扩展、issue 模板、workflow 扩展、i18n-audit skill 升级、`openspec/specs/i18n-audit/spec.md` 更新）。
2. 打 stable release tag（按项目惯例奇数版本号），CI 构建 NSIS + portable zip。
3. 老 Tauri 用户通过 auto-updater 升级到新版本（NSIS 安装用户）。
4. 升级后：用户原 locale 设置（`"auto"` / `"zh-CN"` / `"en"`）继续工作；新语言需在设置中手动切换或靠 `auto` 检测（如系统是 de/DE 则自动切德文）。

**回滚策略**：

- 若发现重大翻译错误或后端重构引入 bug：revert 实施 PR，重新打 patch release。
- `FALLBACK_LOCALE` 切换是单向的——回滚后回到 `"zh-CN"`，对用户无感（缺 key 才走 fallback）。

## Open Questions

1. **`options.json` label 改为 i18n key 后，`LANGUAGE_OPTIONS` 在 `<select>` 渲染时如何避免循环依赖？** label 走 `t()` 意味着切换语言时下拉框 label 跟着变——这是预期行为，但要验证 React 渲染顺序（`options.json` import 在 i18n context 之内）。
2. **opencode `auto-translate` job 的 GITHUB_TOKEN 权限** 是否足以开 PR、推分支？现有 `auto-label` job 已用 `pull-requests: write` + `contents: read`，`auto-translate` 需要 `contents: write` 才能推分支——需要在 workflow 显式声明。
3. **`tray.window_mode` 这条 key 在新 6 语中如何翻译？** 它对应 Tauri tray 的"窗口模式"菜单项——需要确认各语言的 Windows 习惯译法（如 ja: "ウィンドウモード"、de: "Fenstermodus"）。AI 翻译时统一处理，但 review 阶段需要人工 spot check。
