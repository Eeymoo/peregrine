//! 配置文件持久化：路径管理、原子读写、默认配置生成。
//!
//! 配置文件存放于 OS 标准配置目录下的 `Peregrine/config.json`。

use crate::schema::AppConfig;
use std::path::{Path, PathBuf};

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
            .map(|canonical_parent| {
                canonical_parent.join(path.file_name().unwrap_or_default())
            })
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
    /// 写入默认配置前会先校验其合法性。
    pub async fn load_or_create_default(&self) -> crate::Result<AppConfig> {
        if !self.config_path.exists() {
            let default = AppConfig::default_config();
            default.validate()?;
            self.save(&default).await?;
            return Ok(default);
        }
        self.load().await
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
        let temp_path = parent.join(format!(
            ".config.tmp.{}",
            std::process::id()
        ));
        tokio::fs::write(&temp_path, content).await?;
        tokio::fs::rename(&temp_path, &self.config_path).await?;
        Ok(())
    }
}

// `dirs` 是一个小巧的跨平台目录库，没有依赖 winit/wgpu。
#[allow(unused_imports)]
mod dirs {
    use std::path::PathBuf;

    /// 返回 OS 标准用户配置目录。
    pub fn config_dir() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var_os("HOME")?;
            Some(PathBuf::from(home).join("Library/Application Support"))
        }
        #[cfg(target_os = "windows")]
        {
            std::env::var_os("APPDATA").map(PathBuf::from)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let home = std::env::var_os("HOME")?;
            Some(PathBuf::from(home).join(".config"))
        }
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
}
