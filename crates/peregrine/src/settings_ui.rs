//! egui 设置面板状态与绘制逻辑。
//!
//! 负责把当前配置展示为可编辑的 UI，并返回用户是否做了修改。
//! 界面采用左右布局：左侧为演示窗口，右侧为配置项。

use egui::{Color32, ComboBox, Slider, Stroke, Vec2};
use peregrine_config::{
    Anchor, AppConfig, BorderFrameStyle, ConfigSnapshot, CrosshairStyle, OrbPosition, RingStyle,
};

/// UI 状态容器。
#[derive(Default)]
pub struct SettingsUi {
    /// 最近一次帧的用户操作结果。
    response: SettingsResponse,
    /// Overlay 是否正在运行（由 main.rs 每帧同步，控制按钮文字）。
    pub overlay_active: bool,
    /// Overlay 窗口的宽高比（由 main.rs 每帧同步，None 时用默认 16:9）。
    /// 预览区按此比例缩放，确保所见即所得。
    pub overlay_aspect_ratio: Option<f32>,
    /// 缓存的窗口标题列表，避免每帧调用 EnumWindows。
    #[allow(dead_code)]
    cached_window_titles: Vec<String>,
    /// 已查询过宽高比的目标窗口标题（避免重复查询）。
    cached_aspect_for: String,
    /// 缓存的自定义图片纹理（路径变化时重新加载）。
    cached_image: Option<(String, std::sync::Arc<egui::TextureHandle>)>,
}

impl std::fmt::Debug for SettingsUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsUi")
            .field("response", &self.response)
            .field("overlay_active", &self.overlay_active)
            .field("cached_image", &self.cached_image.as_ref().map(|(p, _)| p))
            .finish()
    }
}

/// UI 一帧的返回值。
#[derive(Debug, Clone)]
pub struct SettingsResponse {
    /// 用户是否在本帧修改了配置。
    pub changed: bool,
    /// 修改后的配置副本。
    pub config: ConfigSnapshot,
    /// 用户是否点击了"开始覆盖"按钮，请求创建 Overlay。
    pub start_overlay: bool,
    /// 用户是否点击了"停止覆盖"按钮，请求销毁 Overlay。
    pub stop_overlay: bool,
}

impl Default for SettingsResponse {
    fn default() -> Self {
        Self {
            changed: false,
            config: ConfigSnapshot::new(AppConfig::default_config()),
            start_overlay: false,
            stop_overlay: false,
        }
    }
}

impl SettingsUi {
    /// 创建新的 UI 状态。
    pub fn new() -> Self {
        Self::default()
    }

    /// 绘制设置界面。
    ///
    /// 使用左右布局：左侧为演示窗口（占满剩余空间），右侧为配置项面板。
    /// 配置修改会反应在左侧实时预览中，并通过 [`SettingsResponse`] 通知调用方持久化。
    pub fn ui(&mut self, ctx: &egui::Context, config: &ConfigSnapshot) {
        let original = (**config).clone();
        let mut new_config = (**config).clone();
        let mut start_overlay = false;
        let mut stop_overlay = false;
        let overlay_active = self.overlay_active;

        // 先把当前 target_window 取出来，供「选择窗口」按钮使用，
        // 避免在闭包内与 crosshair 的可变借用产生交叉借用。
        let current_target_window = new_config
            .active_profile()
            .map(|p| p.target_window.clone())
            .unwrap_or_default();

        // 如果目标窗口存在且尚未查询过宽高比，查询一次。
        #[cfg(windows)]
        {
            if !current_target_window.is_empty() && self.cached_aspect_for != current_target_window
            {
                self.cached_aspect_for = current_target_window.clone();
                self.overlay_aspect_ratio =
                    crate::platform::windows::target_window_aspect(&current_target_window);
                tracing::info!(
                    target = %current_target_window,
                    aspect = ?self.overlay_aspect_ratio,
                    "initial aspect ratio query"
                );
            }
        }
        // 按钮点击后写入的新标题，闭包结束后写回 profile。
        // Windows 下在闭包中赋值；非 Windows 下不修改，用 allow 消除警告。
        #[allow(unused_mut)]
        let mut new_target_window: Option<String> = None;

        let profile = new_config
            .active_profile_mut()
            .expect("active profile exists");
        let crosshair = &mut profile.crosshair;

        // 右侧：配置项面板。
        egui::SidePanel::right("settings_panel")
            .default_width(260.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(240.0);
                ui.set_max_width(280.0);

                // 类型下拉框。
                ui.horizontal(|ui| {
                    ui.label("类型：");
                    ComboBox::from_id_salt("crosshair_style")
                        .selected_text(style_display_name(crosshair.style))
                        .show_ui(ui, |ui| {
                            for style in all_styles() {
                                ui.selectable_value(
                                    &mut crosshair.style,
                                    style,
                                    style_display_name(style),
                                );
                            }
                        });
                });

                // 公共配置：透明度、颜色。
                ui.add(
                    Slider::new(&mut crosshair.opacity, 0.0..=1.0)
                        .text("透明度")
                        .clamping(egui::SliderClamping::Always),
                );

                ui.horizontal(|ui| {
                    ui.label("颜色：");
                    ui.color_edit_button_rgba_unmultiplied(&mut crosshair.color);
                });

                ui.separator();

                // 按类型显示专属配置项。
                match crosshair.style {
                    CrosshairStyle::ToiletPaper => {
                        ui.label("卫生纸尺寸");
                        ui.add(Slider::new(&mut crosshair.size, 10.0..=400.0).text("宽度"));
                        ui.add(
                            Slider::new(&mut crosshair.secondary_size, 10.0..=300.0).text("高度"),
                        );
                        ui.add(Slider::new(&mut crosshair.corner_radius, 0.0..=60.0).text("圆角"));

                        ui.horizontal(|ui| {
                            ui.label("贴边：");
                            ComboBox::from_id_salt("toilet_paper_anchor")
                                .selected_text(anchor_display_name(crosshair.anchor))
                                .show_ui(ui, |ui| {
                                    for anchor in all_anchors() {
                                        ui.selectable_value(
                                            &mut crosshair.anchor,
                                            anchor,
                                            anchor_display_name(anchor),
                                        );
                                    }
                                });
                        });

                        ui.add(Slider::new(&mut crosshair.margin, 0.0..=200.0).text("边距"));
                    }
                    CrosshairStyle::Cross => {
                        ui.label("准星尺寸");
                        ui.add(Slider::new(&mut crosshair.size, 5.0..=200.0).text("臂长"));
                        ui.add(Slider::new(&mut crosshair.thickness, 1.0..=20.0).text("线宽"));
                        ui.add(Slider::new(&mut crosshair.gap, 0.0..=50.0).text("中心间隙"));
                    }
                    CrosshairStyle::LargeCross => {
                        ui.label("大准星尺寸");
                        ui.add(Slider::new(&mut crosshair.thickness, 1.0..=30.0).text("线宽"));
                    }
                    CrosshairStyle::CornerDots4
                    | CrosshairStyle::CornerDots6
                    | CrosshairStyle::CornerDots8 => {
                        ui.label("定位球尺寸");
                        ui.add(Slider::new(&mut crosshair.offset, 0.0..=200.0).text("距边缘距离"));
                        let radius_label = if crosshair.radius > 0.0 {
                            "半径"
                        } else {
                            "半径（0=自动）"
                        };
                        ui.add(Slider::new(&mut crosshair.radius, 0.0..=80.0).text(radius_label));
                        ui.add(
                            Slider::new(&mut crosshair.thickness, 1.0..=20.0)
                                .text("线宽（自动半径时生效）"),
                        );
                    }
                    CrosshairStyle::Ring => {
                        ui.label("中心环");
                        ui.add(
                            Slider::new(&mut crosshair.ring_radius_pct, 0.03..=0.08)
                                .text("半径占屏高比例"),
                        );
                        ui.add(Slider::new(&mut crosshair.thickness, 1.0..=3.0).text("线宽"));
                        ui.horizontal(|ui| {
                            ui.label("线型：");
                            ComboBox::from_id_salt("ring_style")
                                .selected_text(ring_style_display_name(crosshair.ring_style))
                                .show_ui(ui, |ui| {
                                    for style in all_ring_styles() {
                                        ui.selectable_value(
                                            &mut crosshair.ring_style,
                                            style,
                                            ring_style_display_name(style),
                                        );
                                    }
                                });
                        });
                    }
                    CrosshairStyle::CustomOrb => {
                        ui.label("自定义定位球");
                        ui.add(Slider::new(&mut crosshair.radius, 4.0..=12.0).text("半径"));
                        ui.add(Slider::new(&mut crosshair.offset, 10.0..=200.0).text("距边缘距离"));
                        ui.add(
                            Slider::new(&mut crosshair.custom_orb_top_count, 1..=10)
                                .text("上边缘数量"),
                        );
                        ui.add(
                            Slider::new(&mut crosshair.custom_orb_bottom_count, 1..=10)
                                .text("下边缘数量"),
                        );
                        ui.add(
                            Slider::new(&mut crosshair.custom_orb_left_count, 1..=10)
                                .text("左边缘数量"),
                        );
                        ui.add(
                            Slider::new(&mut crosshair.custom_orb_right_count, 1..=10)
                                .text("右边缘数量"),
                        );
                        ui.horizontal(|ui| {
                            ui.label("启用：");
                            let mut top = crosshair.orb_positions.contains(OrbPosition::TOP);
                            let mut bottom = crosshair.orb_positions.contains(OrbPosition::BOTTOM);
                            let mut left = crosshair.orb_positions.contains(OrbPosition::LEFT);
                            let mut right = crosshair.orb_positions.contains(OrbPosition::RIGHT);
                            ui.checkbox(&mut top, "上");
                            ui.checkbox(&mut bottom, "下");
                            ui.checkbox(&mut left, "左");
                            ui.checkbox(&mut right, "右");
                            crosshair.orb_positions = OrbPosition::empty();
                            if top {
                                crosshair.orb_positions.insert(OrbPosition::TOP);
                            }
                            if bottom {
                                crosshair.orb_positions.insert(OrbPosition::BOTTOM);
                            }
                            if left {
                                crosshair.orb_positions.insert(OrbPosition::LEFT);
                            }
                            if right {
                                crosshair.orb_positions.insert(OrbPosition::RIGHT);
                            }
                        });
                    }
                    CrosshairStyle::RandomOrb => {
                        ui.label("随机球");
                        ui.add(
                            Slider::new(&mut crosshair.random_orb_count, 1..=10).text("每边数量"),
                        );
                        ui.add(
                            Slider::new(&mut crosshair.random_orb_offset, 0.0..=300.0)
                                .text("距边缘距离"),
                        );
                        ui.add(
                            Slider::new(&mut crosshair.random_orb_jitter, 0.0..=200.0)
                                .text("位置扰动"),
                        );
                        ui.add(
                            Slider::new(&mut crosshair.random_radius_min, 4.0..=12.0)
                                .text("最小半径"),
                        );
                        ui.add(
                            Slider::new(&mut crosshair.random_radius_max, 4.0..=12.0)
                                .text("最大半径"),
                        );
                    }
                    CrosshairStyle::BorderFrame => {
                        ui.label("边框");
                        ui.add(
                            Slider::new(&mut crosshair.thickness, 1.0..=20.0).text("矩形条高度"),
                        );
                        ui.add(Slider::new(&mut crosshair.offset, 0.0..=100.0).text("距边缘距离"));
                        ui.horizontal(|ui| {
                            ui.label("样式：");
                            ComboBox::from_id_salt("border_frame_style")
                                .selected_text(border_frame_style_display_name(
                                    crosshair.border_frame_style,
                                ))
                                .show_ui(ui, |ui| {
                                    for style in all_border_frame_styles() {
                                        ui.selectable_value(
                                            &mut crosshair.border_frame_style,
                                            style,
                                            border_frame_style_display_name(style),
                                        );
                                    }
                                });
                        });
                        ui.checkbox(&mut crosshair.border_gap, "四边中间留 20% 缺口");
                    }
                    CrosshairStyle::CustomImage => {
                        ui.label("自定义图片");
                        ui.horizontal(|ui| {
                            ui.label("文件：");
                            ui.add(
                                egui::TextEdit::singleline(&mut crosshair.image_path)
                                    .desired_width(160.0)
                                    .hint_text("PNG 文件路径"),
                            );
                            if ui.button("浏览…").clicked() {
                                if let Some(path) =
                                    std::env::current_dir().ok().and_then(|_| pick_png_file())
                                {
                                    crosshair.image_path = path;
                                }
                            }
                        });
                        ui.add(
                            Slider::new(&mut crosshair.image_scale, 0.1..=5.0)
                                .text("缩放比例")
                                .step_by(0.1),
                        );
                        ui.add(
                            Slider::new(&mut crosshair.image_offset_x, -500.0..=500.0)
                                .text("水平偏移"),
                        );
                        ui.add(
                            Slider::new(&mut crosshair.image_offset_y, -500.0..=500.0)
                                .text("垂直偏移"),
                        );
                    }
                    CrosshairStyle::EdgeArrows => {
                        ui.label("箭头");
                        ui.add(
                            Slider::new(&mut crosshair.size, 4.0..=60.0)
                                .text("箭头大小")
                                .step_by(1.0),
                        );
                        ui.add(
                            Slider::new(&mut crosshair.arrow_width, 0.0..=72.0)
                                .text("宽度(0=等箭头)")
                                .step_by(1.0),
                        );
                        ui.checkbox(&mut crosshair.arrow_tail_per_edge, "分别设置尾巴长度");
                        if crosshair.arrow_tail_per_edge {
                            ui.add(
                                Slider::new(&mut crosshair.arrow_tail_top, 0.0..=500.0)
                                    .text("上尾巴")
                                    .step_by(1.0),
                            );
                            ui.add(
                                Slider::new(&mut crosshair.arrow_tail_bottom, 0.0..=500.0)
                                    .text("下尾巴")
                                    .step_by(1.0),
                            );
                            ui.add(
                                Slider::new(&mut crosshair.arrow_tail_left, 0.0..=500.0)
                                    .text("左尾巴")
                                    .step_by(1.0),
                            );
                            ui.add(
                                Slider::new(&mut crosshair.arrow_tail_right, 0.0..=500.0)
                                    .text("右尾巴")
                                    .step_by(1.0),
                            );
                        } else {
                            ui.add(
                                Slider::new(&mut crosshair.arrow_distance, 0.0..=500.0)
                                    .text("尾巴长度")
                                    .step_by(1.0),
                            );
                        }
                        ui.horizontal(|ui| {
                            ui.label("显示：");
                            let mut top = crosshair.orb_positions.contains(OrbPosition::TOP);
                            let mut bottom = crosshair.orb_positions.contains(OrbPosition::BOTTOM);
                            let mut left = crosshair.orb_positions.contains(OrbPosition::LEFT);
                            let mut right = crosshair.orb_positions.contains(OrbPosition::RIGHT);
                            ui.checkbox(&mut top, "上");
                            ui.checkbox(&mut bottom, "下");
                            ui.checkbox(&mut left, "左");
                            ui.checkbox(&mut right, "右");
                            crosshair.orb_positions = OrbPosition::empty();
                            if top {
                                crosshair.orb_positions.insert(OrbPosition::TOP);
                            }
                            if bottom {
                                crosshair.orb_positions.insert(OrbPosition::BOTTOM);
                            }
                            if left {
                                crosshair.orb_positions.insert(OrbPosition::LEFT);
                            }
                            if right {
                                crosshair.orb_positions.insert(OrbPosition::RIGHT);
                            }
                        });
                    }
                }

                // 选择目标窗口：下拉列表枚举当前可见的顶层窗口。
                // 仅 Windows 平台支持窗口枚举。
                // 窗口列表缓存在 SettingsUi 中，点击「刷新」按钮才重新枚举，
                // 避免每帧调用 EnumWindows 造成性能问题和日志刷屏。
                ui.horizontal(|ui| {
                    ui.label("目标窗口：");
                    #[cfg(windows)]
                    {
                        // 截断过长的窗口标题，最多显示 30 字符。
                        fn truncate_title(s: &str) -> String {
                            const MAX_LEN: usize = 30;
                            if s.chars().count() > MAX_LEN {
                                let truncated: String = s.chars().take(MAX_LEN).collect();
                                format!("{}…", truncated)
                            } else {
                                s.to_string()
                            }
                        }
                        let selected_text = if current_target_window.is_empty() {
                            "（未选择）".to_string()
                        } else {
                            truncate_title(&current_target_window)
                        };
                        ComboBox::from_id_salt("target_window_select")
                            .selected_text(selected_text)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut profile.target_window,
                                    String::new(),
                                    "（未选择）",
                                );
                                for title in &self.cached_window_titles {
                                    let display = truncate_title(title);
                                    ui.selectable_value(
                                        &mut profile.target_window,
                                        title.clone(),
                                        display,
                                    );
                                }
                            });
                        if profile.target_window != current_target_window {
                            new_target_window = Some(profile.target_window.clone());
                        }
                        // 刷新按钮：点击时重新枚举窗口。
                        if ui.button("🔄").clicked() {
                            self.cached_window_titles =
                                crate::platform::windows::list_window_titles();
                        }
                        // 首次打开时自动填充一次。
                        if self.cached_window_titles.is_empty() {
                            self.cached_window_titles =
                                crate::platform::windows::list_window_titles();
                        }
                    }
                    #[cfg(not(windows))]
                    {
                        let _ = &current_target_window;
                        ui.label("（当前平台不支持）");
                    }
                });

                ui.separator();
                ui.add_space(4.0);
                // 根据状态显示「开始覆盖」或「停止覆盖」。
                if overlay_active {
                    if ui.button("■ 停止覆盖").clicked() {
                        stop_overlay = true;
                    }
                } else {
                    if ui.button("▶ 开始覆盖").clicked() {
                        start_overlay = true;
                    }
                }

                // 底部水印：署名与许可提示。
                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(
                            "PolyForm Noncommercial 1.0.0 · 个人免费 · 禁止商业贩卖",
                        )
                        .small()
                        .color(egui::Color32::from_gray(120)),
                    );
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        ui.hyperlink_to(
                            egui::RichText::new("Peregrine")
                                .small()
                                .color(egui::Color32::from_gray(120)),
                            "https://github.com/Eeymoo/peregrine",
                        );
                        ui.label(
                            egui::RichText::new("© eeymoo · 免费使用")
                                .small()
                                .color(egui::Color32::from_gray(120)),
                        );
                    });
                });
            });

        // 左侧：演示窗口。
        // 预览区保持与实际 overlay（目标窗口）相同的长宽比，
        // 这样预览中准心的位置/大小与最终 overlay 只是缩放关系。
        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_rect_before_wrap();
            // 使用 overlay 窗口的实际宽高比；未运行时默认 16:9。
            let target_ratio = self.overlay_aspect_ratio.unwrap_or(16.0 / 9.0);
            let avail_w = avail.width();
            let avail_h = avail.height();
            let avail_ratio = avail_w / avail_h;
            // contain 缩放：在可用空间内放置最大的目标比例矩形。
            let (pw, ph) = if avail_ratio > target_ratio {
                (avail_h * target_ratio, avail_h)
            } else {
                (avail_w, avail_w / target_ratio)
            };
            let preview_rect = egui::Rect::from_center_size(avail.center(), egui::vec2(pw, ph));
            draw_checkerboard_background(ui, preview_rect);
            draw_preview_shape(ui, preview_rect, crosshair, ctx, &mut self.cached_image);
        });

        // 闭包结束后，把「选择窗口」按钮的选择结果写回 profile。
        if let Some(tw) = &new_target_window {
            if let Some(profile) = new_config.active_profile_mut() {
                profile.target_window = tw.clone();
            }
            // 目标窗口变化时清除缓存，下一帧会重新查询宽高比。
            if tw.is_empty() {
                self.overlay_aspect_ratio = None;
                self.cached_aspect_for.clear();
            } else {
                self.cached_aspect_for.clear();
            }
        }

        let changed = new_config != original;
        self.response = SettingsResponse {
            changed,
            config: ConfigSnapshot::new(new_config),
            start_overlay,
            stop_overlay,
        };
    }

    /// 取走本帧的响应。
    pub fn take_response(&mut self) -> SettingsResponse {
        std::mem::replace(&mut self.response, SettingsResponse::default())
    }
}

/// 所有可用的辅助贴图样式。
fn all_styles() -> [CrosshairStyle; 12] {
    [
        CrosshairStyle::ToiletPaper,
        CrosshairStyle::Cross,
        CrosshairStyle::LargeCross,
        CrosshairStyle::CornerDots4,
        CrosshairStyle::CornerDots6,
        CrosshairStyle::CornerDots8,
        CrosshairStyle::Ring,
        CrosshairStyle::CustomOrb,
        CrosshairStyle::RandomOrb,
        CrosshairStyle::BorderFrame,
        CrosshairStyle::CustomImage,
        CrosshairStyle::EdgeArrows,
    ]
}

/// 样式在 UI 上的显示名称。
fn style_display_name(style: CrosshairStyle) -> String {
    match style {
        CrosshairStyle::ToiletPaper => "卫生纸".to_string(),
        CrosshairStyle::Cross => "准星".to_string(),
        CrosshairStyle::LargeCross => "大准星".to_string(),
        CrosshairStyle::CornerDots4 => "定位球4".to_string(),
        CrosshairStyle::CornerDots6 => "定位球6".to_string(),
        CrosshairStyle::CornerDots8 => "定位球8".to_string(),
        CrosshairStyle::Ring => "中心环".to_string(),
        CrosshairStyle::CustomOrb => "自定义定位球".to_string(),
        CrosshairStyle::RandomOrb => "随机球".to_string(),
        CrosshairStyle::BorderFrame => "边框".to_string(),
        CrosshairStyle::CustomImage => "自定义图片".to_string(),
        CrosshairStyle::EdgeArrows => "箭头".to_string(),
    }
}

/// 所有可用的贴边位置。
fn all_anchors() -> [Anchor; 5] {
    [
        Anchor::Top,
        Anchor::Bottom,
        Anchor::Left,
        Anchor::Right,
        Anchor::Center,
    ]
}

/// 贴边位置在 UI 上的显示名称。
fn anchor_display_name(anchor: Anchor) -> String {
    match anchor {
        Anchor::Top => "顶部".to_string(),
        Anchor::Bottom => "底部".to_string(),
        Anchor::Left => "左侧".to_string(),
        Anchor::Right => "右侧".to_string(),
        Anchor::Center => "居中".to_string(),
    }
}

/// 所有可用的中心环线型。
fn all_ring_styles() -> [RingStyle; 3] {
    [RingStyle::Solid, RingStyle::Dashed, RingStyle::Double]
}

/// 中心环线型在 UI 上的显示名称。
fn ring_style_display_name(style: RingStyle) -> String {
    match style {
        RingStyle::Solid => "实线".to_string(),
        RingStyle::Dashed => "虚线".to_string(),
        RingStyle::Double => "双环".to_string(),
    }
}

/// 所有可用的边框样式。
fn all_border_frame_styles() -> [BorderFrameStyle; 2] {
    [BorderFrameStyle::Solid, BorderFrameStyle::Gap]
}

/// 边框样式在 UI 上的显示名称。
fn border_frame_style_display_name(style: BorderFrameStyle) -> String {
    match style {
        BorderFrameStyle::Solid => "完整".to_string(),
        BorderFrameStyle::Gap => "四边缺口".to_string(),
    }
}

/// 在演示区域绘制差色网格背景，便于观察透明贴图效果。
fn draw_checkerboard_background(ui: &mut egui::Ui, rect: egui::Rect) {
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, Color32::from_gray(30));

    let cell_size = 20.0;
    let mut y = rect.min.y;
    let mut row = 0i32;
    while y < rect.max.y {
        let mut x = rect.min.x;
        let mut col = 0i32;
        while x < rect.max.x {
            let color = if (row + col) % 2 == 0 {
                Color32::from_gray(60)
            } else {
                Color32::from_gray(90)
            };
            let cell_rect =
                egui::Rect::from_min_size(egui::pos2(x, y), Vec2::new(cell_size, cell_size))
                    .intersect(rect);
            ui.painter()
                .rect_filled(cell_rect, egui::CornerRadius::ZERO, color);
            x += cell_size;
            col += 1;
        }
        y += cell_size;
        row += 1;
    }

    // 绘制一个细边框标识演示窗口边界。
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_gray(150)),
        egui::StrokeKind::Inside,
    );
}

/// 在演示区域绘制当前选中的辅助贴图。
///
/// 使用 [`crate::shapes::build_shapes`] 生成与覆盖层完全一致的几何图元，
/// 再用 egui painter 渲染，确保预览即所得。
#[allow(clippy::too_many_arguments)]
fn draw_preview_shape(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    crosshair: &peregrine_config::Crosshair,
    ctx: &egui::Context,
    cached_image: &mut Option<(String, std::sync::Arc<egui::TextureHandle>)>,
) {
    // CustomImage 不走矢量图元，单独处理。
    if crosshair.style == CrosshairStyle::CustomImage {
        draw_preview_image(ui, rect, crosshair, ctx, cached_image);
        return;
    }

    let color = apply_opacity(color_f32_to_color32(&crosshair.color), crosshair.opacity);

    // 用共享几何模块生成图元，确保预览与 overlay 完全一致。
    let screen = crate::shapes::RectF {
        min_x: rect.min.x,
        min_y: rect.min.y,
        max_x: rect.max.x,
        max_y: rect.max.y,
    };
    let shapes = crate::shapes::build_shapes(&screen, crosshair);
    for shape in shapes {
        render_shape_egui(ui, &shape, color);
    }
}

/// 将一条 [`Shape`]（共享几何图元）用 egui painter 渲染。
///
/// 与 overlay_renderer 中 `rasterize_shape` 一一对应，确保两套渲染输出相同图形。
fn render_shape_egui(ui: &mut egui::Ui, shape: &crate::shapes::Shape, color: Color32) {
    use crate::shapes::Shape;
    match shape {
        Shape::Rect { x, y, w, h } => {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(egui::pos2(*x, *y), Vec2::new(*w, *h)),
                egui::CornerRadius::ZERO,
                color,
            );
        }
        Shape::Circle { cx, cy, radius } => {
            ui.painter()
                .circle_filled(egui::pos2(*cx, *cy), *radius, color);
        }
        Shape::CircleStroke {
            cx,
            cy,
            radius,
            thickness,
        } => {
            ui.painter().circle_stroke(
                egui::pos2(*cx, *cy),
                *radius,
                Stroke::new(*thickness, color),
            );
        }
        Shape::DashedCircle {
            cx,
            cy,
            radius,
            thickness,
            dash_len,
            gap_len,
        } => {
            render_dashed_circle_egui(
                ui,
                egui::pos2(*cx, *cy),
                *radius,
                *thickness,
                color,
                *dash_len,
                *gap_len,
            );
        }
        Shape::Triangle {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        } => {
            let points = vec![
                egui::pos2(*x1, *y1),
                egui::pos2(*x2, *y2),
                egui::pos2(*x3, *y3),
            ];
            ui.painter().add(egui::Shape::convex_polygon(
                points,
                color,
                egui::Stroke::NONE,
            ));
        }
    }
}

/// 用 egui painter 绘制虚线圆。
fn render_dashed_circle_egui(
    ui: &mut egui::Ui,
    center: egui::Pos2,
    radius: f32,
    thickness: f32,
    color: Color32,
    dash_len: f32,
    gap_len: f32,
) {
    let circumference = 2.0 * std::f32::consts::PI * radius;
    let unit = dash_len + gap_len;
    let segments = (circumference / unit).ceil() as usize;
    let step_angle = 2.0 * std::f32::consts::PI / segments as f32;

    for i in 0..segments {
        let start_angle = i as f32 * step_angle;
        let end_angle = start_angle + step_angle * (dash_len / unit);
        let start = egui::pos2(
            center.x + radius * start_angle.cos(),
            center.y + radius * start_angle.sin(),
        );
        let end = egui::pos2(
            center.x + radius * end_angle.cos(),
            center.y + radius * end_angle.sin(),
        );
        ui.painter()
            .line_segment([start, end], Stroke::new(thickness, color));
    }
}

/// 把 [f32; 4] RGBA 转换为 egui Color32。
fn color_f32_to_color32(color: &[f32; 4]) -> Color32 {
    Color32::from_rgba_premultiplied(
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        (color[3] * 255.0) as u8,
    )
}

/// 把透明度应用到颜色上。
fn apply_opacity(color: Color32, opacity: f32) -> Color32 {
    let mut color = color;
    color[3] = (color[3] as f32 * opacity.clamp(0.0, 1.0)) as u8;
    color
}
// ===== 自定义图片预览 =====

/// 在设置面板预览区绘制 CustomImage。
///
/// 从 `image_path` 加载 PNG，用 egui 的 `TextureHandle` 缓存，
/// 按配置的缩放比例与偏移居中绘制在预览区。
fn draw_preview_image(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    crosshair: &peregrine_config::Crosshair,
    ctx: &egui::Context,
    cached_image: &mut Option<(String, std::sync::Arc<egui::TextureHandle>)>,
) {
    if crosshair.image_path.trim().is_empty() {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "请选择 PNG 文件",
            egui::FontId::proportional(14.0),
            egui::Color32::from_gray(120),
        );
        return;
    }

    // 检查路径是否变化，需要重新加载。
    let need_reload = match cached_image {
        Some((cached_path, _)) => cached_path != &crosshair.image_path,
        None => true,
    };

    if need_reload {
        match load_png_for_egui(&crosshair.image_path) {
            Ok((rgba, w, h)) => {
                let color_image = egui::ColorImage {
                    size: [w, h],
                    pixels: rgba,
                };
                let texture = std::sync::Arc::new(ctx.load_texture(
                    "peregrine_crosshair_img",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
                *cached_image = Some((crosshair.image_path.clone(), texture));
            }
            Err(e) => {
                *cached_image = None;
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("加载失败：{}", e),
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgb(200, 80, 80),
                );
                return;
            }
        }
    }

    let Some((_, texture)) = cached_image.as_ref() else {
        return;
    };

    let size = texture.size_vec2();
    let scaled = size * crosshair.image_scale;
    let center = egui::pos2(
        rect.center().x + crosshair.image_offset_x,
        rect.center().y + crosshair.image_offset_y,
    );
    let image_rect = egui::Rect::from_center_size(center, scaled);

    let tint = apply_opacity(
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 255),
        crosshair.opacity,
    );
    ui.painter().image(
        texture.id(),
        image_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        tint,
    );
}

/// 加载 PNG 文件为 egui 可用的 RGBA 像素。
fn load_png_for_egui(path: &str) -> Result<(Vec<egui::Color32>, usize, usize), String> {
    let decoder =
        png::Decoder::new(std::fs::File::open(path).map_err(|e| format!("打开文件失败：{}", e))?);
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("读取 PNG 头失败：{}", e))?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let frame = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("解码失败：{}", e))?;
    let bytes = &buf[..frame.buffer_size()];

    let w = info.width as usize;
    let h = info.height as usize;

    let pixels: Vec<egui::Color32> = match info.color_type {
        png::ColorType::Rgba => bytes
            .chunks_exact(4)
            .map(|c| egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
            .collect(),
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .map(|c| egui::Color32::from_rgb(c[0], c[1], c[2]))
            .collect(),
        png::ColorType::Grayscale => bytes.iter().map(|&v| egui::Color32::from_gray(v)).collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .map(|c| egui::Color32::from_rgba_unmultiplied(c[0], c[0], c[0], c[1]))
            .collect(),
        png::ColorType::Indexed => bytes
            .chunks_exact(3)
            .map(|c| egui::Color32::from_rgb(c[0], c[1], c[2]))
            .collect(),
    };

    Ok((pixels, w, h))
}

/// 弹出原生文件选择对话框，返回用户选择的 PNG 文件路径。
fn pick_png_file() -> Option<String> {
    // 使用 rfd（rust file dialog）弹出文件选择。
    // 如果 rfd 未引入，则返回 None（用户需手动粘贴路径）。
    None
}
