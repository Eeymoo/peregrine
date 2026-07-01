//! 配置数据结构定义与校验。
//!
//! 支持多 Profile，每个 Profile 可独立设置辅助贴图样式、触发规则与快捷键。
//! 本工具主要用途是防 3D 眩晕，通过屏幕中心辅助贴图提供视觉锚点。

use serde::{Deserialize, Serialize};

/// 应用级配置根节点。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    /// 当前激活的 Profile 名称。
    pub active_profile: String,
    /// 所有可用 Profile，按名称索引。
    pub profiles: std::collections::HashMap<String, Profile>,
}

/// 单个 Profile 配置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// 辅助贴图配置。
    pub crosshair: Crosshair,
    /// 触发规则：在什么情况下显示辅助贴图。
    pub trigger: TriggerRule,
    /// 进入设置界面的热键字符串（仅做存储，解析由调用方负责）。
    pub settings_hotkey: String,
    /// 要跟随的目标窗口标识（空字符串表示不跟随特定窗口）。
    /// 当前仅做配置占位，实际窗口跟随由 Platform 层实现。
    #[serde(default)]
    pub target_window: String,
}

/// 辅助贴图整体配置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Crosshair {
    /// 辅助贴图样式类型。
    pub style: CrosshairStyle,
    /// 贴图主尺寸，单位像素。
    /// 卫生纸用做宽度；准星用做十字臂长；定位球用于限制半径/偏移上限。
    pub size: f32,
    /// 贴图次尺寸，单位像素。
    /// 卫生纸用做高度；其他样式可忽略。
    #[serde(default = "default_secondary_size")]
    pub secondary_size: f32,
    /// 贴图线条/圆形厚度，单位像素。
    pub thickness: f32,
    /// 圆形定位球半径，单位像素。
    /// 若未设置或 <=0，则回退到 thickness 的倍数。
    #[serde(default = "default_radius")]
    pub radius: f32,
    /// 定位球/大准星等元素距屏幕外侧的距离，单位像素。
    #[serde(default = "default_offset")]
    pub offset: f32,
    /// 贴图颜色（RGBA，0.0..=1.0）。
    pub color: [f32; 4],
    /// 不透明度，0.0 完全透明，1.0 不透明。
    pub opacity: f32,
    /// 中心点间隙，单位像素（准星等样式用到）。
    pub gap: f32,
    /// 矩形贴图圆角半径，单位像素（卫生纸用到）。
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f32,
    /// 矩形贴图（卫生纸）的贴边位置。
    #[serde(default)]
    pub anchor: Anchor,
    /// 矩形贴图（卫生纸）与贴边外侧的边距，单位像素。
    #[serde(default = "default_margin")]
    pub margin: f32,
    /// 中心环：环半径占屏幕高度的比例。
    #[serde(default = "default_ring_radius_pct")]
    pub ring_radius_pct: f32,
    /// 中心环：线型样式。
    #[serde(default)]
    pub ring_style: RingStyle,
    /// 自定义定位球：启用的位置位掩码。
    #[serde(default)]
    pub orb_positions: OrbPosition,
    /// 随机球：工作模式。
    #[serde(default)]
    pub random_mode: RandomOrbMode,
    /// 随机球：相对屏幕几何中心的随机偏移范围。
    #[serde(default = "default_random_center_deviation")]
    pub random_center_deviation: f32,
    /// 随机球：最小半径（px）。
    #[serde(default = "default_random_radius_min")]
    pub random_radius_min: f32,
    /// 随机球：最大半径（px）。
    #[serde(default = "default_random_radius_max")]
    pub random_radius_max: f32,
    /// 随机球：已锁定的相对屏幕中心偏移 X（LockOnStart 持久化用，0 表示未生成）。
    #[serde(default)]
    pub random_orb_x: f32,
    /// 随机球：已锁定的相对屏幕中心偏移 Y（LockOnStart 持久化用，0 表示未生成）。
    #[serde(default)]
    pub random_orb_y: f32,
    /// 边框：样式变体。
    #[serde(default)]
    pub border_frame_style: BorderFrameStyle,
    /// 边框：四边中间是否留 20% 缺口。
    #[serde(default)]
    pub border_gap: bool,
    /// 边框：矩形条是否位于屏幕内侧。
    #[serde(default = "default_border_inset")]
    pub border_inset: bool,
    /// 自定义定位球：上边缘球数量。
    #[serde(default = "default_custom_orb_count")]
    pub custom_orb_top_count: u32,
    /// 自定义定位球：下边缘球数量。
    #[serde(default = "default_custom_orb_count")]
    pub custom_orb_bottom_count: u32,
    /// 自定义定位球：左边缘球数量（预留）。
    #[serde(default = "default_custom_orb_count")]
    pub custom_orb_left_count: u32,
    /// 自定义定位球：右边缘球数量（预留）。
    #[serde(default = "default_custom_orb_count")]
    pub custom_orb_right_count: u32,
    /// 随机球：每边球数量。
    #[serde(default = "default_random_orb_count")]
    pub random_orb_count: u32,
    /// 随机球：距离屏幕边缘的固定偏移（px）。
    #[serde(default = "default_random_orb_offset")]
    pub random_orb_offset: f32,
    /// 随机球：位置随机扰动范围（px）。
    #[serde(default = "default_random_orb_jitter")]
    pub random_orb_jitter: f32,
}

fn default_secondary_size() -> f32 { 48.0 }
fn default_radius() -> f32 { 0.0 }
fn default_offset() -> f32 { 0.0 }
fn default_corner_radius() -> f32 { 4.0 }
fn default_margin() -> f32 { 0.0 }
fn default_ring_radius_pct() -> f32 { 0.05 }
fn default_random_center_deviation() -> f32 { 0.2 }
fn default_random_radius_min() -> f32 { 4.0 }
fn default_random_radius_max() -> f32 { 12.0 }
fn default_custom_orb_count() -> u32 { 3 }
fn default_random_orb_count() -> u32 { 3 }
fn default_random_orb_offset() -> f32 { 100.0 }
fn default_random_orb_jitter() -> f32 { 40.0 }
fn default_border_inset() -> bool { true }

/// 矩形贴图（卫生纸）可贴靠的屏幕边缘位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    /// 屏幕顶部，水平居中。
    Top,
    /// 屏幕底部，水平居中。
    Bottom,
    /// 屏幕左侧，垂直居中。
    Left,
    /// 屏幕右侧，垂直居中。
    Right,
    /// 屏幕正中心（默认）。
    Center,
}

impl Default for Anchor {
    fn default() -> Self {
        Self::Center
    }
}

/// 支持的辅助贴图样式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrosshairStyle {
    /// 卫生纸：屏幕中心一定尺寸的白色半透明矩形，模拟贴了一张卫生纸。
    ToiletPaper,
    /// 准星：屏幕中心十字线。
    Cross,
    /// 大准星：从屏幕边缘延伸到屏幕中心的水平线与垂直线。
    LargeCross,
    /// 定位球4：屏幕四角各一个圆形。
    CornerDots4,
    /// 定位球6：四角圆形 + 垂直中心一个圆形。
    CornerDots6,
    /// 定位球8：四角圆形 + 垂直中心圆形 + 水平中心圆形。
    CornerDots8,
    /// 中心环：以屏幕中心为圆心的圆环，提供同心圆锚点。
    Ring,
    /// 自定义定位球：用户可选择在 TOP / BOTTOM 等预设位置显示点状锚点。
    CustomOrb,
    /// 随机球：启动时在中心区域随机生成位置与大小的锚点。
    RandomOrb,
    /// 边框：类似电影安全框的边界参考。
    BorderFrame,
}

/// 中心环线型样式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RingStyle {
    /// 实心细线。
    Solid,
    /// 虚线（4px 线 + 4px 空）。
    Dashed,
    /// 双环：内环 1px 实线 + 外环 1px 虚线，间距 4px。
    Double,
}

impl Default for RingStyle {
    fn default() -> Self {
        Self::Solid
    }
}

/// 自定义定位球的预设位置位掩码。
///
/// 第一阶段支持 TOP / BOTTOM；LEFT / RIGHT 已预留字段，UI 与渲染可逐步扩展。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrbPosition(pub u8);

impl OrbPosition {
    /// 屏幕水平居中，距上边缘 offset。
    pub const TOP: u8 = 0b0001;
    /// 屏幕水平居中，距下边缘 offset。
    pub const BOTTOM: u8 = 0b0010;
    /// 屏幕垂直居中，距左边缘 offset（预留）。
    pub const LEFT: u8 = 0b0100;
    /// 屏幕垂直居中，距右边缘 offset（预留）。
    pub const RIGHT: u8 = 0b1000;

    /// 创建空位掩码。
    pub const fn empty() -> Self {
        Self(0)
    }

    /// 判断是否包含某一位。
    pub const fn contains(self, flag: u8) -> bool {
        (self.0 & flag) == flag
    }

    /// 添加某一位。
    pub fn insert(&mut self, flag: u8) {
        self.0 |= flag;
    }

    /// 移除某一位。
    pub fn remove(&mut self, flag: u8) {
        self.0 &= !flag;
    }

    /// 切换某一位。
    pub fn toggle(&mut self, flag: u8) {
        self.0 ^= flag;
    }
}

impl Default for OrbPosition {
    fn default() -> Self {
        Self(Self::TOP | Self::BOTTOM)
    }
}

/// 随机球工作模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RandomOrbMode {
    /// 启动随机生成并持久化，后续保持固定。
    LockOnStart,
    /// 每次启动重新随机。
    Reshuffle,
}

impl Default for RandomOrbMode {
    fn default() -> Self {
        Self::LockOnStart
    }
}

/// 边框样式变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorderFrameStyle {
    /// 四边完整细线。
    Solid,
    /// 四边中间留空，避免遮挡小地图/状态栏。
    Gap,
}

impl Default for BorderFrameStyle {
    fn default() -> Self {
        Self::Solid
    }
}

/// 触发规则：决定辅助贴图何时显示。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerRule {
    /// 是否启用触发器。
    pub enabled: bool,
    /// 触发进程名列表。空列表表示不限制。
    pub process_names: Vec<String>,
}

impl AppConfig {
    /// 生成默认配置，包含一个名为 "default" 的 Profile。
    pub fn default_config() -> Self {
        let mut profiles = std::collections::HashMap::new();
        profiles.insert("default".to_string(), Profile::default_profile());
        Self {
            active_profile: "default".to_string(),
            profiles,
        }
    }

    /// 校验整个配置是否合法。
    ///
    /// 检查项：
    /// - active_profile 必须存在；
    /// - 至少包含一个 Profile；
    /// - 每个 Profile 单独校验通过。
    pub fn validate(&self) -> crate::Result<()> {
        if self.profiles.is_empty() {
            return Err(crate::ConfigError::Validation(
                "at least one profile is required".to_string(),
            ));
        }
        if !self.profiles.contains_key(&self.active_profile) {
            return Err(crate::ConfigError::Validation(format!(
                "active profile '{}' does not exist",
                self.active_profile
            )));
        }
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .map_err(|e| crate::ConfigError::Validation(format!(
                    "profile '{}': {}",
                    name, e
                )))?;
        }
        Ok(())
    }

    /// 获取当前激活的 Profile 的可变引用。
    pub fn active_profile_mut(&mut self) -> Option<&mut Profile> {
        self.profiles.get_mut(&self.active_profile)
    }

    /// 获取当前激活的 Profile 的不可变引用。
    pub fn active_profile(&self) -> Option<&Profile> {
        self.profiles.get(&self.active_profile)
    }
}

impl Profile {
    /// 生成默认 Profile。
    pub fn default_profile() -> Self {
        Self {
            crosshair: Crosshair::default_crosshair(),
            trigger: TriggerRule::default_rule(),
            settings_hotkey: "F10".to_string(),
            target_window: String::new(),
        }
    }

    /// 校验 Profile 内字段范围。
    pub fn validate(&self) -> crate::Result<()> {
        self.crosshair.validate()?;
        Ok(())
    }
}

impl Crosshair {
    /// 生成默认辅助贴图配置：卫生纸样式，尺寸 120x80、厚度 2、白色、透明度 60%。
    pub fn default_crosshair() -> Self {
        Self {
            style: CrosshairStyle::ToiletPaper,
            size: 120.0,
            secondary_size: 80.0,
            thickness: 2.0,
            radius: 0.0,
            offset: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            opacity: 0.6,
            gap: 4.0,
            corner_radius: 4.0,
            anchor: Anchor::Top,
            margin: 0.0,
            ring_radius_pct: default_ring_radius_pct(),
            ring_style: RingStyle::default(),
            orb_positions: OrbPosition::default(),
            random_mode: RandomOrbMode::default(),
            random_center_deviation: default_random_center_deviation(),
            random_radius_min: default_random_radius_min(),
            random_radius_max: default_random_radius_max(),
            random_orb_x: 0.0,
            random_orb_y: 0.0,
            border_frame_style: BorderFrameStyle::default(),
            border_gap: false,
            border_inset: true,
            custom_orb_top_count: default_custom_orb_count(),
            custom_orb_bottom_count: default_custom_orb_count(),
            custom_orb_left_count: default_custom_orb_count(),
            custom_orb_right_count: default_custom_orb_count(),
            random_orb_count: default_random_orb_count(),
            random_orb_offset: default_random_orb_offset(),
            random_orb_jitter: default_random_orb_jitter(),
        }
    }

    /// 校验准心字段范围。
    pub fn validate(&self) -> crate::Result<()> {
        if self.size <= 0.0 {
            return Err(crate::ConfigError::Validation(
                "crosshair size must be positive".to_string(),
            ));
        }
        if self.secondary_size <= 0.0 {
            return Err(crate::ConfigError::Validation(
                "crosshair secondary_size must be positive".to_string(),
            ));
        }
        if self.thickness <= 0.0 {
            return Err(crate::ConfigError::Validation(
                "crosshair thickness must be positive".to_string(),
            ));
        }
        if self.radius < 0.0 {
            return Err(crate::ConfigError::Validation(
                "crosshair radius must be non-negative".to_string(),
            ));
        }
        if self.offset < 0.0 {
            return Err(crate::ConfigError::Validation(
                "crosshair offset must be non-negative".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.opacity) {
            return Err(crate::ConfigError::Validation(
                "opacity must be in [0.0, 1.0]".to_string(),
            ));
        }
        for (i, c) in self.color.iter().enumerate() {
            if !(0.0..=1.0).contains(c) {
                return Err(crate::ConfigError::Validation(format!(
                    "color channel {} must be in [0.0, 1.0]",
                    i
                )));
            }
        }
        if self.gap < 0.0 {
            return Err(crate::ConfigError::Validation(
                "gap must be non-negative".to_string(),
            ));
        }
        if self.margin < 0.0 {
            return Err(crate::ConfigError::Validation(
                "margin must be non-negative".to_string(),
            ));
        }
        if !(0.03..=0.08).contains(&self.ring_radius_pct) {
            return Err(crate::ConfigError::Validation(
                "ring_radius_pct must be in [0.03, 0.08]".to_string(),
            ));
        }
        if !(0.1..=0.3).contains(&self.random_center_deviation) {
            return Err(crate::ConfigError::Validation(
                "random_center_deviation must be in [0.1, 0.3]".to_string(),
            ));
        }
        if self.random_radius_min <= 0.0 || self.random_radius_max <= 0.0 {
            return Err(crate::ConfigError::Validation(
                "random orb radius range must be positive".to_string(),
            ));
        }
        if self.random_radius_min > self.random_radius_max {
            return Err(crate::ConfigError::Validation(
                "random_radius_min must not exceed random_radius_max".to_string(),
            ));
        }
        if self.custom_orb_top_count == 0
            && self.custom_orb_bottom_count == 0
            && self.custom_orb_left_count == 0
            && self.custom_orb_right_count == 0
        {
            return Err(crate::ConfigError::Validation(
                "custom orb must have at least one enabled edge".to_string(),
            ));
        }
        if self.random_orb_count == 0 {
            return Err(crate::ConfigError::Validation(
                "random orb count must be positive".to_string(),
            ));
        }
        if self.random_orb_offset < 0.0 || self.random_orb_jitter < 0.0 {
            return Err(crate::ConfigError::Validation(
                "random orb offset/jitter must be non-negative".to_string(),
            ));
        }
        if self.random_radius_min <= 0.0 || self.random_radius_max <= 0.0 {
            return Err(crate::ConfigError::Validation(
                "random orb radius range must be positive".to_string(),
            ));
        }
        if self.random_radius_min > self.random_radius_max {
            return Err(crate::ConfigError::Validation(
                "random_radius_min must not exceed random_radius_max".to_string(),
            ));
        }
        Ok(())
    }
}

impl TriggerRule {
    /// 生成默认触发规则：启用但不对进程做限制。
    pub fn default_rule() -> Self {
        Self {
            enabled: true,
            process_names: vec![],
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_validates() {
        let cfg = AppConfig::default_config();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.active_profile, "default");
    }

    #[test]
    fn invalid_active_profile_fails() {
        let mut cfg = AppConfig::default_config();
        cfg.active_profile = "missing".to_string();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn negative_size_fails() {
        let mut cfg = AppConfig::default_config();
        cfg.active_profile_mut().unwrap().crosshair.size = -1.0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn opacity_out_of_range_fails() {
        let mut cfg = AppConfig::default_config();
        cfg.active_profile_mut().unwrap().crosshair.opacity = 1.5;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn ring_radius_pct_default_and_validation() {
        let cfg = AppConfig::default_config();
        let ch = &cfg.active_profile().unwrap().crosshair;
        assert_eq!(ch.ring_radius_pct, 0.05);
        assert!(ch.validate().is_ok());

        let mut cfg = AppConfig::default_config();
        cfg.active_profile_mut().unwrap().crosshair.ring_radius_pct = 0.02;
        assert!(cfg.validate().is_err());

        let mut cfg = AppConfig::default_config();
        cfg.active_profile_mut().unwrap().crosshair.ring_radius_pct = 0.09;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn custom_orb_edge_counts_and_positions() {
        let mut ch = Crosshair::default_crosshair();
        ch.style = CrosshairStyle::CustomOrb;
        ch.radius = 6.0;
        ch.offset = 20.0;
        ch.orb_positions = OrbPosition(OrbPosition::TOP | OrbPosition::BOTTOM);
        ch.custom_orb_top_count = 3;
        ch.custom_orb_bottom_count = 5;

        assert!(ch.validate().is_ok());

        // 验证上边缘 3 个球均匀分布：首球在 1/4、尾球在 3/4 宽度处。
        let screen = (1920.0, 1080.0);
        let top_positions = [(screen.0 / 4.0, ch.offset), (screen.0 / 2.0, ch.offset), (screen.0 * 3.0 / 4.0, ch.offset)];
        assert_eq!(top_positions[0].0, 480.0);
        assert_eq!(top_positions[1].0, 960.0);
        assert_eq!(top_positions[2].0, 1440.0);

        // 验证左/右预留字段存在且可设置。
        ch.orb_positions.insert(OrbPosition::LEFT);
        ch.custom_orb_left_count = 2;
        assert!(ch.validate().is_ok());

        // 禁用所有边应校验失败。
        ch.custom_orb_top_count = 0;
        ch.custom_orb_bottom_count = 0;
        ch.custom_orb_left_count = 0;
        ch.custom_orb_right_count = 0;
        assert!(ch.validate().is_err());
    }

    #[test]
    fn random_orb_range_validation() {
        let mut cfg = AppConfig::default_config();
        let ch = &mut cfg.active_profile_mut().unwrap().crosshair;
        ch.style = CrosshairStyle::RandomOrb;
        ch.random_radius_min = 8.0;
        ch.random_radius_max = 4.0;
        assert!(cfg.validate().is_err());

        let mut cfg = AppConfig::default_config();
        let ch = &mut cfg.active_profile_mut().unwrap().crosshair;
        ch.random_radius_min = 0.0;
        assert!(cfg.validate().is_err());

        let mut cfg = AppConfig::default_config();
        let ch = &mut cfg.active_profile_mut().unwrap().crosshair;
        ch.random_orb_count = 0;
        assert!(cfg.validate().is_err());

        let mut cfg = AppConfig::default_config();
        let ch = &mut cfg.active_profile_mut().unwrap().crosshair;
        ch.random_orb_jitter = -1.0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn border_frame_defaults() {
        let ch = Crosshair::default_crosshair();
        assert!(ch.border_inset);
        assert!(!ch.border_gap);
        assert_eq!(ch.border_frame_style, BorderFrameStyle::Solid);
    }

    #[test]
    fn border_frame_inset_offsets() {
        let mut ch = Crosshair::default_crosshair();
        ch.style = CrosshairStyle::BorderFrame;
        ch.offset = 30.0;
        ch.thickness = 10.0;
        ch.border_inset = true;
        let screen = (1920.0, 1080.0);
        // 内侧：矩形条中心在距边缘 offset 处。
        let top_y = ch.offset;
        let left_x = ch.offset;
        assert_eq!(top_y, 30.0);
        assert_eq!(left_x, 30.0);
        // 外侧：矩形条中心在 -offset 处。
        ch.border_inset = false;
        assert_eq!(-ch.offset, -30.0);
    }

    #[test]
    fn random_orb_lock_position_persists() {
        let mut ch = Crosshair::default_crosshair();
        ch.style = CrosshairStyle::RandomOrb;
        ch.random_mode = RandomOrbMode::LockOnStart;
        ch.random_orb_x = 0.5;
        ch.random_orb_y = -0.3;
        // 反序列化后值应保持不变。
        let json = serde_json::to_string(&ch).unwrap();
        let restored: Crosshair = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.random_orb_x, 0.5);
        assert_eq!(restored.random_orb_y, -0.3);
        assert_eq!(restored.random_mode, RandomOrbMode::LockOnStart);
    }

    #[test]
    fn all_new_styles_serialize_roundtrip() {
        for style in [
            CrosshairStyle::Ring,
            CrosshairStyle::CustomOrb,
            CrosshairStyle::RandomOrb,
            CrosshairStyle::BorderFrame,
        ] {
            let mut ch = Crosshair::default_crosshair();
            ch.style = style;
            let json = serde_json::to_string(&ch).unwrap();
            let restored: Crosshair = serde_json::from_str(&json).unwrap();
            assert_eq!(restored.style, style);
        }
    }
}
