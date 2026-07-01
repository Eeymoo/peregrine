//! egui 设置面板状态与绘制逻辑。
//!
//! 负责把当前配置展示为可编辑的 UI，并返回用户是否做了修改。
//! 界面采用左右布局：左侧为演示窗口，右侧为配置项。

use egui::{Color32, ComboBox, CornerRadius, Slider, Stroke, Vec2};
use peregrine_config::{
    Anchor, AppConfig, BorderFrameStyle, ConfigSnapshot, CrosshairStyle, OrbPosition, RingStyle,
};

/// UI 状态容器。
#[derive(Debug, Default)]
pub struct SettingsUi {
    /// 最近一次帧的用户操作结果。
    response: SettingsResponse,
}

/// UI 一帧的返回值。
#[derive(Debug, Clone)]
pub struct SettingsResponse {
    /// 用户是否在本帧修改了配置。
    pub changed: bool,
    /// 修改后的配置副本。
    pub config: ConfigSnapshot,
    /// 用户是否点击了"开始覆盖"按钮，请求切换到 Overlay 模式。
    pub start_overlay: bool,
}

impl Default for SettingsResponse {
    fn default() -> Self {
        Self {
            changed: false,
            config: ConfigSnapshot::new(AppConfig::default_config()),
            start_overlay: false,
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
    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        config: &ConfigSnapshot,
    ) {
        let original = (**config).clone();
        let mut new_config = (**config).clone();
        let mut start_overlay = false;
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
                ui.heading("辅助贴图设置");
                ui.separator();

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
                        ui.add(Slider::new(&mut crosshair.secondary_size, 10.0..=300.0).text("高度"));
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
                        let radius_label = if crosshair.radius > 0.0 { "半径" } else { "半径（0=自动）" };
                        ui.add(Slider::new(&mut crosshair.radius, 0.0..=80.0).text(radius_label));
                        ui.add(Slider::new(&mut crosshair.thickness, 1.0..=20.0).text("线宽（自动半径时生效）"));
                    }
                    CrosshairStyle::Ring => {
                        ui.label("中心环");
                        ui.add(Slider::new(&mut crosshair.ring_radius_pct, 0.03..=0.08).text("半径占屏高比例"));
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
                        ui.add(Slider::new(&mut crosshair.custom_orb_top_count, 1..=10).text("上边缘数量"));
                        ui.add(Slider::new(&mut crosshair.custom_orb_bottom_count, 1..=10).text("下边缘数量"));
                        ui.add(Slider::new(&mut crosshair.custom_orb_left_count, 1..=10).text("左边缘数量"));
                        ui.add(Slider::new(&mut crosshair.custom_orb_right_count, 1..=10).text("右边缘数量"));
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
                            if top { crosshair.orb_positions.insert(OrbPosition::TOP); }
                            if bottom { crosshair.orb_positions.insert(OrbPosition::BOTTOM); }
                            if left { crosshair.orb_positions.insert(OrbPosition::LEFT); }
                            if right { crosshair.orb_positions.insert(OrbPosition::RIGHT); }
                        });
                    }
                    CrosshairStyle::RandomOrb => {
                        ui.label("随机球");
                        ui.add(Slider::new(&mut crosshair.random_orb_count, 1..=10).text("每边数量"));
                        ui.add(Slider::new(&mut crosshair.random_orb_offset, 0.0..=300.0).text("距边缘距离"));
                        ui.add(Slider::new(&mut crosshair.random_orb_jitter, 0.0..=200.0).text("位置扰动"));
                        ui.add(Slider::new(&mut crosshair.random_radius_min, 4.0..=12.0).text("最小半径"));
                        ui.add(Slider::new(&mut crosshair.random_radius_max, 4.0..=12.0).text("最大半径"));
                    }
                    CrosshairStyle::BorderFrame => {
                        ui.label("边框");
                        ui.add(Slider::new(&mut crosshair.thickness, 1.0..=20.0).text("矩形条高度"));
                        ui.add(Slider::new(&mut crosshair.offset, 0.0..=100.0).text("距边缘距离"));
                        ui.horizontal(|ui| {
                            ui.label("样式：");
                            ComboBox::from_id_salt("border_frame_style")
                                .selected_text(border_frame_style_display_name(crosshair.border_frame_style))
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
                }

                // 选择窗口按钮：Windows 下枚举顶层窗口并循环选中下一个。
                if ui.button("选择窗口").clicked() {
                    #[cfg(target_os = "windows")]
                    if let Some(next) = crate::platform::windows::next_window_title(&profile.target_window) {
                        profile.target_window = next;
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        profile.target_window = "选择窗口仅在 Windows 可用".to_string();
                    }
                }
                if !profile.target_window.is_empty() {
                    ui.label(format!(
                        "目标窗口：{}",
                        profile.target_window
                    ));
                }

                ui.separator();
                ui.label("提示：按 Tab 或点击下方按钮切换");
                ui.add_space(4.0);
                if ui.button("▶ 开始覆盖").clicked() {
                    start_overlay = true;
                }
            });

        // 左侧：演示窗口。
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            draw_checkerboard_background(ui, rect);
            draw_preview_shape(ui, rect, crosshair);
        });

        let changed = new_config != original;
        self.response = SettingsResponse {
            changed,
            config: ConfigSnapshot::new(new_config),
            start_overlay,
        };
    }

    /// 取走本帧的响应。
    pub fn take_response(&mut self,
    ) -> SettingsResponse {
        std::mem::replace(&mut self.response, SettingsResponse::default())
    }
}

/// 所有可用的辅助贴图样式。
fn all_styles() -> [CrosshairStyle; 10] {
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
    ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, Color32::from_gray(30));

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
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(x, y),
                Vec2::new(cell_size, cell_size),
            )
            .intersect(rect);
            ui.painter().rect_filled(cell_rect, egui::CornerRadius::ZERO, color);
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
fn draw_preview_shape(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    crosshair: &peregrine_config::Crosshair,
) {
    let center = rect.center();
    let color = apply_opacity(color_f32_to_color32(&crosshair.color), crosshair.opacity);

    match crosshair.style {
        CrosshairStyle::ToiletPaper => {
            // 根据贴边位置与边距计算矩形中心。
            let width = crosshair.size;
            let height = crosshair.secondary_size;
            let margin = crosshair.margin;
            let center = match crosshair.anchor {
                Anchor::Top => egui::pos2(rect.center().x, rect.min.y + height / 2.0 + margin),
                Anchor::Bottom => egui::pos2(rect.center().x, rect.max.y - height / 2.0 - margin),
                Anchor::Left => egui::pos2(rect.min.x + width / 2.0 + margin, rect.center().y),
                Anchor::Right => egui::pos2(rect.max.x - width / 2.0 - margin, rect.center().y),
                Anchor::Center => rect.center(),
            };
            let shape_rect = egui::Rect::from_center_size(
                center,
                Vec2::new(width, height),
            );
            ui.painter().rect_filled(
                shape_rect,
                CornerRadius::same(crosshair.corner_radius as u8),
                color,
            );
        }
        CrosshairStyle::Cross => {
            // 屏幕中心十字，预留中心间隙。
            let arm = crosshair.size;
            let half_gap = crosshair.gap / 2.0;
            let thickness = crosshair.thickness;
            let horizontal_left = egui::Rect::from_center_size(
                egui::pos2(center.x - (arm + half_gap) / 2.0 - half_gap / 2.0, center.y),
                Vec2::new(arm - half_gap, thickness),
            );
            let horizontal_right = egui::Rect::from_center_size(
                egui::pos2(center.x + (arm + half_gap) / 2.0 + half_gap / 2.0, center.y),
                Vec2::new(arm - half_gap, thickness),
            );
            let vertical_top = egui::Rect::from_center_size(
                egui::pos2(center.x, center.y - (arm + half_gap) / 2.0 - half_gap / 2.0),
                Vec2::new(thickness, arm - half_gap),
            );
            let vertical_bottom = egui::Rect::from_center_size(
                egui::pos2(center.x, center.y + (arm + half_gap) / 2.0 + half_gap / 2.0),
                Vec2::new(thickness, arm - half_gap),
            );
            for rect in [horizontal_left, horizontal_right, vertical_top, vertical_bottom] {
                ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, color);
            }
        }
        CrosshairStyle::LargeCross => {
            // 从屏幕边缘延伸到中心的水平线与垂直线。
            let thickness = crosshair.thickness;
            let horizontal = egui::Rect::from_min_max(
                egui::pos2(rect.min.x, center.y - thickness / 2.0),
                egui::pos2(rect.max.x, center.y + thickness / 2.0),
            );
            let vertical = egui::Rect::from_min_max(
                egui::pos2(center.x - thickness / 2.0, rect.min.y),
                egui::pos2(center.x + thickness / 2.0, rect.max.y),
            );
            ui.painter().rect_filled(horizontal, egui::CornerRadius::ZERO, color);
            ui.painter().rect_filled(vertical, egui::CornerRadius::ZERO, color);
        }
        CrosshairStyle::CornerDots4
        | CrosshairStyle::CornerDots6
        | CrosshairStyle::CornerDots8 => {
            // 四角圆形。
            let configured_offset = if crosshair.offset > 0.0 {
                crosshair.offset
            } else {
                crosshair.size
            };
            let offset = configured_offset.min(rect.width() / 4.0).min(rect.height() / 4.0);
            let radius = if crosshair.radius > 0.0 {
                crosshair.radius
            } else {
                crosshair.thickness * 3.0
            };
            let corners = [
                egui::pos2(rect.min.x + offset, rect.min.y + offset),
                egui::pos2(rect.max.x - offset, rect.min.y + offset),
                egui::pos2(rect.min.x + offset, rect.max.y - offset),
                egui::pos2(rect.max.x - offset, rect.max.y - offset),
            ];
            for pos in corners {
                ui.painter().circle_filled(pos, radius, color);
            }
            // 垂直中心圆形。
            if matches!(
                crosshair.style,
                CrosshairStyle::CornerDots6 | CrosshairStyle::CornerDots8
            ) {
                ui.painter().circle_filled(egui::pos2(center.x, rect.min.y + offset), radius, color);
                ui.painter().circle_filled(egui::pos2(center.x, rect.max.y - offset), radius, color);
            }
            // 水平中心圆形。
            if matches!(crosshair.style, CrosshairStyle::CornerDots8) {
                ui.painter().circle_filled(egui::pos2(rect.min.x + offset, center.y), radius, color);
                ui.painter().circle_filled(egui::pos2(rect.max.x - offset, center.y), radius, color);
            }
        }
        CrosshairStyle::Ring => {
            let radius = rect.height() * crosshair.ring_radius_pct;
            let thickness = crosshair.thickness;
            match crosshair.ring_style {
                RingStyle::Solid => {
                    ui.painter().circle_stroke(center, radius, Stroke::new(thickness, color));
                }
                RingStyle::Dashed => {
                    draw_dashed_circle(ui, center, radius, thickness, color, 4.0, 4.0);
                }
                RingStyle::Double => {
                    ui.painter().circle_stroke(center, radius - 2.0, Stroke::new(1.0, color));
                    draw_dashed_circle(ui, center, radius + 2.0, 1.0, color, 4.0, 4.0);
                }
            }
        }
        CrosshairStyle::CustomOrb => {
            let radius = crosshair.radius.max(1.0);
            let offset = crosshair.offset;
            let draw_edge_orbs = |ui: &mut egui::Ui, count: u32, positions: Vec<egui::Pos2>| {
                if count == 0 || positions.is_empty() {
                    return;
                }
                if count == 1 {
                    ui.painter().circle_filled(positions[positions.len() / 2], radius, color);
                    return;
                }
                let step = (positions.len() - 1) as f32 / (count + 1) as f32;
                for i in 1..=count {
                    let idx_f = i as f32 * step;
                    let idx0 = idx_f.floor() as usize;
                    let idx1 = (idx0 + 1).min(positions.len() - 1);
                    let t = idx_f - idx0 as f32;
                    let p0 = positions[idx0];
                    let p1 = positions[idx1];
                    let pos = p0 + (p1 - p0) * t;
                    ui.painter().circle_filled(pos, radius, color);
                }
            };

            if crosshair.orb_positions.contains(OrbPosition::TOP) {
                let positions: Vec<_> = (0..=16)
                    .map(|i| {
                        let x = egui::lerp(rect.min.x..=rect.max.x, i as f32 / 16.0);
                        egui::pos2(x, rect.min.y + offset)
                    })
                    .collect();
                draw_edge_orbs(ui, crosshair.custom_orb_top_count, positions);
            }
            if crosshair.orb_positions.contains(OrbPosition::BOTTOM) {
                let positions: Vec<_> = (0..=16)
                    .map(|i| {
                        let x = egui::lerp(rect.min.x..=rect.max.x, i as f32 / 16.0);
                        egui::pos2(x, rect.max.y - offset)
                    })
                    .collect();
                draw_edge_orbs(ui, crosshair.custom_orb_bottom_count, positions);
            }
            if crosshair.orb_positions.contains(OrbPosition::LEFT) {
                let positions: Vec<_> = (0..=16)
                    .map(|i| {
                        let y = egui::lerp(rect.min.y..=rect.max.y, i as f32 / 16.0);
                        egui::pos2(rect.min.x + offset, y)
                    })
                    .collect();
                draw_edge_orbs(ui, crosshair.custom_orb_left_count, positions);
            }
            if crosshair.orb_positions.contains(OrbPosition::RIGHT) {
                let positions: Vec<_> = (0..=16)
                    .map(|i| {
                        let y = egui::lerp(rect.min.y..=rect.max.y, i as f32 / 16.0);
                        egui::pos2(rect.max.x - offset, y)
                    })
                    .collect();
                draw_edge_orbs(ui, crosshair.custom_orb_right_count, positions);
            }
        }
        CrosshairStyle::RandomOrb => {
            // 使用配置值作为稳定种子，避免每帧闪烁。
            let seed = (crosshair.random_orb_offset * 1000.0) as u64
                + (crosshair.random_orb_jitter * 100.0) as u64
                + (crosshair.random_radius_min * 10.0) as u64
                + (crosshair.random_radius_max * 10.0) as u64;
            let mut rng = SimpleRng::new(seed);
            let count = crosshair.random_orb_count as usize;
            let offset = crosshair.random_orb_offset;
            let jitter = crosshair.random_orb_jitter;
            let min_r = crosshair.random_radius_min;
            let max_r = crosshair.random_radius_max;

            let radius_for = |rng: &mut SimpleRng| -> f32 {
                min_r + rng.next_f32() * (max_r - min_r)
            };
            let jitter_for = |rng: &mut SimpleRng| -> f32 {
                (rng.next_f32() - 0.5) * 2.0 * jitter
            };

            for _ in 0..count {
                let radius = radius_for(&mut rng);
                let x = egui::lerp(rect.min.x..=rect.max.x, rng.next_f32());
                let pos = egui::pos2(x + jitter_for(&mut rng), rect.min.y + offset + jitter_for(&mut rng));
                ui.painter().circle_filled(pos, radius, color);
            }
            for _ in 0..count {
                let radius = radius_for(&mut rng);
                let x = egui::lerp(rect.min.x..=rect.max.x, rng.next_f32());
                let pos = egui::pos2(x + jitter_for(&mut rng), rect.max.y - offset + jitter_for(&mut rng));
                ui.painter().circle_filled(pos, radius, color);
            }
            for _ in 0..count {
                let radius = radius_for(&mut rng);
                let y = egui::lerp(rect.min.y..=rect.max.y, rng.next_f32());
                let pos = egui::pos2(rect.min.x + offset + jitter_for(&mut rng), y + jitter_for(&mut rng));
                ui.painter().circle_filled(pos, radius, color);
            }
            for _ in 0..count {
                let radius = radius_for(&mut rng);
                let y = egui::lerp(rect.min.y..=rect.max.y, rng.next_f32());
                let pos = egui::pos2(rect.max.x - offset + jitter_for(&mut rng), y + jitter_for(&mut rng));
                ui.painter().circle_filled(pos, radius, color);
            }
        }
        CrosshairStyle::BorderFrame => {
            let thickness = crosshair.thickness;
            let offset = crosshair.offset;

            let top_y = rect.min.y + offset;
            let bottom_y = rect.max.y - offset;
            let left_x = rect.min.x + offset;
            let right_x = rect.max.x - offset;

            match crosshair.border_frame_style {
                BorderFrameStyle::Solid => {
                    draw_solid_border_frame(ui, rect, top_y, bottom_y, left_x, right_x, thickness, color);
                }
                BorderFrameStyle::Gap => {
                    draw_gap_border_frame(ui, rect, top_y, bottom_y, left_x, right_x, thickness, color);
                }
            }
        }
    }
}

/// 绘制虚线圆：按线段长度与间隔将圆周分段。
fn draw_dashed_circle(
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
        ui.painter().line_segment([start, end], Stroke::new(thickness, color));
    }
}

/// 简单的线性同余 RNG，用于预览/覆盖层生成稳定随机球位置。
#[derive(Debug, Clone, Copy)]
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
    }
}

/// 绘制完整边框（4 条矩形条）。
fn draw_solid_border_frame(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    top_y: f32,
    bottom_y: f32,
    left_x: f32,
    right_x: f32,
    thickness: f32,
    color: Color32,
) {
    ui.painter().rect_filled(
        egui::Rect::from_min_max(egui::pos2(rect.min.x, top_y - thickness / 2.0), egui::pos2(rect.max.x, top_y + thickness / 2.0)),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(egui::pos2(rect.min.x, bottom_y - thickness / 2.0), egui::pos2(rect.max.x, bottom_y + thickness / 2.0)),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(egui::pos2(left_x - thickness / 2.0, rect.min.y), egui::pos2(left_x + thickness / 2.0, rect.max.y)),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(egui::pos2(right_x - thickness / 2.0, rect.min.y), egui::pos2(right_x + thickness / 2.0, rect.max.y)),
        egui::CornerRadius::ZERO,
        color,
    );
}

/// 绘制带中间缺口的边框。
fn draw_gap_border_frame(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    top_y: f32,
    bottom_y: f32,
    left_x: f32,
    right_x: f32,
    thickness: f32,
    color: Color32,
) {
    let gap_pct = 0.2;
    let half_gap_w = rect.width() * gap_pct / 2.0;
    let half_gap_h = rect.height() * gap_pct / 2.0;

    // 上边。
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.min.x, top_y - thickness / 2.0),
            egui::pos2(rect.center().x - half_gap_w, top_y + thickness / 2.0),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.center().x + half_gap_w, top_y - thickness / 2.0),
            egui::pos2(rect.max.x, top_y + thickness / 2.0),
        ),
        egui::CornerRadius::ZERO,
        color,
    );

    // 下边。
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.min.x, bottom_y - thickness / 2.0),
            egui::pos2(rect.center().x - half_gap_w, bottom_y + thickness / 2.0),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.center().x + half_gap_w, bottom_y - thickness / 2.0),
            egui::pos2(rect.max.x, bottom_y + thickness / 2.0),
        ),
        egui::CornerRadius::ZERO,
        color,
    );

    // 左边。
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(left_x - thickness / 2.0, rect.min.y),
            egui::pos2(left_x + thickness / 2.0, rect.center().y - half_gap_h),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(left_x - thickness / 2.0, rect.center().y + half_gap_h),
            egui::pos2(left_x + thickness / 2.0, rect.max.y),
        ),
        egui::CornerRadius::ZERO,
        color,
    );

    // 右边。
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(right_x - thickness / 2.0, rect.min.y),
            egui::pos2(right_x + thickness / 2.0, rect.center().y - half_gap_h),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(right_x - thickness / 2.0, rect.center().y + half_gap_h),
            egui::pos2(right_x + thickness / 2.0, rect.max.y),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
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
