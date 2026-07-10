//! Peregrine 配置库。
//!
//! 职责：定义配置结构、持久化、文件热重载以及变更广播。
//! 不包含任何 UI、GPU 或窗口平台代码。

pub mod notifier;
pub mod schema;
pub mod storage;
pub mod watcher;

pub use notifier::{ConfigNotifier, ConfigSnapshot};
pub use schema::{
    Anchor, AppConfig, AppSettings, BorderFrameStyle, Crosshair, CrosshairStyle, OrbPosition,
    Profile, RandomOrbMode, RingStyle,
};
pub use storage::ConfigStorage;
pub use watcher::ConfigWatcher;

/// 配置库统一错误类型。
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// 文件 IO 失败。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// 序列化/反序列化失败。
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    /// 配置校验失败。
    #[error("validation error: {0}")]
    Validation(String),
    /// 文件系统监听失败。
    #[error("file watcher error: {0}")]
    Watcher(#[from] notify::Error),
    /// 广播通道已关闭。
    #[error("notifier channel closed")]
    NotifierClosed,
}

/// 库级结果类型别名。
pub type Result<T> = std::result::Result<T, ConfigError>;
