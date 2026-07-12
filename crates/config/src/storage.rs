//! 配置文件持久化：路径管理、原子读写、默认配置生成。
//!
//! 配置文件存放于 OS 标准配置目录下的 `Peregrine/config.json`。

use crate::schema::AppConfig;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;

/// 临时文件名计数器，保证同进程并发 save 使用不同临时文件。
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 配置文件读写器。
///
/// 通过 [`ConfigStorage::new`] 传入路径，或 [`ConfigStorage::default_path`] 使用 OS 标准目录。
#[derive(Debug, Clone)]
pub struct ConfigStorage {
    config_path: PathBuf,
}

impl ConfigStorage {
    /// 使用显式路径创建存储。
    ///
    /// 若路径的父目录已存在，则把父目录解析为规范路径（canonical），再拼回文件名。
    /// 这用于兼容 macOS 上 `/var` 到 `/private/var` 等符号链接导致的 notify 路径不一致
    /// 问题，同时允许目标文件本身尚不存在。
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let config_path = path
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .map(|canonical_parent| canonical_parent.join(path.file_name().unwrap_or_default()))
            .unwrap_or(path);
        Self { config_path }
    }

    /// 获取 OS 标准配置目录下的默认配置文件路径。
    ///
    /// - macOS: `~/Library/Application Support/Peregrine/config.json`
    /// - Windows: `%APPDATA%/Peregrine/config.json`
    /// - Linux: `~/.config/Peregrine/config.json`
    pub fn default_path() -> crate::Result<PathBuf> {
        let dir = dirs::config_dir()
            .ok_or_else(|| crate::ConfigError::Validation("config dir not found".to_string()))?;
        Ok(dir.join("Peregrine").join("config.json"))
    }

    /// 使用默认路径创建存储。
    pub fn with_default_path() -> crate::Result<Self> {
        Ok(Self::new(Self::default_path()?))
    }

    /// 配置文件路径。
    pub fn path(&self) -> &Path {
        &self.config_path
    }

    /// 从磁盘读取配置；若文件不存在则创建默认配置并写入。
    ///
    /// 若现有配置文件无法解析或校验失败（通常是版本不兼容或被手动改坏），
    /// 不再直接报错，而是**备份损坏文件**后回退到默认配置并重新写入，
    /// 保证程序始终能启动。写入默认配置前会先校验其合法性。
    pub async fn load_or_create_default(&self) -> crate::Result<AppConfig> {
        if !self.config_path.exists() {
            let default = AppConfig::default_config();
            default.validate()?;
            self.save(&default).await?;
            return Ok(default);
        }
        match self.load().await {
            Ok(config) => Ok(config),
            Err(e) => {
                tracing::warn!(
                    "配置文件不兼容或已损坏，将备份原文件并重置为默认配置: {}",
                    e
                );
                self.backup_broken_config().await;
                let default = AppConfig::default_config();
                default.validate()?;
                self.save(&default).await?;
                Ok(default)
            }
        }
    }

    /// 把无法解析的配置文件重命名为 `<name>.bak`，尽力而为，失败仅告警。
    ///
    /// 采用重命名而非删除，避免丢失用户既有配置，便于事后人工恢复。
    /// 若 `.bak` 已存在，则追加数字后缀（`.bak.2`、`.bak.3`…）避免覆盖。
    async fn backup_broken_config(&self) {
        let mut backup = self.config_path.clone();
        let file_name = backup
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();

        // 查找第一个不存在的备份文件名。
        let mut backup_name = file_name.clone();
        backup_name.push(".bak");
        let mut seq = 2u32;
        while tokio::fs::metadata(backup.with_file_name(&backup_name))
            .await
            .is_ok()
        {
            backup_name = file_name.clone();
            backup_name.push(format!(".bak.{}", seq));
            seq += 1;
        }
        backup.set_file_name(backup_name);

        match tokio::fs::rename(&self.config_path, &backup).await {
            Ok(()) => tracing::warn!("已将损坏的配置文件备份到 {}", backup.display()),
            Err(e) => tracing::warn!("备份损坏的配置文件失败: {}", e),
        }
    }

    /// 从磁盘读取配置。
    pub async fn load(&self) -> crate::Result<AppConfig> {
        let content = tokio::fs::read_to_string(&self.config_path).await?;
        let config: AppConfig = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// 将配置原子写入磁盘：先写临时文件，再重命名。
    ///
    /// 写入前会先校验合法性，避免把无效配置落盘。
    pub async fn save(&self, config: &AppConfig) -> crate::Result<()> {
        config.validate()?;
        let parent = self
            .config_path
            .parent()
            .ok_or_else(|| crate::ConfigError::Validation("invalid config path".to_string()))?;
        tokio::fs::create_dir_all(parent).await?;

        let content = serde_json::to_string_pretty(config)?;
        // 临时文件与目标文件放在同一目录，保证 rename 原子且跨文件系统可靠。
        // 文件名含 PID + 原子计数器，避免同进程并发 save 时冲突。
        let temp_path = parent.join(format!(
            ".config.tmp.{}.{}",
            std::process::id(),
            TMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        tokio::fs::write(&temp_path, content).await?;
        if let Err(e) = tokio::fs::rename(&temp_path, &self.config_path).await {
            // rename 失败时尽力清理临时文件，避免残留。
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(e.into());
        }
        Ok(())
    }
}

// `dirs` 是一个小巧的跨平台目录查找模块。
//
// 不依赖外部 `dirs` crate，自行实现以保持 config 库零平台依赖。
// 与 OS 标准目录约定一致：
// - macOS: `~/Library/Application Support`
// - Windows: `%APPDATA%`（即 `Roaming`）
// - Linux/其它: `$XDG_CONFIG_HOME`，回退到 `~/.config`
mod dirs {
    use std::path::PathBuf;

    /// 返回平台标准的用户配置目录。
    pub fn config_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            std::env::var_os("APPDATA").map(PathBuf::from)
        }
        #[cfg(target_os = "macos")]
        {
            Some(home_dir()?.join("Library").join("Application Support"))
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            // Linux / FreeBSD 等：遵循 XDG Base Directory 规范。
            // XDG 规范要求：XDG_CONFIG_HOME 必须为绝对路径，否则忽略并回退到 ~/.config。
            if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
                let p = PathBuf::from(&xdg);
                if p.is_absolute() {
                    return Some(p);
                }
            }
            home_dir()?.join(".config").into()
        }
    }

    /// 获取用户 HOME 目录（macOS / Linux）。
    #[cfg(not(target_os = "windows"))]
    fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn roundtrip_default_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let storage = ConfigStorage::new(&path);

        let original = AppConfig::default_config();
        storage.save(&original).await.unwrap();
        let loaded = storage.load().await.unwrap();
        assert_eq!(original, loaded);
    }

    #[tokio::test]
    async fn load_or_create_default_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let storage = ConfigStorage::new(&path);

        assert!(!path.exists());
        let cfg = storage.load_or_create_default().await.unwrap();
        assert!(path.exists());
        assert_eq!(cfg.active_profile, "default");
    }

    #[tokio::test]
    async fn save_invalid_config_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let storage = ConfigStorage::new(&path);

        let mut cfg = AppConfig::default_config();
        cfg.active_profile_mut().unwrap().crosshair.opacity = 2.0;
        assert!(storage.save(&cfg).await.is_err());
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn load_or_create_default_recovers_from_incompatible_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let storage = ConfigStorage::new(&path);

        // 写入一份无法解析的配置（含未知枚举变体 corner_only，且缺少必填字段），
        // 模拟版本不兼容 / 损坏的旧配置文件。
        tokio::fs::write(
            &path,
            r#"{"active_profile":"default","profiles":{"default":{"crosshair":{"style":"border_frame","border_frame_style":"corner_only"}}}}"#,
        )
        .await
        .unwrap();

        // 不再报错：应回退到默认配置并把损坏文件备份为 .bak。
        let cfg = storage.load_or_create_default().await.unwrap();
        assert_eq!(cfg.active_profile, "default");
        // 备份文件已生成。
        let backup = dir.path().join("config.json.bak");
        assert!(backup.exists(), "损坏的配置应被备份为 .bak");
        // 新的默认配置已写回，且可以正常再次加载。
        assert!(path.exists());
        let reloaded = storage.load().await.unwrap();
        assert_eq!(reloaded.active_profile, "default");
    }
}
