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
    /// 应用级 UI 设置（非 Profile 绑定的全局偏好）。
    #[serde(default)]
    pub settings: AppSettings,
}

/// 应用级 UI 设置（全局偏好，不随 Profile 切换）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    /// 开始覆盖时是否自动隐藏配置窗口并切换到目标窗口。
    /// - `"yes"`: 自动隐藏并切换
    /// - `"no"`: 保持配置窗口显示
    /// - `"ask"`: 每次询问
    #[serde(default = "default_auto_switch_on_overlay")]
    pub auto_switch_on_overlay: String,
    /// UI 语言（`"zh-CN"` / `"en"`），`"auto"` 表示跟随系统语言。
    #[serde(default = "default_locale")]
    pub locale: String,
    /// Overlay 覆盖模式：true=全屏覆盖，false=仅跟随目标窗口区域。默认 true。
    #[serde(default = "default_fullscreen_overlay")]
    pub fullscreen_overlay: bool,
    /// 拖拽窗口时是否实时显示准心（仅窗口模式生效）。
    /// 默认 false：拖拽期间隐藏准心，停止拖拽 1200ms 后恢复显示。
    #[serde(default)]
    pub live_drag_preview: bool,
    /// 是否启用 WebView2 GPU 硬件加速。
    /// 默认 false：关闭 GPU 加速以降低内存占用（GPU 进程 ~80MB → ~15MB）。
    #[serde(default)]
    pub gpu_acceleration: bool,
    /// 自动更新通道：`"stable"`（正式版，奇数版本号）或 `"prerelease"`（尝鲜版，偶数/预发布）。
    /// 默认 `"stable"`。
    #[serde(default = "default_update_channel")]
    pub update_channel: String,
    /// 是否使用中国大陆加速镜像（gh-proxy）访问 GitHub Release。
    /// 默认 `false`；简体中文用户首次启动时由前端自动设为 `true`。
    #[serde(default)]
    pub cn_mirror: bool,
    /// 加速镜像站地址，默认 `"https://v4.gh-proxy.org"`。
    /// 用户可在设置中自定义。
    #[serde(default = "default_mirror_url")]
    pub mirror_url: String,
    /// 是否启用覆盖层抗锯齿。
    /// 默认 true：开启后圆形、圆环、三角形等曲线边缘更平滑；
    /// 关闭可略微降低 CPU 开销（低性能设备适用）。
    #[serde(default = "default_antialiasing")]
    pub antialiasing: bool,
    /// 覆盖层渲染后端。
    /// - `"cpu"`：手写 CPU 像素光栅化（默认，零额外依赖）
    /// - `"svg"`：将图元转为 SVG 由 resvg/tiny-skia 光栅化（抗锯齿质量更高）
    #[serde(default = "default_renderer_backend")]
    pub renderer_backend: RendererBackend,
    /// 快捷颜色预设（5 色，默认白绿蓝红橙）。
    /// 配置页点击色块可一键切换准心颜色，设置页可自定义。
    #[serde(default = "default_quick_colors")]
    pub quick_colors: [[f32; 4]; 5],
    /// 快捷键绑定（action → 快捷键字符串）。
    /// Vec<(action, key)> 保证序列化稳定，运行时可转 HashMap 查找。
    #[serde(default = "default_hotkey_bindings")]
    pub hotkey_bindings: Vec<(HotkeyAction, String)>,
}

fn default_auto_switch_on_overlay() -> String {
    "ask".to_string()
}

fn default_locale() -> String {
    "auto".to_string()
}

fn default_fullscreen_overlay() -> bool {
    true
}

fn default_update_channel() -> String {
    "stable".to_string()
}

fn default_mirror_url() -> String {
    "https://v4.gh-proxy.org".to_string()
}

/// 抗锯齿默认开启。
fn default_antialiasing() -> bool {
    true
}

/// 渲染后端默认为 CPU。
fn default_renderer_backend() -> RendererBackend {
    RendererBackend::Cpu
}

/// 默认快捷颜色预设：白、绿、蓝、红、橙。
fn default_quick_colors() -> [[f32; 4]; 5] {
    [
        [1.0, 1.0, 1.0, 1.0], // 白
        [0.0, 1.0, 0.0, 1.0], // 绿
        [0.2, 0.5, 1.0, 1.0], // 蓝
        [1.0, 0.0, 0.0, 1.0], // 红
        [1.0, 0.5, 0.0, 1.0], // 橙
    ]
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_switch_on_overlay: default_auto_switch_on_overlay(),
            locale: default_locale(),
            fullscreen_overlay: default_fullscreen_overlay(),
            live_drag_preview: false,
            gpu_acceleration: false,
            update_channel: default_update_channel(),
            cn_mirror: false,
            mirror_url: default_mirror_url(),
            antialiasing: default_antialiasing(),
            renderer_backend: default_renderer_backend(),
            quick_colors: default_quick_colors(),
            hotkey_bindings: default_hotkey_bindings(),
        }
    }
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
    /// 贴边矩形用做宽度；准星用做十字臂长；定位球用于限制半径/偏移上限。
    pub size: f32,
    /// 贴图次尺寸，单位像素。
    /// 贴边矩形用做高度；其他样式可忽略。
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
    /// 矩形贴图圆角半径，单位像素（贴边矩形用到）。
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f32,
    /// 矩形贴图（贴边矩形）的贴边位置。
    #[serde(default)]
    pub anchor: Anchor,
    /// 矩形贴图（贴边矩形）与贴边外侧的边距，单位像素。
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
    /// 自定义图片：PNG 文件路径（空字符串表示未选择）。
    #[serde(default)]
    pub image_path: String,
    /// 自定义图片：缩放比例（1.0 = 原始大小）。
    #[serde(default = "default_image_scale")]
    pub image_scale: f32,
    /// 自定义图片：相对屏幕中心的水平偏移（px，逻辑坐标）。
    #[serde(default)]
    pub image_offset_x: f32,
    /// 自定义图片：相对屏幕中心的垂直偏移（px，逻辑坐标）。
    #[serde(default)]
    pub image_offset_y: f32,
    /// 箭头：距屏幕边缘的像素距离（0 = 贴边）。
    #[serde(default = "default_arrow_distance")]
    pub arrow_distance: f32,
    /// 箭头：尾巴宽度（px）。默认 0 表示等于箭头大小（size）。
    #[serde(default)]
    pub arrow_width: f32,
    /// 箭头：是否为每边单独设置尾巴长度。
    #[serde(default)]
    pub arrow_tail_per_edge: bool,
    /// 箭头：上边尾巴长度（仅 arrow_tail_per_edge 为 true 时生效）。
    #[serde(default)]
    pub arrow_tail_top: f32,
    /// 箭头：下边尾巴长度。
    #[serde(default)]
    pub arrow_tail_bottom: f32,
    /// 箭头：左边尾巴长度。
    #[serde(default)]
    pub arrow_tail_left: f32,
    /// 箭头：右边尾巴长度。
    #[serde(default)]
    pub arrow_tail_right: f32,
    /// 网格：单格宽度（像素），横竖格数根据屏幕宽高 / grid_size 自动计算。默认 80。
    #[serde(default = "default_grid_size")]
    pub grid_size: f32,
    /// 网格：对齐方式（居中 / 贴边），默认居中。
    #[serde(default)]
    pub grid_alignment: GridAlignment,
}

fn default_secondary_size() -> f32 {
    48.0
}
fn default_radius() -> f32 {
    0.0
}
fn default_offset() -> f32 {
    0.0
}
fn default_corner_radius() -> f32 {
    4.0
}
fn default_margin() -> f32 {
    0.0
}
fn default_ring_radius_pct() -> f32 {
    0.05
}
fn default_random_center_deviation() -> f32 {
    0.2
}
fn default_random_radius_min() -> f32 {
    4.0
}
fn default_random_radius_max() -> f32 {
    12.0
}
fn default_custom_orb_count() -> u32 {
    3
}
fn default_random_orb_count() -> u32 {
    3
}
fn default_random_orb_offset() -> f32 {
    100.0
}
fn default_random_orb_jitter() -> f32 {
    40.0
}
fn default_image_scale() -> f32 {
    1.0
}
fn default_arrow_distance() -> f32 {
    0.0
}
fn default_grid_size() -> f32 {
    80.0
}
fn default_border_inset() -> bool {
    true
}

/// 矩形贴图（贴边矩形）可贴靠的屏幕边缘位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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
    #[default]
    Center,
}

/// 覆盖层渲染后端。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RendererBackend {
    /// 手写 CPU 像素光栅化（默认，零额外依赖）。
    #[default]
    Cpu,
    /// 将图元转为 SVG 字符串，由 resvg/tiny-skia 光栅化（抗锯齿质量更高）。
    Svg,
}

/// 支持的辅助贴图样式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrosshairStyle {
    /// 贴边矩形：可贴靠屏幕四边或居中的半透明矩形，作为固定视觉锚点。
    #[serde(alias = "toilet_paper")]
    EdgeRect,
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
    /// 自定义图片：加载 PNG 文件作为准心图案。
    CustomImage,
    /// 箭头：屏幕四边中点各一个指向中心的三角形箭头。
    EdgeArrows,
    /// 网格：全屏棋盘式格子（类似围棋棋盘），用格子数与线宽控制。
    Grid,
}

/// 中心环线型样式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RingStyle {
    /// 实心细线。
    #[default]
    Solid,
    /// 虚线（4px 线 + 4px 空）。
    Dashed,
    /// 双环：内环 1px 实线 + 外环 1px 虚线，间距 4px。
    Double,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RandomOrbMode {
    /// 启动随机生成并持久化，后续保持固定。
    #[default]
    LockOnStart,
    /// 每次启动重新随机。
    Reshuffle,
}

/// 边框样式变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorderFrameStyle {
    /// 四边完整细线。
    #[default]
    Solid,
    /// 四边中间留空，避免遮挡小地图/状态栏。
    Gap,
}

/// 网格对齐方式：决定线条在屏幕内的定位策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridAlignment {
    /// 居中分布（默认）：线条均匀分布在屏幕中间区域，不贴边。
    #[default]
    Center,
    /// 贴边分布：含屏幕外边缘线，格子完整覆盖整个屏幕。
    Edge,
}

/// 快捷键可触发的动作类型，新增功能时只需在此枚举追加变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyAction {
    /// 切换覆盖层显示。
    ToggleOverlay,
    /// 开启覆盖层。
    StartOverlay,
    /// 关闭覆盖层。
    StopOverlay,
    /// 切换到下一个颜色预设。
    CycleColorNext,
    /// 切换到上一个颜色预设。
    CycleColorPrev,
    /// 设置颜色 1。
    #[serde(rename = "set_color_1")]
    SetColor1,
    /// 设置颜色 2。
    #[serde(rename = "set_color_2")]
    SetColor2,
    /// 设置颜色 3。
    #[serde(rename = "set_color_3")]
    SetColor3,
    /// 设置颜色 4。
    #[serde(rename = "set_color_4")]
    SetColor4,
    /// 设置颜色 5。
    #[serde(rename = "set_color_5")]
    SetColor5,
}

/// 快捷键绑定默认值：仅绑定 ToggleOverlay。
fn default_hotkey_bindings() -> Vec<(HotkeyAction, String)> {
    vec![(HotkeyAction::ToggleOverlay, "Ctrl+Alt+O".to_string())]
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
            settings: AppSettings::default(),
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
            profile.validate().map_err(|e| {
                crate::ConfigError::Validation(format!("profile '{}': {}", name, e))
            })?;
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
    /// 生成默认辅助贴图配置：贴边矩形样式，白色、透明度 60%。
    pub fn default_crosshair() -> Self {
        Self::default_for_style(CrosshairStyle::EdgeRect)
    }

    /// 为指定样式生成一套开箱即用的默认参数。
    ///
    /// 切换样式时，前端/后端可调用此函数重置参数，避免旧样式的尺寸/偏移在新样式下不可用。
    pub fn default_for_style(style: CrosshairStyle) -> Self {
        let mut base = Self {
            style,
            size: 16.0,
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
            ring_radius_pct: 0.05,
            ring_style: RingStyle::default(),
            orb_positions: OrbPosition::default(),
            random_mode: RandomOrbMode::default(),
            random_center_deviation: 0.2,
            random_radius_min: 4.0,
            random_radius_max: 12.0,
            random_orb_x: 0.0,
            random_orb_y: 0.0,
            border_frame_style: BorderFrameStyle::default(),
            border_inset: true,
            custom_orb_top_count: 3,
            custom_orb_bottom_count: 3,
            custom_orb_left_count: 3,
            custom_orb_right_count: 3,
            random_orb_count: 3,
            random_orb_offset: 100.0,
            random_orb_jitter: 40.0,
            image_path: String::new(),
            image_scale: 1.0,
            image_offset_x: 0.0,
            image_offset_y: 0.0,
            arrow_distance: 0.0,
            arrow_width: 0.0,
            arrow_tail_per_edge: false,
            arrow_tail_top: 0.0,
            arrow_tail_bottom: 0.0,
            arrow_tail_left: 0.0,
            arrow_tail_right: 0.0,
            grid_size: 80.0,
            grid_alignment: GridAlignment::default(),
        };

        // 按样式设置开箱即用的推荐参数。
        match style {
            CrosshairStyle::EdgeRect => {
                base.size = 180.0;
                base.secondary_size = 24.0;
                base.thickness = 4.0;
                base.anchor = Anchor::Top;
                base.margin = 16.0;
                base.corner_radius = 12.0;
            }
            CrosshairStyle::Cross => {
                base.size = 24.0;
                base.thickness = 2.0;
                base.gap = 4.0;
                base.opacity = 0.8;
            }
            CrosshairStyle::LargeCross => {
                base.thickness = 2.0;
                base.opacity = 0.5;
            }
            CrosshairStyle::CornerDots4
            | CrosshairStyle::CornerDots6
            | CrosshairStyle::CornerDots8 => {
                base.offset = 40.0;
                base.thickness = 3.0;
                base.radius = 0.0;
                base.opacity = 0.7;
            }
            CrosshairStyle::Ring => {
                base.thickness = 2.0;
                base.ring_radius_pct = 0.06;
                base.ring_style = RingStyle::Solid;
                base.opacity = 0.8;
            }
            CrosshairStyle::CustomOrb => {
                base.radius = 6.0;
                base.offset = 30.0;
                base.orb_positions = OrbPosition(OrbPosition::TOP | OrbPosition::BOTTOM);
                base.custom_orb_top_count = 3;
                base.custom_orb_bottom_count = 3;
                base.custom_orb_left_count = 3;
                base.custom_orb_right_count = 3;
                base.opacity = 0.7;
            }
            CrosshairStyle::RandomOrb => {
                base.random_orb_count = 3;
                base.random_orb_offset = 80.0;
                base.random_orb_jitter = 30.0;
                base.random_radius_min = 4.0;
                base.random_radius_max = 10.0;
                base.opacity = 0.6;
            }
            CrosshairStyle::BorderFrame => {
                base.thickness = 6.0;
                base.offset = 24.0;
                base.border_frame_style = BorderFrameStyle::Solid;
                base.border_inset = false;
                base.opacity = 0.5;
            }
            CrosshairStyle::EdgeArrows => {
                base.size = 16.0;
                base.arrow_distance = 60.0;
                base.arrow_width = 0.0;
                base.arrow_tail_per_edge = false;
                base.orb_positions = OrbPosition(0);
                base.opacity = 0.7;
            }
            CrosshairStyle::Grid => {
                base.grid_size = 120.0;
                base.thickness = 2.0;
                base.grid_alignment = GridAlignment::Center;
                base.opacity = 0.35;
            }
            // 自定义图片需要用户选择路径，保留最小化默认值。
            CrosshairStyle::CustomImage => {
                base.size = 64.0;
                base.image_scale = 1.0;
                base.image_offset_x = 0.0;
                base.image_offset_y = 0.0;
                base.opacity = 0.9;
            }
        }

        base
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
        if self.image_scale <= 0.0 {
            return Err(crate::ConfigError::Validation(
                "image_scale must be positive".to_string(),
            ));
        }
        // CustomImage 样式要求 image_path 非空（仅在样式为 CustomImage 时检查）。
        if matches!(self.style, CrosshairStyle::CustomImage) && self.image_path.trim().is_empty() {
            return Err(crate::ConfigError::Validation(
                "image_path must not be empty when style is CustomImage".to_string(),
            ));
        }
        if self.grid_size < 10.0 || self.grid_size > 500.0 {
            return Err(crate::ConfigError::Validation(
                "grid_size must be in [10, 500]".to_string(),
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

// ===== 四层可定制化架构：元素 / 物料 / 图层 =====
//
// 以下类型定义了"元素 → 物料 → 图层 → 配置"四层架构的数据模型。
// 物料是 Rhai 脚本定义的"参数 → Element 列表"映射，通过 peregrine_material crate 求值。
// 一个 Profile 可包含多个图层，每个图层引用一个物料实例并携带参数、变换、样式。

/// 逻辑坐标矩形区域，作为物料求值的输入。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// 左上角 X 坐标（逻辑像素）。
    pub min_x: f32,
    /// 左上角 Y 坐标。
    pub min_y: f32,
    /// 右下角 X 坐标。
    pub max_x: f32,
    /// 右下角 Y 坐标。
    pub max_y: f32,
}

impl Rect {
    /// 矩形宽度。
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    /// 矩形高度。
    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    /// 中心 X 坐标。
    pub fn center_x(&self) -> f32 {
        (self.min_x + self.max_x) / 2.0
    }

    /// 中心 Y 坐标。
    pub fn center_y(&self) -> f32 {
        (self.min_y + self.max_y) / 2.0
    }
}

/// 基础图元（Element）：不可再分的渲染原语。
///
/// 物料脚本的输出由若干 Element 组成，渲染器（overlay_renderer）与前端 Canvas 预览
/// 共同消费同一份 Element 列表，确保 WYSIWYG。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Element {
    /// 填充矩形（逻辑坐标）。
    Rect {
        /// 左上角 X。
        x: f32,
        /// 左上角 Y。
        y: f32,
        /// 宽度。
        w: f32,
        /// 高度。
        h: f32,
    },
    /// 填充圆。
    Circle {
        /// 圆心 X。
        cx: f32,
        /// 圆心 Y。
        cy: f32,
        /// 半径。
        radius: f32,
    },
    /// 圆环描边。
    CircleStroke {
        cx: f32,
        cy: f32,
        radius: f32,
        /// 描边厚度。
        thickness: f32,
    },
    /// 虚线圆环。
    DashedCircle {
        cx: f32,
        cy: f32,
        radius: f32,
        thickness: f32,
        /// 单段虚线长度。
        dash_len: f32,
        /// 虚线间隔长度。
        gap_len: f32,
    },
    /// 填充三角形（3 个顶点）。
    Triangle {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    },
    /// 填充多边形（顶点数组，按顺序连接）。
    Polygon {
        /// 顶点列表，每项 `[x, y]`。
        points: Vec<[f32; 2]>,
    },
    /// 粗线段。
    Line {
        /// 起点 X。
        x1: f32,
        /// 起点 Y。
        y1: f32,
        /// 终点 X。
        x2: f32,
        /// 终点 Y。
        y2: f32,
        /// 线条厚度。
        thickness: f32,
    },
    /// 文本。
    Text {
        /// 左下角基线 X 坐标。
        x: f32,
        /// 基线 Y 坐标。
        y: f32,
        /// 文本内容。
        content: String,
        /// 字号（逻辑像素）。
        font_size: f32,
    },
    /// 图片。
    Image {
        /// 图片文件绝对路径。
        path: String,
        /// 左上角 X。
        x: f32,
        /// 左上角 Y。
        y: f32,
        /// 显示宽度。
        w: f32,
        /// 显示高度。
        h: f32,
    },
}

/// 物料引用：图层所用的物料来源。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaterialRef {
    /// 内置物料：由二进制内嵌的 `.rhai` 提供，如 `builtin.cross`。
    Builtin {
        /// 物料 id（含 `builtin.` 前缀）。
        id: String,
    },
    /// 用户物料：位于 `%APPDATA%/Peregrine/materials/<name>.rhai`。
    User {
        /// 物料名称（不含扩展名，含 `user.` 前缀）。
        name: String,
    },
}

impl MaterialRef {
    /// 获取物料的完整查找 id。
    ///
    /// - `Builtin { id }` → 直接返回 `id`
    /// - `User { name }` → 返回 `name`（已含 `user.` 前缀）
    pub fn material_id(&self) -> &str {
        match self {
            MaterialRef::Builtin { id } => id,
            MaterialRef::User { name } => name,
        }
    }

    /// 是否为内置物料。
    pub fn is_builtin(&self) -> bool {
        matches!(self, MaterialRef::Builtin { .. })
    }
}

/// 图层几何变换：在物料输出 Element 后应用的平移 / 缩放 / 旋转。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    /// X 方向位移（逻辑像素，默认 0）。
    #[serde(default)]
    pub offset_x: f32,
    /// Y 方向位移。
    #[serde(default)]
    pub offset_y: f32,
    /// 均匀缩放因子（默认 1.0）。
    #[serde(default = "default_transform_scale")]
    pub scale: f32,
    /// 围绕屏幕中心的旋转角度（度，默认 0）。
    #[serde(default)]
    pub rotation_deg: f32,
}

fn default_transform_scale() -> f32 {
    1.0
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
            rotation_deg: 0.0,
        }
    }
}

/// 图层混合模式（首期仅支持 `Normal`，预留扩展点）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    /// 普通透明度混合（src over dst）。
    #[default]
    Normal,
}

/// 图层级样式：颜色 / 不透明度 / 混合模式。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerStyle {
    /// RGBA 颜色（0.0..=1.0）。
    #[serde(default = "default_layer_color")]
    pub color: [f32; 4],
    /// 图层整体不透明度（0.0..=1.0）。
    #[serde(default = "default_layer_opacity")]
    pub opacity: f32,
    /// 混合模式。
    #[serde(default)]
    pub blend_mode: BlendMode,
}

fn default_layer_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

fn default_layer_opacity() -> f32 {
    0.6
}

impl Default for LayerStyle {
    fn default() -> Self {
        Self {
            color: default_layer_color(),
            opacity: default_layer_opacity(),
            blend_mode: BlendMode::default(),
        }
    }
}

/// 单个图层：一个物料实例 + 参数 + 变换 + 样式。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    /// 图层内唯一标识（UUID v4 或简单序号字符串）。
    pub id: String,
    /// 用户可读的图层名。
    pub name: String,
    /// 引用的物料。
    pub material: MaterialRef,
    /// 该图层实例的具体参数（JSON 对象，覆盖物料 `defaults()`）。
    #[serde(default)]
    pub params: serde_json::Value,
    /// 图层级样式。
    #[serde(default)]
    pub style: LayerStyle,
    /// 几何变换。
    #[serde(default)]
    pub transform: Transform2D,
    /// 是否可见（false 时不参与渲染）。
    #[serde(default = "default_layer_visible")]
    pub visible: bool,
    /// 是否锁定（锁定后 UI 不可误改）。
    #[serde(default)]
    pub locked: bool,
}

fn default_layer_visible() -> bool {
    true
}

impl Layer {
    /// 创建一个引用指定物料的新图层，参数取物料 defaults（由调用方填充）。
    pub fn new(id: impl Into<String>, name: impl Into<String>, material: MaterialRef) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            material,
            params: serde_json::Value::Object(serde_json::Map::new()),
            style: LayerStyle::default(),
            transform: Transform2D::default(),
            visible: true,
            locked: false,
        }
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
        let ch = &cfg.active_profile().unwrap().crosshair;
        assert_eq!(ch.style, CrosshairStyle::EdgeRect);
        assert_eq!(ch.size, 180.0);
        assert_eq!(ch.secondary_size, 24.0);
        assert_eq!(ch.thickness, 4.0);
        assert_eq!(ch.margin, 16.0);
        assert_eq!(ch.corner_radius, 12.0);
    }

    #[test]
    fn default_for_all_styles_validates() {
        for style in [
            CrosshairStyle::EdgeRect,
            CrosshairStyle::Cross,
            CrosshairStyle::LargeCross,
            CrosshairStyle::CornerDots4,
            CrosshairStyle::CornerDots6,
            CrosshairStyle::CornerDots8,
            CrosshairStyle::Ring,
            CrosshairStyle::CustomOrb,
            CrosshairStyle::RandomOrb,
            CrosshairStyle::BorderFrame,
            CrosshairStyle::EdgeArrows,
            CrosshairStyle::Grid,
        ] {
            let ch = Crosshair::default_for_style(style);
            assert_eq!(ch.style, style);
            assert!(
                ch.validate().is_ok(),
                "style {:?} 的默认参数校验失败",
                style
            );
        }

        // CustomImage 默认无图片路径，需额外设置后才可校验。
        let mut ch = Crosshair::default_for_style(CrosshairStyle::CustomImage);
        assert_eq!(ch.style, CrosshairStyle::CustomImage);
        assert!(ch.validate().is_err());
        ch.image_path = "/tmp/crosshair.png".to_string();
        assert!(ch.validate().is_ok());
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
    fn default_ring_style_is_solid() {
        let ch = Crosshair::default_for_style(CrosshairStyle::Ring);
        assert_eq!(ch.ring_radius_pct, 0.06);
        assert_eq!(ch.thickness, 2.0);
        assert_eq!(ch.ring_style, RingStyle::Solid);
        assert_eq!(ch.opacity, 0.8);
    }

    #[test]
    fn default_cross_style_is_visible() {
        let ch = Crosshair::default_for_style(CrosshairStyle::Cross);
        assert_eq!(ch.size, 24.0);
        assert_eq!(ch.thickness, 2.0);
        assert_eq!(ch.gap, 4.0);
        assert_eq!(ch.opacity, 0.8);
    }

    #[test]
    fn custom_orb_edge_counts_and_positions() {
        let mut ch = Crosshair::default_for_style(CrosshairStyle::CustomOrb);
        assert_eq!(ch.radius, 6.0);
        assert_eq!(ch.offset, 30.0);
        assert_eq!(
            ch.orb_positions,
            OrbPosition(OrbPosition::TOP | OrbPosition::BOTTOM)
        );
        assert_eq!(ch.custom_orb_top_count, 3);
        assert_eq!(ch.custom_orb_bottom_count, 3);
        assert_eq!(ch.opacity, 0.7);
        assert!(ch.validate().is_ok());

        // 验证上边缘 3 个球均匀分布：首球在 1/4、尾球在 3/4 宽度处。
        let screen = (1920.0, 1080.0);
        let top_positions = [
            (screen.0 / 4.0, ch.offset),
            (screen.0 / 2.0, ch.offset),
            (screen.0 * 3.0 / 4.0, ch.offset),
        ];
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
        let ch = Crosshair::default_for_style(CrosshairStyle::BorderFrame);
        assert!(!ch.border_inset);
        assert_eq!(ch.border_frame_style, BorderFrameStyle::Solid);
        assert_eq!(ch.thickness, 6.0);
        assert_eq!(ch.offset, 24.0);
        assert_eq!(ch.opacity, 0.5);
        assert!(ch.validate().is_ok());
    }

    #[test]
    fn border_frame_inset_offsets() {
        let mut ch = Crosshair::default_for_style(CrosshairStyle::BorderFrame);
        assert_eq!(ch.offset, 24.0);
        assert_eq!(ch.thickness, 6.0);
        assert!(!ch.border_inset);
        ch.offset = 30.0;
        ch.thickness = 10.0;
        ch.border_inset = true;
        let _screen = (1920.0, 1080.0);
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
        let mut ch = Crosshair::default_for_style(CrosshairStyle::RandomOrb);
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
    fn all_styles_serialize_roundtrip() {
        for style in [
            CrosshairStyle::EdgeRect,
            CrosshairStyle::Cross,
            CrosshairStyle::LargeCross,
            CrosshairStyle::CornerDots4,
            CrosshairStyle::CornerDots6,
            CrosshairStyle::CornerDots8,
            CrosshairStyle::Ring,
            CrosshairStyle::CustomOrb,
            CrosshairStyle::RandomOrb,
            CrosshairStyle::BorderFrame,
            CrosshairStyle::EdgeArrows,
            CrosshairStyle::Grid,
        ] {
            let ch = Crosshair::default_for_style(style);
            let json = serde_json::to_string(&ch).unwrap();
            let restored: Crosshair = serde_json::from_str(&json).unwrap();
            assert_eq!(restored.style, style);
        }
    }

    #[test]
    fn edge_rect_alias_loads_old_toilet_paper() {
        let json = r#"{
            "style": "toilet_paper",
            "size": 120.0,
            "secondary_size": 80.0,
            "thickness": 2.0,
            "radius": 0.0,
            "offset": 0.0,
            "color": [1.0, 1.0, 1.0, 1.0],
            "opacity": 0.6,
            "gap": 4.0,
            "corner_radius": 4.0,
            "anchor": "top",
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
        }"#;
        let restored: Crosshair = serde_json::from_str(json).unwrap();
        assert_eq!(restored.style, CrosshairStyle::EdgeRect);
    }

    #[test]
    fn custom_image_validation_and_roundtrip() {
        // 默认配置中 image 字段都有默认值。
        let ch = Crosshair::default_crosshair();
        assert!(ch.image_path.is_empty());
        assert_eq!(ch.image_scale, 1.0);
        assert_eq!(ch.image_offset_x, 0.0);
        assert_eq!(ch.image_offset_y, 0.0);

        // CustomImage 默认 image_path 为空 → 校验失败。
        let mut ch = Crosshair::default_for_style(CrosshairStyle::CustomImage);
        assert!(ch.validate().is_err());

        // 设置了路径 → 校验通过。
        ch.image_path = "/path/to/crosshair.png".to_string();
        ch.image_scale = 0.5;
        ch.image_offset_x = 10.0;
        ch.image_offset_y = -5.0;
        assert!(ch.validate().is_ok());

        // 序列化往返。
        let json = serde_json::to_string(&ch).unwrap();
        let restored: Crosshair = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.style, CrosshairStyle::CustomImage);
        assert_eq!(restored.image_path, "/path/to/crosshair.png");
        assert_eq!(restored.image_scale, 0.5);
        assert_eq!(restored.image_offset_x, 10.0);
        assert_eq!(restored.image_offset_y, -5.0);

        // image_scale <= 0 → 校验失败。
        let mut ch = Crosshair::default_crosshair();
        ch.image_scale = 0.0;
        assert!(ch.validate().is_err());

        let mut ch = Crosshair::default_crosshair();
        ch.image_scale = -1.0;
        assert!(ch.validate().is_err());
    }

    #[test]
    fn old_config_without_image_fields_loads() {
        // 模拟旧配置文件（没有 image_* 字段），确保 serde 默认值生效。
        let json = r#"{
            "style": "cross",
            "size": 10.0,
            "secondary_size": 48.0,
            "thickness": 2.0,
            "radius": 0.0,
            "offset": 0.0,
            "color": [1.0, 1.0, 1.0, 1.0],
            "opacity": 0.8,
            "gap": 4.0,
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
        }"#;
        let restored: Crosshair = serde_json::from_str(json).unwrap();
        assert_eq!(restored.image_path, "");
        assert_eq!(restored.image_scale, 1.0);
        assert_eq!(restored.image_offset_x, 0.0);
        assert_eq!(restored.image_offset_y, 0.0);
    }

    #[test]
    fn app_settings_defaults() {
        let s = AppSettings::default();
        assert_eq!(s.auto_switch_on_overlay, "ask");
        assert_eq!(s.locale, "auto");
        assert!(s.fullscreen_overlay);
        assert!(!s.live_drag_preview);
        assert!(!s.gpu_acceleration);
        assert_eq!(s.update_channel, "stable");
        assert!(!s.cn_mirror);
        assert_eq!(s.mirror_url, "https://v4.gh-proxy.org");
        assert!(s.antialiasing);
        assert_eq!(s.quick_colors.len(), 5);
        assert_eq!(s.quick_colors[0], [1.0, 1.0, 1.0, 1.0]); // 白
        assert_eq!(s.hotkey_bindings.len(), 1);
        assert_eq!(s.hotkey_bindings[0].0, HotkeyAction::ToggleOverlay);
    }

    #[test]
    fn app_settings_roundtrip() {
        let s = AppSettings {
            auto_switch_on_overlay: "yes".to_string(),
            locale: "en".to_string(),
            fullscreen_overlay: false,
            live_drag_preview: true,
            gpu_acceleration: true,
            update_channel: "prerelease".to_string(),
            cn_mirror: true,
            mirror_url: "https://gh-proxy.org".to_string(),
            antialiasing: false,
            renderer_backend: RendererBackend::Cpu,
            quick_colors: default_quick_colors(),
            hotkey_bindings: default_hotkey_bindings(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let restored: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn old_config_without_settings_loads() {
        // 模拟旧配置文件（无 settings 字段），serde 应回退到 AppSettings::default()。
        let json = r#"{
            "active_profile": "default",
            "profiles": {
                "default": {
                    "crosshair": {
                        "style": "cross",
                        "size": 10.0,
                        "secondary_size": 48.0,
                        "thickness": 2.0,
                        "radius": 0.0,
                        "offset": 0.0,
                        "color": [1.0, 1.0, 1.0, 1.0],
                        "opacity": 0.8,
                        "gap": 4.0,
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
        let restored: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(restored.settings.auto_switch_on_overlay, "ask");
        assert_eq!(restored.settings.locale, "auto");
        assert!(restored.settings.fullscreen_overlay);
        assert!(!restored.settings.live_drag_preview);
        assert!(!restored.settings.gpu_acceleration);
        assert_eq!(restored.settings.update_channel, "stable");
        assert!(restored.settings.antialiasing);
    }

    #[test]
    fn grid_style_defaults_and_validation() {
        let mut ch = Crosshair::default_for_style(CrosshairStyle::Grid);
        // 默认值检查。
        assert_eq!(ch.grid_size, 120.0);
        assert_eq!(ch.thickness, 2.0);
        assert_eq!(ch.grid_alignment, GridAlignment::Center);
        assert_eq!(ch.opacity, 0.35);
        assert!(ch.validate().is_ok());

        // grid_size 超出范围 → 校验失败。
        ch.grid_size = 5.0;
        assert!(ch.validate().is_err());
        ch.grid_size = 600.0;
        assert!(ch.validate().is_err());

        // 正常值。
        ch.grid_size = 100.0;
        ch.grid_alignment = GridAlignment::Edge;
        assert!(ch.validate().is_ok());
    }

    #[test]
    fn grid_style_roundtrip() {
        let ch = Crosshair::default_for_style(CrosshairStyle::Grid);

        let json = serde_json::to_string(&ch).unwrap();
        let restored: Crosshair = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.style, CrosshairStyle::Grid);
        assert_eq!(restored.grid_size, 120.0);
        assert_eq!(restored.grid_alignment, GridAlignment::Center);
        assert_eq!(restored.thickness, 2.0);
    }

    #[test]
    fn grid_style_serializes_as_snake_case() {
        let json = serde_json::to_string(&CrosshairStyle::Grid).unwrap();
        assert_eq!(json, "\"grid\"");

        let restored: CrosshairStyle = serde_json::from_str("\"grid\"").unwrap();
        assert_eq!(restored, CrosshairStyle::Grid);
    }

    #[test]
    fn grid_alignment_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&GridAlignment::Center).unwrap(),
            "\"center\""
        );
        assert_eq!(
            serde_json::to_string(&GridAlignment::Edge).unwrap(),
            "\"edge\""
        );
    }

    #[test]
    fn old_config_without_grid_fields_loads() {
        // 旧配置文件中没有 grid_size / grid_alignment 字段，serde 应回退到默认值。
        let json = r#"{
            "style": "grid",
            "size": 10.0,
            "secondary_size": 48.0,
            "thickness": 2.0,
            "radius": 0.0,
            "offset": 0.0,
            "color": [1.0, 1.0, 1.0, 1.0],
            "opacity": 0.8,
            "gap": 4.0,
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
        }"#;
        let restored: Crosshair = serde_json::from_str(json).unwrap();
        assert_eq!(restored.style, CrosshairStyle::Grid);
        assert_eq!(restored.grid_size, 80.0); // 默认值
        assert_eq!(restored.grid_alignment, GridAlignment::Center); // 默认值
    }

    #[test]
    fn quick_colors_defaults() {
        let s = AppSettings::default();
        assert_eq!(s.quick_colors.len(), 5);
        // 白绿蓝红橙。
        assert_eq!(s.quick_colors[0], [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(s.quick_colors[1], [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(s.quick_colors[2], [0.2, 0.5, 1.0, 1.0]);
        assert_eq!(s.quick_colors[3], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(s.quick_colors[4], [1.0, 0.5, 0.0, 1.0]);
    }

    #[test]
    fn quick_colors_roundtrip() {
        let s = AppSettings {
            auto_switch_on_overlay: "ask".to_string(),
            locale: "auto".to_string(),
            fullscreen_overlay: true,
            live_drag_preview: false,
            gpu_acceleration: false,
            update_channel: "stable".to_string(),
            cn_mirror: false,
            mirror_url: "https://v4.gh-proxy.org".to_string(),
            antialiasing: true,
            renderer_backend: RendererBackend::Cpu,
            quick_colors: [
                [0.1, 0.2, 0.3, 1.0],
                [0.4, 0.5, 0.6, 1.0],
                [0.7, 0.8, 0.9, 1.0],
                [0.0, 0.0, 0.0, 1.0],
                [1.0, 1.0, 0.0, 1.0],
            ],
            hotkey_bindings: default_hotkey_bindings(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let restored: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.quick_colors, s.quick_colors);
    }

    #[test]
    fn hotkey_bindings_defaults() {
        let s = AppSettings::default();
        assert_eq!(s.hotkey_bindings.len(), 1);
        assert_eq!(s.hotkey_bindings[0].0, HotkeyAction::ToggleOverlay);
        assert_eq!(s.hotkey_bindings[0].1, "Ctrl+Alt+O");
    }

    #[test]
    fn hotkey_action_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&HotkeyAction::ToggleOverlay).unwrap(),
            "\"toggle_overlay\""
        );
        assert_eq!(
            serde_json::to_string(&HotkeyAction::CycleColorNext).unwrap(),
            "\"cycle_color_next\""
        );
        assert_eq!(
            serde_json::to_string(&HotkeyAction::SetColor3).unwrap(),
            "\"set_color_3\""
        );
    }

    #[test]
    fn hotkey_bindings_roundtrip() {
        let bindings = vec![
            (HotkeyAction::ToggleOverlay, "Ctrl+Alt+O".to_string()),
            (HotkeyAction::CycleColorNext, "Ctrl+Shift+Tab".to_string()),
            (HotkeyAction::SetColor1, "Alt+1".to_string()),
        ];
        let s = AppSettings {
            auto_switch_on_overlay: "ask".to_string(),
            locale: "auto".to_string(),
            fullscreen_overlay: true,
            live_drag_preview: false,
            gpu_acceleration: false,
            update_channel: "stable".to_string(),
            cn_mirror: false,
            mirror_url: "https://v4.gh-proxy.org".to_string(),
            antialiasing: true,
            renderer_backend: RendererBackend::Cpu,
            quick_colors: default_quick_colors(),
            hotkey_bindings: bindings.clone(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let restored: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.hotkey_bindings, bindings);
    }

    #[test]
    fn old_settings_without_hotkey_bindings_loads() {
        // 旧配置没有 hotkey_bindings / quick_colors 字段，应回退到默认值。
        let json = r#"{
            "auto_switch_on_overlay": "ask",
            "locale": "auto",
            "fullscreen_overlay": true,
            "live_drag_preview": false,
            "gpu_acceleration": false,
            "update_channel": "stable",
            "cn_mirror": false,
            "mirror_url": "https://v4.gh-proxy.org",
            "antialiasing": true
        }"#;
        let restored: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(restored.hotkey_bindings.len(), 1);
        assert_eq!(restored.hotkey_bindings[0].0, HotkeyAction::ToggleOverlay);
        assert_eq!(restored.quick_colors.len(), 5);
    }

    // ===== 四层架构类型测试 =====

    #[test]
    fn element_rect_serialization() {
        let e = Element::Rect {
            x: 10.0,
            y: 20.0,
            w: 100.0,
            h: 50.0,
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"type\":\"rect\""));
        assert!(json.contains("\"x\":10.0"));
        let restored: Element = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, e);
    }

    #[test]
    fn element_circle_serialization() {
        let e = Element::Circle {
            cx: 5.0,
            cy: 5.0,
            radius: 2.0,
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"type\":\"circle\""));
        let restored: Element = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, e);
    }

    #[test]
    fn element_text_serialization() {
        let e = Element::Text {
            x: 0.0,
            y: 16.0,
            content: "Hello".to_string(),
            font_size: 16.0,
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn element_polygon_serialization() {
        let e = Element::Polygon {
            points: vec![[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]],
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"type\":\"polygon\""));
        let restored: Element = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, e);
    }

    #[test]
    fn material_ref_builtin_serialization() {
        let r = MaterialRef::Builtin {
            id: "builtin.cross".to_string(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"kind\":\"builtin\""));
        assert!(json.contains("\"id\":\"builtin.cross\""));
        let restored: MaterialRef = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, r);
        assert_eq!(r.material_id(), "builtin.cross");
        assert!(r.is_builtin());
    }

    #[test]
    fn material_ref_user_serialization() {
        let r = MaterialRef::User {
            name: "user.my_cross".to_string(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"kind\":\"user\""));
        assert_eq!(r.material_id(), "user.my_cross");
        assert!(!r.is_builtin());
    }

    #[test]
    fn transform2d_defaults() {
        let t = Transform2D::default();
        assert_eq!(t.offset_x, 0.0);
        assert_eq!(t.offset_y, 0.0);
        assert_eq!(t.scale, 1.0);
        assert_eq!(t.rotation_deg, 0.0);
    }

    #[test]
    fn transform2d_partial_deserialize() {
        // 只提供 offset_x，其他字段回退默认。
        let json = r#"{"offset_x": 10.0}"#;
        let t: Transform2D = serde_json::from_str(json).unwrap();
        assert_eq!(t.offset_x, 10.0);
        assert_eq!(t.offset_y, 0.0);
        assert_eq!(t.scale, 1.0);
        assert_eq!(t.rotation_deg, 0.0);
    }

    #[test]
    fn layer_style_defaults() {
        let s = LayerStyle::default();
        assert_eq!(s.color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(s.opacity, 0.6);
        assert_eq!(s.blend_mode, BlendMode::Normal);
    }

    #[test]
    fn layer_new_basic() {
        let l = Layer::new(
            "layer-1",
            "中心十字",
            MaterialRef::Builtin {
                id: "builtin.cross".to_string(),
            },
        );
        assert_eq!(l.id, "layer-1");
        assert_eq!(l.name, "中心十字");
        assert!(l.visible);
        assert!(!l.locked);
    }

    #[test]
    fn layer_partial_deserialize() {
        // 最小合法 Layer JSON：只提供必填字段。
        let json = r#"{
            "id": "l1",
            "name": "test",
            "material": {"kind": "builtin", "id": "builtin.cross"}
        }"#;
        let l: Layer = serde_json::from_str(json).unwrap();
        assert_eq!(l.id, "l1");
        assert!(l.visible); // 默认 true
        assert!(!l.locked); // 默认 false
        assert_eq!(l.transform.scale, 1.0); // 默认
        assert_eq!(l.style.opacity, 0.6); // 默认
    }

    #[test]
    fn rect_width_height_center() {
        let r = Rect {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1920.0,
            max_y: 1080.0,
        };
        assert_eq!(r.width(), 1920.0);
        assert_eq!(r.height(), 1080.0);
        assert_eq!(r.center_x(), 960.0);
        assert_eq!(r.center_y(), 540.0);
    }
}
