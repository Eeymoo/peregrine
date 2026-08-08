//! Tauri 构建脚本。
//!
//! 除标准 tauri-build 外还负责：
//! 1. 解析仓库根 `.env.development`，将 `GLITCHTIP_DSN_TEST` 注入编译期环境
//!    （供遥测模块 `option_env!` 读取；外部环境变量已设置时不覆盖，空值忽略）。
//! 2. 检测 `PEREGRINE_DISABLE_TELEMETRY` 环境变量，发出
//!    `peregrine_disable_telemetry` cfg，使遥测模块整体编译为 no-op 桩。

use std::path::Path;

fn main() {
    // 声明自定义 cfg，避免 unexpected_cfgs 编译警告。
    println!("cargo::rustc-check-cfg=cfg(peregrine_disable_telemetry)");

    // 编译期禁用遥测：设置 PEREGRINE_DISABLE_TELEMETRY（任意值）后构建，
    // 遥测模块编译为空实现，二进制不含任何上报代码路径与网络请求。
    println!("cargo:rerun-if-env-changed=PEREGRINE_DISABLE_TELEMETRY");
    if std::env::var_os("PEREGRINE_DISABLE_TELEMETRY").is_some() {
        println!("cargo:rustc-cfg=peregrine_disable_telemetry");
    }

    // DSN 注入：优先沿用外部环境变量（CI release 注入 GLITCHTIP_DSN）；
    // 本地 dev 未设置 GLITCHTIP_DSN_TEST 时，解析仓库根 .env.development。
    println!("cargo:rerun-if-env-changed=GLITCHTIP_DSN");
    println!("cargo:rerun-if-env-changed=GLITCHTIP_DSN_TEST");

    // 按 cfg!(debug_assertions) 决定本次构建生效的 DSN：
    // - dev：GLITCHTIP_DSN_TEST（本地 .env.development 或 CI dev 环境）
    // - release：GLITCHTIP_DSN（CI release/snapshot 注入）
    // 仅对生效路径做格式校验；未注入（空）时跳过，应用按零网络请求降级。
    let active_key = if cfg!(debug_assertions) {
        "GLITCHTIP_DSN_TEST"
    } else {
        "GLITCHTIP_DSN"
    };
    let active_raw = std::env::var_os(active_key)
        .and_then(|s| s.to_str().map(|str| str.trim().to_string()))
        .unwrap_or_default();
    let active_len = active_raw.len();
    if active_len == 0 {
        println!(
            "cargo:warning=[telemetry-diag] {} not set; telemetry SDK will not initialize (zero network requests)",
            active_key
        );
    } else if let Err(reason) = validate_dsn(&active_raw) {
        // 格式错误：直接 panic 让构建失败，避免产出「看似正常但遥测全废」的二进制。
        // 常见错因：Secret 存的是 GlitchTip Key URL（含 ?glitchtip_key=...）而非标准 Sentry DSN。
        panic!(
            "\n[telemetry] {} 格式非法，构建中止。\n  原因: {}\n  长度: {}\n  正确格式示例: https://<key>@<host>/<project_id>\n  错误格式示例: https://<host>/api/<id>/security/?glitchtip_key=<key>\n  请到 GlitchTip 项目设置复制标准 DSN，更新对应 GitHub Secret 后重试。",
            active_key, reason, active_len
        );
    } else {
        println!(
            "cargo:warning=[telemetry-diag] {} OK len={} (parsed as standard Sentry DSN)",
            active_key, active_len
        );
    }

    let env_file = Path::new("../.env.development");
    println!("cargo:rerun-if-changed={}", env_file.display());
    if std::env::var_os("GLITCHTIP_DSN_TEST").is_none()
        && let Some(value) = read_env_value(env_file, "GLITCHTIP_DSN_TEST")
    {
        println!("cargo:rustc-env=GLITCHTIP_DSN_TEST={value}");
        println!(
            "cargo:warning=[telemetry-diag] GLITCHTIP_DSN_TEST injected from .env.development len={}",
            value.len()
        );
    }

    tauri_build::build();
}

/// 校验 DSN 是否为标准 Sentry DSN 格式：`https?://<key>@<host>/<project_id>`。
///
/// 拒绝 GlitchTip Key URL（`.../api/<id>/security/?glitchtip_key=...`）等非标准格式——
/// sentry crate 与 @sentry/react 只认标准格式，非标准会让 dsn.parse() 失败、SDK 不初始化。
/// 返回 Ok(()) 表示通过；Err(描述) 表示不通过。
fn validate_dsn(dsn: &str) -> Result<(), String> {
    let trimmed = dsn.trim();
    if trimmed.is_empty() {
        return Err("empty".to_string());
    }
    // 标准格式：scheme://credentials@host/path
    // credentials 段必须存在（Sentry DSN 的 public key，32 字符 hex 串）。
    let at_count = trimmed.matches('@').count();
    if at_count != 1 {
        return Err(format!(
            "expected exactly 1 '@' separator, found {} (标准格式为 https://<key>@<host>/<id>)",
            at_count
        ));
    }
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err("must start with http:// or https://".to_string());
    }
    // 拒绝 query string：标准 Sentry DSN 不带 ?key=value 形式（GlitchTip Key URL 会带）。
    if trimmed.contains('?') {
        return Err(
            "contains '?' query string (疑似 GlitchTip Key URL 而非标准 Sentry DSN)".to_string(),
        );
    }
    Ok(())
}

/// 解析简单 `KEY=VALUE` 格式的 env 文件，返回指定键的非空值。
///
/// 忽略注释与空行，不展开变量、不处理引号转义（DSN 为纯 URL，无需复杂解析）。
fn read_env_value(path: &Path, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.trim() == key {
            let v = v.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}
