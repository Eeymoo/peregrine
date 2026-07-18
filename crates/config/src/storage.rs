//! 配置文件持久化：路径管理、原子读写、默认配置生成。
//!
//! 配置文件存放于 OS 标准配置目录下的 `Peregrine/config.json`。

use crate::schema::{AppConfig, MaterialRef};
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
    ///
    /// **迁移**：若检测到旧格式（含 `crosshair` 字段且 `layers` 为空），
    /// 自动调用 `migration::migrate_profile` 把旧 Crosshair 转换为新 layers，
    /// 备份原文件为 `<name>.legacy.bak`，写入新格式。
    pub async fn load_or_create_default(&self) -> crate::Result<AppConfig> {
        if !self.config_path.exists() {
            let default = AppConfig::default_config();
            default.validate()?;
            self.save(&default).await?;
            return Ok(default);
        }
        match self.load().await {
            Ok(config) => {
                // 检测并执行旧格式迁移。
                let migrated = self.migrate_if_needed(config).await?;
                Ok(migrated)
            }
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

    /// 检测配置是否为旧格式（含 crosshair 字段），若是则迁移。
    ///
    /// 迁移过程：
    /// 1. 备份原文件为 `<name>.legacy.bak`
    /// 2. 调用 `migration::migrate_profile` 把每个 profile 的 crosshair 转为 layers
    /// 3. 写入新格式配置
    async fn migrate_if_needed(&self, mut config: AppConfig) -> crate::Result<AppConfig> {
        let needs_migration = config.profiles.values().any(|p| {
            p.crosshair.is_some() && p.layers.is_empty()
        });

        if !needs_migration {
            return Ok(config);
        }

        tracing::info!("检测到旧格式配置（含 crosshair 字段），开始迁移到新 layers 格式");
        self.backup_legacy_config().await;

        for (_name, profile) in config.profiles.iter_mut() {
            if profile.crosshair.is_some() && profile.layers.is_empty() {
                let migrated = crate::migration::migrate_profile(profile);
                *profile = migrated;
            }
        }

        config.validate()?;
        self.save(&config).await?;
        tracing::info!("配置迁移完成，新格式已写入");
        Ok(config)
    }

    /// 把旧格式配置文件备份为 `<name>.legacy.bak`（区别于损坏文件的 `.bak`）。
    async fn backup_legacy_config(&self) {
        let mut backup = self.config_path.clone();
        let file_name = backup
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();
        let mut backup_name = file_name;
        backup_name.push(".legacy.bak");
        backup.set_file_name(backup_name);

        match tokio::fs::rename(&self.config_path, &backup).await {
            Ok(()) => tracing::info!("已将旧格式配置备份到 {}", backup.display()),
            Err(e) => tracing::warn!("备份旧格式配置失败: {}", e),
        }
    }

    /// 把无法解析的配置文件重命名为 `<name>.bak`，尽力而为，失败仅告警。
    ///
    /// 采用重命名而非删除，避免丢失用户既有配置，便于事后人工恢复。
    async fn backup_broken_config(&self) {
        let mut backup = self.config_path.clone();
        let file_name = backup
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();
        let mut backup_name = file_name;
        backup_name.push(".bak");
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
        let temp_path = parent.join(format!(".config.tmp.{}", std::process::id()));
        tokio::fs::write(&temp_path, content).await?;
        tokio::fs::rename(&temp_path, &self.config_path).await?;
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
            // Linux / FreeBSD 等：遵循 XDG Base Directory。
            if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
                Some(PathBuf::from(xdg))
            } else {
                home_dir()?.join(".config").into()
            }
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

        let mut cfg = AppConfig::legacy_default_config();
        cfg.active_profile_mut()
            .unwrap()
            .crosshair
            .as_mut()
            .unwrap()
            .opacity = 2.0;
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

    #[tokio::test]
    async fn load_or_create_default_migrates_legacy_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let storage = ConfigStorage::new(&path);

        // 写入一份旧格式配置（含 crosshair.style=cross，无 layers）。
        let legacy_json = r#"{
            "active_profile": "default",
            "profiles": {
                "default": {
                    "crosshair": {
                        "style": "cross",
                        "size": 50.0,
                        "secondary_size": 48.0,
                        "thickness": 3.0,
                        "radius": 0.0,
                        "offset": 0.0,
                        "color": [1.0, 0.0, 0.0, 1.0],
                        "opacity": 0.8,
                        "gap": 6.0,
                        "corner_radius": 4.0,
                        "anchor": "center",
                        "margin": 0.0,
                        "ring_radius_pct": 0.05,
                        "ring_style": "solid",
                        "orb_positions": 3,
                        "random_mode": "lock_on_start",
                        "random_center_deviation": 0.2,
                        "random_radius_min": 4.0,
                        "random_radius_max": 12.0,
                        "random_orb_x": 0.0,
                        "random_orb_y": 0.0,
                        "border_frame_style": "solid",
                        "border_inset": true,
                        "custom_orb_top_count": 3,
                        "custom_orb_bottom_count": 3,
                        "custom_orb_left_count": 3,
                        "custom_orb_right_count": 3,
                        "random_orb_count": 3,
                        "random_orb_offset": 100.0,
                        "random_orb_jitter": 40.0
                    },
                    "trigger": { "enabled": true, "process_names": [] },
                    "settings_hotkey": "F10",
                    "target_window": ""
                }
            }
        }"#;
        tokio::fs::write(&path, legacy_json).await.unwrap();

        // 加载时应自动迁移。
        let cfg = storage.load_or_create_default().await.unwrap();

        // 默认 profile 现在应该有 layers，且 crosshair 应为 None。
        let profile = cfg.profiles.get("default").unwrap();
        assert!(profile.crosshair.is_none(), "crosshair should be None after migration");
        assert_eq!(profile.layers.len(), 1, "should have 1 layer after migration");

        // 图层引用 builtin.cross 物料。
        let layer = &profile.layers[0];
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.cross".to_string()
            }
        );

        // 参数应保留旧值（size=50, thickness=3, gap=6）。
        let params = layer.params.as_object().unwrap();
        assert_eq!(params.get("size").unwrap(), 50.0);
        assert_eq!(params.get("thickness").unwrap(), 3.0);
        assert_eq!(params.get("gap").unwrap(), 6.0);

        // 颜色与不透明度应保留。
        assert_eq!(layer.style.color, [1.0, 0.0, 0.0, 1.0]);
        assert!((layer.style.opacity - 0.8).abs() < 1e-6);

        // 原文件应备份为 .legacy.bak。
        let backup = dir.path().join("config.json.legacy.bak");
        assert!(backup.exists(), "旧配置应备份为 .legacy.bak");

        // 新格式配置已写入，可重新加载。
        let reloaded = storage.load().await.unwrap();
        let p = reloaded.profiles.get("default").unwrap();
        assert!(p.crosshair.is_none());
        assert_eq!(p.layers.len(), 1);
    }

    #[tokio::test]
    async fn load_or_create_default_no_migration_for_new_format() {
        // 新格式配置（含 layers、无 crosshair）不应触发迁移。
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let storage = ConfigStorage::new(&path);

        let new_config = AppConfig::default_config();
        storage.save(&new_config).await.unwrap();

        let loaded = storage.load_or_create_default().await.unwrap();
        let profile = loaded.profiles.get("default").unwrap();
        assert!(profile.crosshair.is_none());
        assert_eq!(profile.layers.len(), 1);

        // 不应生成 .legacy.bak 文件。
        let backup = dir.path().join("config.json.legacy.bak");
        assert!(!backup.exists());
    }
}
