## ADDED Requirements

### Requirement: 后端翻译表数据驱动加载

`src-tauri` 二进制层 MUST 通过 `include_str!` 在编译期将前端共享的 locale JSON（`src/i18n/locales/<locale>.json`，共 6 份：zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）嵌入二进制；运行时通过 `serde_json` 反序列化为 `HashMap<locale_id, HashMap<key, value>>` 形态的翻译表。`src-tauri` MUST NOT 再保留任何"语言枚举 + match 翻译"形式的硬编码翻译表。

#### Scenario: 加载后所有 6 语翻译可读

- **WHEN** 应用启动完成
- **THEN** 6 门语言的所有 key 均可由翻译表查询返回对应译文，不依赖任何运行时磁盘 IO

#### Scenario: 加一门新语言无需改 Rust 翻译代码

- **WHEN** 维护者在 `src/i18n/locales/` 新增一份合规 JSON 并在加载列表中注册
- **THEN** 后端无需修改 `src-tauri/src/lib.rs` 的翻译查表逻辑即可在新语言下返回该 JSON 中的译文

### Requirement: 后端翻译查表与回退

后端 MUST 提供 `translate(locale: &str, key: &str) -> String` 函数作为唯一翻译入口。查表顺序 MUST 为：当前 locale 命中 → `FALLBACK_LOCALE`（`"en"`）命中 → 返回原始 key 字符串。该回退顺序 MUST 与前端 `src/lib/i18n.tsx` 的 `localeMap[resolved][key] ?? localeMap[FALLBACK_LOCALE][key] ?? key` 行为完全一致。

#### Scenario: 当前 locale 命中

- **WHEN** 调用 `translate("ja-JP", "tray.settings")` 且 `ja-JP.json` 包含该 key
- **THEN** 返回日文翻译

#### Scenario: 当前 locale 缺 key 时回退英文

- **WHEN** 调用 `translate("ja-JP", "some.key")` 且 `ja-JP.json` 缺该 key 但 `en.json` 存在
- **THEN** 返回英文翻译

#### Scenario: 英文也缺 key 时返回原始 key

- **WHEN** 调用 `translate("ja-JP", "unknown.key")` 且所有 locale 均无此 key
- **THEN** 返回字符串 `"unknown.key"`

### Requirement: 废除 BackendLocale 硬编码枚举

`src-tauri/src/lib.rs` 中的 `BackendLocale` 枚举、`BackendLocale::from_str`、`BackendLocale::detect`、`tr(locale, key)` match 表 MUST 被完全移除；其所有调用点（`current_locale`、`detect_locale`、tray 构建、`start_overlay` 校验、mode 切换提示等）MUST 改为使用返回 `&'static str`（locale id）的新函数 + `translate()` 组合。

#### Scenario: 移除枚举后所有调用点仍可编译

- **WHEN** `BackendLocale` 被删除后执行 `cargo check -p peregrine-tauri`
- **THEN** 编译通过，无残留引用错误

#### Scenario: tray 菜单在新 locale 下显示对应翻译

- **WHEN** 用户切换 `AppConfig.settings.locale` 为 `de-DE` 并触发 tray 菜单重建
- **THEN** tray 菜单项显示德文译文（如 "Einstellungen"），而非中文或英文 fallback

### Requirement: 统一 locale 检测的前后端映射表

前端 `src/lib/i18n.tsx` 的 `detectLocale()` 与后端 `src-tauri/src/lib.rs` 的 `detect_locale()` MUST 共享同一份前缀映射语义：系统 locale 小写前缀为 `zh`→`zh-CN`、`en`→`en`、`ja`→`ja-JP`、`de`→`de-DE`、`fr`→`fr-FR`、`ru`→`ru-RU`，其它前缀回退到 `FALLBACK_LOCALE`。前端基于 `navigator.language`，后端基于 Win32 `GetUserDefaultLocaleName`（Windows）或环境变量 `LANG`/`LC_ALL`/`LC_MESSAGES`（非 Windows）。

#### Scenario: 系统为 de-AT 时前后端均映射到 de-DE

- **WHEN** 用户系统 locale 为 `de-AT`（奥地利德语）
- **THEN** 前端 `detectLocale()` 与后端 `detect_locale()` 均返回 `"de-DE"`，前端 UI 与后端 tray 显示一致的德文翻译

#### Scenario: 系统为未支持语言时回退英文

- **WHEN** 用户系统 locale 为 `ko-KR`（不在 6 语映射表中）
- **THEN** 前后端检测均返回 `"en"`

### Requirement: Fallback 语言改为英文

前端 `src/lib/i18n.tsx` 的 `FALLBACK_LOCALE` 与后端对应常量 MUST 设为 `"en"`（此前为 `"zh-CN"`）。这是面向开发约定的 BREAKING 变更，对终端用户无感（仅影响缺 key 时的回退路径）。

#### Scenario: 中文用户在某 key 缺失时看到英文而非中文

- **WHEN** 用户 locale 为 `zh-CN` 且某 key 在 `zh-CN.json` 缺失但 `en.json` 存在
- **THEN** 翻译函数返回英文译文（而非报错或显示原始 key）

### Requirement: options.json 标签本地化

`src/i18n/options.json` 中 `LANGUAGE_OPTIONS` 的 `label` 字段 MUST 改为 i18n key（如 `option.follow_system`），MUST NOT 直接存储翻译文案；前端 `LANGUAGE_OPTIONS` 渲染时 MUST 对 label 走 `t()` 翻译。各 locale JSON MUST 新增对应的 `option.*` key。

#### Scenario: 切换语言时下拉框 label 跟随

- **WHEN** 当前 locale 为 `fr-FR`，用户展开语言下拉框
- **THEN** "跟随系统"选项显示为法文译文（如 "Suivre le système"），而非中文

### Requirement: 6 门目标语言 locale 文件齐全

`src/i18n/locales/` MUST 提供 6 份 JSON：`zh-CN.json` / `en.json` / `ja-JP.json` / `de-DE.json` / `fr-FR.json` / `ru-RU.json`。6 份 JSON 扁平化后的 key 集合 MUST 完全一致；翻译文案由 AI 一次性生成，接受"有翻译不保证完全地道"，后续由社区修正。

#### Scenario: 6 语 key 集合一致

- **WHEN** 对 6 份 JSON 扁平化后比较 key 集合
- **THEN** 集合完全一致，无 key 缺失或冗余

#### Scenario: 插值占位符在所有语言下保留

- **WHEN** 某 key 在 `en.json` 含 `{name}` 等插值占位符
- **THEN** 其它 5 份 JSON 对应 key 的译文 MUST 保留相同的占位符
