---
title: "开发构建"
---

## 环境要求

- Rust 1.85 或更高版本（edition 2024）
- Node.js 20 或更高版本（前端构建）
- Windows SDK（用于 Win32 API 与 `windows` crate）
- Cargo

## 克隆仓库

```bash
git clone https://github.com/eeymoo/peregrine.git
cd peregrine
```

## 构建

```bash
# 安装前端依赖
npm install

# 调试构建
cargo build

# 发布构建（体积小、性能高）
cargo build --release

# 运行 Tauri 开发版本（带热更新）
npx tauri dev

# 构建 Tauri release 安装包
npx tauri build
```

## 测试

```bash
# 运行全部测试
cargo test

# 只运行配置库测试
cargo test -p peregrine_config
```

## 代码检查

```bash
cargo fmt
cargo clippy -p peregrine_config -- -D warnings
```

## 发布产物

`npx tauri build` 生成的 release 产物位于 `src-tauri/target/release/` 目录下，MSI 安装包位于 `src-tauri/target/release/bundle/msi/`。

发布版本的编译选项已针对体积与性能优化：

- `opt-level = "z"`
- `lto = true`
- `codegen-units = 1`
- `strip = true`
- `panic = "abort"`

## 国际化（i18n）

Peregrine 支持 6 门 UI 语言（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）。前后端各自实现了一份 `detectLocale` / `map_locale_prefix`，但**前缀映射表必须前后端一字不差对齐**：

- 前端：`src/lib/i18n.tsx` → `mapLocalePrefix()`（读取 `navigator.language`）。
- 后端：`src-tauri/src/lib.rs` → `map_locale_prefix()`（Windows 上读取 `GetUserDefaultLocaleName`，非 Windows 读取 `LANG`/`LC_ALL`/`LC_MESSAGES`）。

两处使用同一份前缀→locale id 映射：`zh`→`zh-CN`、`en`→`en`、`ja`→`ja-JP`、`de`→`de-DE`、`fr`→`fr-FR`、`ru`→`ru-RU`，其它前缀→`FALLBACK_LOCALE = "en"`。后端有单元测试（`tests::map_locale_prefix_covers_all_supported_branches`）镜像每个分支——新增语言时，**同时更新两处映射并扩展测试**。

locale JSON 放在 `src/i18n/locales/`；前端（`localeMap` 导入）与后端（`include_str!` 内嵌）共用同一份数据源。需要审查覆盖度 / key 对齐时，运行 `i18n-audit` skill；`scripts/check-i18n.mjs` CI 校验门会强制翻译类 PR 不加 / 删 key。

## 遥测开发

Peregrine 集成了匿名 GlitchTip（Sentry 协议）遥测：崩溃上报、启动统计、关键路径错误上报。用户向隐私说明见 [隐私与遥测](./privacy)；面向开发者的全部上报 Code 登记表见仓库根目录的 [`REPORT_CODES.md`](./report-codes)。

### 模块定位

| 层 | 文件 | 职责 |
|---|---|---|
| Rust 后端 | `src-tauri/src/telemetry.rs` | install_id 生命周期、pending 存储、事件脱敏、panic hook、`report_code` 常量、启动 / `safe_try!` 错误出口 |
| 前端 | `src/lib/telemetry.ts` | `REPORT_CODES` 常量、`initTelemetry`、`captureFrontendError`、pending / authorize Tauri command 封装 |
| 构建胶水 | `src-tauri/build.rs` | 从 `.env.development` 注入 DSN、发出 `peregrine_disable_telemetry` cfg |

### DSN 环境变量

DSN 在 **构建期** 注入，运行时不读盘：

| 构建模式 | 环境变量 | 来源 |
|---|---|---|
| `dev` / debug | `GLITCHTIP_DSN_TEST`（Rust）/ `VITE_GLITCHTIP_DSN_TEST`（前端） | 仓库根 `.env.development`（已被 gitignore，按需从 `.env.development.example` 复制）或外部环境 |
| `release` | `GLITCHTIP_DSN`（Rust）/ `VITE_GLITCHTIP_DSN`（前端） | CI GitHub Secrets |
| snapshot | 同 release 但指向 TEST 项目 | 工作流把 `GLITCHTIP_DSN_TEST` 映射为 `GLITCHTIP_DSN` |

`src-tauri/build.rs` 解析 `.env.development` 并把 `GLITCHTIP_DSN_TEST` 透传到编译期环境，供 `telemetry.rs::dsn()` 中的 `option_env!("GLITCHTIP_DSN_TEST")` 读取。前端通过 Vite 的 `import.meta.env` 读取。

**无 DSN 本地调试**：若 `.env.development` 与外部环境都未设置，Rust `dsn()` 与前端 `TELEMETRY_DSN_AVAILABLE` 均返回 `None`/`false`，SDK 不初始化，应用 **零网络请求**，遥测相关 UI 自动隐藏。这是新克隆仓库的默认状态。

> 注意：修改 `.env.development` 后增量编译可能不会重编译使用 `option_env!` 的 Rust 源码，需要 `cargo clean -p peregrine-tauri`（或 `touch` 任一 Rust 源文件）后再构建。

### `PEREGRINE_DISABLE_TELEMETRY`（编译期禁用）

若需让产物 **完全不含上报代码**，构建前设置环境变量（任意值均可）：

```bash
PEREGRINE_DISABLE_TELEMETRY=1 cargo build --release
PEREGRINE_DISABLE_TELEMETRY=1 npx tauri build
```

`build.rs` 发出 `peregrine_disable_telemetry` cfg，整个 `telemetry` 模块编译为 no-op 桩：API 签名保留但内部无 IO、无网络、不注册 panic hook。

### `safe_try!` 使用约定

`safe_try!($expr, $code)`（定义在 `src-tauri/src/lib.rs`）包装任意返回 `Result` 的调用：

- **Ok** → 直通，不上报。
- **Err** → 携带函数名 + 调用点文件:行号 + 脱敏后消息（消息经 `anonymize_text` 处理），调用 `telemetry::report_safe_try_error` 上报：
  - SDK 激活 → sentry Error 级事件（tag：`code` / `event_type=error` / `priority=p2` / `function` / `location`）。
  - SDK 未激活 → 改为落盘本地 pending，零网络。
- 原始 `Err` **原样返回**，调用方可以继续降级处理。

**仅关键路径使用，禁止滥用**：文件 IO、渲染入口、窗口桥接、外部调用等。不要把普通方法全部包裹，会稀释信号、淹没 issue 列表。传入的 Code **必须** 先在 `REPORT_CODES.md` 与 `report_code` 模块登记。

目前已接线的 `safe_try!` Code：`PGR-2101`（配置 IO，`lib.rs` 4 处）、`PGR-4101`（overlay 渲染，`overlay.rs` 渲染循环）。其余声明但未接线的 Code（`PGR-2102` / `2401` / `2501` / `4102` / `4201` / `4202` / `51xx`）在 `REPORT_CODES.md` 中标注为预留。

### Code 登记治理流程

新增上报点之前：

1. 在对应号段（详见 `REPORT_CODES.md` 号段表）选取一个未占用的 Code。
2. **同一 PR 内** 把常量加入 `report_code`（Rust）或 `REPORT_CODES`（前端），并在 `REPORT_CODES.md` 增加一行。
3. 然后才编写 `safe_try!` / `capture_message` / `captureFrontendError` 调用点。
4. 把登记表中的「接线状态」更新为 ✅ 已接线，并填写实际触发点（函数 / 文件）。

PR 中不允许出现绕过登记表的硬编码 Code。Code 一旦发布即稳定，**不得** 改号或复用。

### 前端接线

前端在启动时初始化 Sentry（`src/main.tsx`）：

```ts
const telemetryEnabled = config.settings.telemetry_enabled === true;
initTelemetry(telemetryEnabled);
```

仅当 flag 为 `true` **且** `TELEMETRY_DSN_AVAILABLE` 为 `true` 时 `initTelemetry` 才真正初始化。浏览器侧同样运行与 Rust 一致的 `beforeSend` 脱敏。

错误统一经 `captureFrontendError(code, error, tags?)` 出口：

- SDK 已初始化 → 携带 `code` tag 的 sentry 事件。
- SDK 未初始化 → 走 `store_pending_report` Tauri command 落盘。

首次启动的授权弹窗（`ConfigApp.tsx`）与报错页面的一次性「匿名上传错误报告」按钮（`ErrorBoundary.tsx`）是仅有的两处会修改 `telemetry_enabled` 或调用 `authorizeUploadAll` 的地方，把它们视作用户面；其余都是即发即忘的诊断出口。
