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

    // 诊断输出（仅长度不泄露值）：
    // - GLITCHTIP_DSN：release 构建读取，CI release/snapshot 通过 option_env! 进入二进制。
    // - GLITCHTIP_DSN_TEST：dev 构建读取，本地 .env.development 注入。
    // CI 日志中可见这两行 warning，用于排查「DSN 是否真的编译进了二进制」。
    let env_dsn = std::env::var_os("GLITCHTIP_DSN")
        .and_then(|s| s.to_str().map(|str| str.len()))
        .unwrap_or(0);
    let env_dsn_test = std::env::var_os("GLITCHTIP_DSN_TEST")
        .and_then(|s| s.to_str().map(|str| str.len()))
        .unwrap_or(0);
    println!(
        "cargo:warning=[telemetry-diag] GLITCHTIP_DSN len={} (release path, option_env! reads)",
        env_dsn
    );
    println!(
        "cargo:warning=[telemetry-diag] GLITCHTIP_DSN_TEST len={} (dev path)",
        env_dsn_test
    );

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
