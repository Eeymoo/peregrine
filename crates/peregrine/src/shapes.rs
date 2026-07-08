//! 准心几何计算共享模块。
//!
//! 预览（egui）与覆盖层（softbuffer）使用同一套几何公式，
//! 确保用户所见即所得：预览中准心的大小、位置与最终 overlay 完全一致。
//!
//! 使用方式：
//! 1. 调用 [`build_shapes`] 得到一组 [`Shape`]（矩形、圆、圆环等）；
//! 2. 预览用 egui painter 绘制；overlay 用 CPU 像素光栅化绘制。
//! 3. 两者输入相同（`RectF` + `Crosshair`），输出形状完全一致。

use peregrine_config::{
    Anchor, BorderFrameStyle, Crosshair, CrosshairStyle, OrbPosition, RingStyle,
};

// ===== 对外类型 =====

/// 逻辑坐标矩形区域。
#[derive(Debug, Clone, Copy)]
pub struct RectF {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl RectF {
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }
    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }
    pub fn center_x(&self) -> f32 {
        (self.min_x + self.max_x) / 2.0
    }
    pub fn center_y(&self) -> f32 {
        (self.min_y + self.max_y) / 2.0
    }
}

/// 一条带颜色的几何图元，预览与覆盖层共用。
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    /// 填充矩形（逻辑坐标）。
    Rect { x: f32, y: f32, w: f32, h: f32 },
    /// 填充圆（逻辑坐标）。
    Circle { cx: f32, cy: f32, radius: f32 },
    /// 圆环描边。
    CircleStroke {
        cx: f32,
        cy: f32,
        radius: f32,
        thickness: f32,
    },
    /// 虚线圆环。
    DashedCircle {
        cx: f32,
        cy: f32,
        radius: f32,
        thickness: f32,
        dash_len: f32,
        gap_len: f32,
    },
    /// 填充三角形（3 个顶点，逻辑坐标）。
    Triangle {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    },
}

/// 根据准心配置与绘制区域，生成一组图元。
///
/// `screen` 是准心绘制区域的逻辑矩形（覆盖层为全屏，预览为预览区）。
/// 返回的图元与 `crosshair` 参数完全对应，预览与覆盖层调用方式相同。
pub fn build_shapes(screen: &RectF, crosshair: &Crosshair) -> Vec<Shape> {
    let cx = screen.center_x();
    let cy = screen.center_y();
    let mut shapes = Vec::new();

    match crosshair.style {
        CrosshairStyle::Cross => {
            let arm = crosshair.size;
            let half_gap = crosshair.gap / 2.0;
            let thickness = crosshair.thickness;
            // 四条臂（以中心为原点对称展开，间距两侧均等）。
            shapes.push(Shape::Rect {
                x: cx - arm,
                y: cy - thickness / 2.0,
                w: arm - half_gap,
                h: thickness,
            });
            shapes.push(Shape::Rect {
                x: cx + half_gap,
                y: cy - thickness / 2.0,
                w: arm - half_gap,
                h: thickness,
            });
            shapes.push(Shape::Rect {
                x: cx - thickness / 2.0,
                y: cy - arm,
                w: thickness,
                h: arm - half_gap,
            });
            shapes.push(Shape::Rect {
                x: cx - thickness / 2.0,
                y: cy + half_gap,
                w: thickness,
                h: arm - half_gap,
            });
        }
        CrosshairStyle::LargeCross => {
            let thickness = crosshair.thickness;
            shapes.push(Shape::Rect {
                x: screen.min_x,
                y: cy - thickness / 2.0,
                w: screen.width(),
                h: thickness,
            });
            shapes.push(Shape::Rect {
                x: cx - thickness / 2.0,
                y: screen.min_y,
                w: thickness,
                h: screen.height(),
            });
        }
        CrosshairStyle::EdgeRect => {
            let w = crosshair.size;
            let h = crosshair.secondary_size;
            let margin = crosshair.margin;
            let (px, py) = match crosshair.anchor {
                Anchor::Top => (cx, screen.min_y + h / 2.0 + margin),
                Anchor::Bottom => (cx, screen.max_y - h / 2.0 - margin),
                Anchor::Left => (screen.min_x + w / 2.0 + margin, cy),
                Anchor::Right => (screen.max_x - w / 2.0 - margin, cy),
                Anchor::Center => (cx, cy),
            };
            shapes.push(Shape::Rect {
                x: px - w / 2.0,
                y: py - h / 2.0,
                w,
                h,
            });
        }
        CrosshairStyle::CornerDots4 | CrosshairStyle::CornerDots6 | CrosshairStyle::CornerDots8 => {
            let configured_offset = if crosshair.offset > 0.0 {
                crosshair.offset
            } else {
                crosshair.size
            };
            let offset = configured_offset
                .min(screen.width() / 4.0)
                .min(screen.height() / 4.0);
            let radius = if crosshair.radius > 0.0 {
                crosshair.radius
            } else {
                crosshair.thickness * 3.0
            };
            let corners = [
                (screen.min_x + offset, screen.min_y + offset),
                (screen.max_x - offset, screen.min_y + offset),
                (screen.min_x + offset, screen.max_y - offset),
                (screen.max_x - offset, screen.max_y - offset),
            ];
            for (x, y) in corners {
                shapes.push(Shape::Circle {
                    cx: x,
                    cy: y,
                    radius,
                });
            }
            if matches!(
                crosshair.style,
                CrosshairStyle::CornerDots6 | CrosshairStyle::CornerDots8
            ) {
                shapes.push(Shape::Circle {
                    cx,
                    cy: screen.min_y + offset,
                    radius,
                });
                shapes.push(Shape::Circle {
                    cx,
                    cy: screen.max_y - offset,
                    radius,
                });
            }
            if matches!(crosshair.style, CrosshairStyle::CornerDots8) {
                shapes.push(Shape::Circle {
                    cx: screen.min_x + offset,
                    cy,
                    radius,
                });
                shapes.push(Shape::Circle {
                    cx: screen.max_x - offset,
                    cy,
                    radius,
                });
            }
        }
        CrosshairStyle::Ring => {
            let radius = screen.height() * crosshair.ring_radius_pct;
            let thickness = crosshair.thickness;
            match crosshair.ring_style {
                RingStyle::Solid => {
                    shapes.push(Shape::CircleStroke {
                        cx,
                        cy,
                        radius,
                        thickness,
                    });
                }
                RingStyle::Dashed => {
                    shapes.push(Shape::DashedCircle {
                        cx,
                        cy,
                        radius,
                        thickness,
                        dash_len: 4.0,
                        gap_len: 4.0,
                    });
                }
                RingStyle::Double => {
                    shapes.push(Shape::CircleStroke {
                        cx,
                        cy,
                        radius: radius - 2.0,
                        thickness: 1.0,
                    });
                    shapes.push(Shape::DashedCircle {
                        cx,
                        cy,
                        radius: radius + 2.0,
                        thickness: 1.0,
                        dash_len: 4.0,
                        gap_len: 4.0,
                    });
                }
            }
        }
        CrosshairStyle::CustomOrb => {
            let radius = crosshair.radius.max(1.0);
            let offset = crosshair.offset;
            if crosshair.orb_positions.contains(OrbPosition::TOP) {
                edge_orb_shapes(
                    &mut shapes,
                    screen.min_x,
                    screen.min_y + offset,
                    screen.max_x,
                    screen.min_y + offset,
                    crosshair.custom_orb_top_count,
                    radius,
                );
            }
            if crosshair.orb_positions.contains(OrbPosition::BOTTOM) {
                edge_orb_shapes(
                    &mut shapes,
                    screen.min_x,
                    screen.max_y - offset,
                    screen.max_x,
                    screen.max_y - offset,
                    crosshair.custom_orb_bottom_count,
                    radius,
                );
            }
            if crosshair.orb_positions.contains(OrbPosition::LEFT) {
                edge_orb_shapes(
                    &mut shapes,
                    screen.min_x + offset,
                    screen.min_y,
                    screen.min_x + offset,
                    screen.max_y,
                    crosshair.custom_orb_left_count,
                    radius,
                );
            }
            if crosshair.orb_positions.contains(OrbPosition::RIGHT) {
                edge_orb_shapes(
                    &mut shapes,
                    screen.max_x - offset,
                    screen.min_y,
                    screen.max_x - offset,
                    screen.max_y,
                    crosshair.custom_orb_right_count,
                    radius,
                );
            }
        }
        CrosshairStyle::RandomOrb => {
            let seed = ((crosshair.random_orb_offset * 1000.0) as u64)
                .wrapping_add((crosshair.random_orb_jitter * 100.0) as u64)
                .wrapping_add((crosshair.random_radius_min * 10.0) as u64)
                .wrapping_add((crosshair.random_radius_max * 10.0) as u64)
                .wrapping_add(crosshair.random_orb_count as u64);
            let mut rng = SimpleRng::new(seed);
            let count = crosshair.random_orb_count as usize;
            let offset = crosshair.random_orb_offset;
            let jitter = crosshair.random_orb_jitter;
            let min_r = crosshair.random_radius_min;
            let max_r = crosshair.random_radius_max;

            for edge in 0..4 {
                for _ in 0..count {
                    let radius = min_r + rng.next_f32() * (max_r - min_r);
                    let j = (rng.next_f32() - 0.5) * 2.0 * jitter;
                    let (x, y) = match edge {
                        0 => {
                            let x = screen.min_x + rng.next_f32() * screen.width();
                            (x + j, screen.min_y + offset + j)
                        }
                        1 => {
                            let x = screen.min_x + rng.next_f32() * screen.width();
                            (x + j, screen.max_y - offset + j)
                        }
                        2 => {
                            let y = screen.min_y + rng.next_f32() * screen.height();
                            (screen.min_x + offset + j, y + j)
                        }
                        _ => {
                            let y = screen.min_y + rng.next_f32() * screen.height();
                            (screen.max_x - offset + j, y + j)
                        }
                    };
                    shapes.push(Shape::Circle {
                        cx: x,
                        cy: y,
                        radius,
                    });
                }
            }
        }
        CrosshairStyle::BorderFrame => {
            let thickness = crosshair.thickness;
            let offset = crosshair.offset;
            let top_y = screen.min_y + offset;
            let bottom_y = screen.max_y - offset;
            let left_x = screen.min_x + offset;
            let right_x = screen.max_x - offset;
            match crosshair.border_frame_style {
                BorderFrameStyle::Solid => {
                    solid_frame_shapes(
                        &mut shapes,
                        screen,
                        top_y,
                        bottom_y,
                        left_x,
                        right_x,
                        thickness,
                    );
                }
                BorderFrameStyle::Gap => {
                    gap_frame_shapes(
                        &mut shapes,
                        screen,
                        top_y,
                        bottom_y,
                        left_x,
                        right_x,
                        thickness,
                    );
                }
            }
        }
        // 箭头：四边各一个三角形 + 尾巴矩形，从边缘向中心 ===>。
        // 所有尺寸按屏幕短边等比缩放，确保预览与实际 overlay 比例一致。
        CrosshairStyle::EdgeArrows => {
            // 基准短边像素（1080p 短边），所有配置值以此为参照。
            const BASE_SHORT: f32 = 1080.0;
            let screen_short = screen.width().min(screen.height());
            let scale = screen_short / BASE_SHORT;

            let tri_size = (crosshair.size.max(4.0)) * scale;
            let tri_half = tri_size * 0.6;
            // 每边的尾巴长度。
            let default_tail = crosshair.arrow_distance.max(0.0) * scale;
            let (tail_top, tail_bottom, tail_left, tail_right) = if crosshair.arrow_tail_per_edge {
                (
                    crosshair.arrow_tail_top.max(0.0) * scale,
                    crosshair.arrow_tail_bottom.max(0.0) * scale,
                    crosshair.arrow_tail_left.max(0.0) * scale,
                    crosshair.arrow_tail_right.max(0.0) * scale,
                )
            } else {
                (default_tail, default_tail, default_tail, default_tail)
            };
            let tail_half = if crosshair.arrow_width > 0.0 {
                ((crosshair.arrow_width / 2.0) * scale).min(tri_half)
            } else {
                tri_half
            };

            let pos = crosshair.orb_positions;
            let all_off = !pos.contains(OrbPosition::TOP)
                && !pos.contains(OrbPosition::BOTTOM)
                && !pos.contains(OrbPosition::LEFT)
                && !pos.contains(OrbPosition::RIGHT);
            let (show_top, show_bottom, show_left, show_right) = if all_off {
                (true, true, true, true)
            } else {
                (
                    pos.contains(OrbPosition::TOP),
                    pos.contains(OrbPosition::BOTTOM),
                    pos.contains(OrbPosition::LEFT),
                    pos.contains(OrbPosition::RIGHT),
                )
            };

            if show_top {
                let tri_base_y = screen.min_y + tail_top;
                let tri_tip_y = tri_base_y + tri_size;
                if tail_top > 0.0 {
                    shapes.push(Shape::Rect {
                        x: cx - tail_half,
                        y: screen.min_y,
                        w: tail_half * 2.0,
                        h: tail_top,
                    });
                }
                shapes.push(Shape::Triangle {
                    x1: cx,
                    y1: tri_tip_y,
                    x2: cx - tri_half,
                    y2: tri_base_y,
                    x3: cx + tri_half,
                    y3: tri_base_y,
                });
            }
            if show_bottom {
                let tri_base_y = screen.max_y - tail_bottom;
                let tri_tip_y = tri_base_y - tri_size;
                if tail_bottom > 0.0 {
                    shapes.push(Shape::Rect {
                        x: cx - tail_half,
                        y: tri_base_y,
                        w: tail_half * 2.0,
                        h: tail_bottom,
                    });
                }
                shapes.push(Shape::Triangle {
                    x1: cx,
                    y1: tri_tip_y,
                    x2: cx - tri_half,
                    y2: tri_base_y,
                    x3: cx + tri_half,
                    y3: tri_base_y,
                });
            }
            if show_left {
                let tri_base_x = screen.min_x + tail_left;
                let tri_tip_x = tri_base_x + tri_size;
                if tail_left > 0.0 {
                    shapes.push(Shape::Rect {
                        x: screen.min_x,
                        y: cy - tail_half,
                        w: tail_left,
                        h: tail_half * 2.0,
                    });
                }
                shapes.push(Shape::Triangle {
                    x1: tri_tip_x,
                    y1: cy,
                    x2: tri_base_x,
                    y2: cy - tri_half,
                    x3: tri_base_x,
                    y3: cy + tri_half,
                });
            }
            if show_right {
                let tri_base_x = screen.max_x - tail_right;
                let tri_tip_x = tri_base_x - tri_size;
                if tail_right > 0.0 {
                    shapes.push(Shape::Rect {
                        x: tri_base_x,
                        y: cy - tail_half,
                        w: tail_right,
                        h: tail_half * 2.0,
                    });
                }
                shapes.push(Shape::Triangle {
                    x1: tri_tip_x,
                    y1: cy,
                    x2: tri_base_x,
                    y2: cy - tri_half,
                    x3: tri_base_x,
                    y3: cy + tri_half,
                });
            }
        }
        // CustomImage 不生成矢量图元，由各渲染器单独处理图片加载与 blit。
        CrosshairStyle::CustomImage => {}
    }

    shapes
}

/// 在一条边上均匀分布绘制圆点（与 overlay_renderer / settings_ui 的逻辑一致）。
fn edge_orb_shapes(
    shapes: &mut Vec<Shape>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    count: u32,
    radius: f32,
) {
    if count == 0 {
        return;
    }
    if count == 1 {
        let mx = (x0 + x1) / 2.0;
        let my = (y0 + y1) / 2.0;
        shapes.push(Shape::Circle {
            cx: mx,
            cy: my,
            radius,
        });
        return;
    }
    let step = 1.0 / (count + 1) as f32;
    for i in 1..=count {
        let t = i as f32 * step;
        let x = x0 + (x1 - x0) * t;
        let y = y0 + (y1 - y0) * t;
        shapes.push(Shape::Circle {
            cx: x,
            cy: y,
            radius,
        });
    }
}

/// 完整边框（4 条矩形条）。
fn solid_frame_shapes(
    shapes: &mut Vec<Shape>,
    rect: &RectF,
    top_y: f32,
    bottom_y: f32,
    left_x: f32,
    right_x: f32,
    thickness: f32,
) {
    let half_t = thickness / 2.0;
    shapes.push(Shape::Rect {
        x: rect.min_x,
        y: top_y - half_t,
        w: rect.width(),
        h: thickness,
    });
    shapes.push(Shape::Rect {
        x: rect.min_x,
        y: bottom_y - half_t,
        w: rect.width(),
        h: thickness,
    });
    shapes.push(Shape::Rect {
        x: left_x - half_t,
        y: rect.min_y,
        w: thickness,
        h: rect.height(),
    });
    shapes.push(Shape::Rect {
        x: right_x - half_t,
        y: rect.min_y,
        w: thickness,
        h: rect.height(),
    });
}

/// 带中间缺口的边框。
fn gap_frame_shapes(
    shapes: &mut Vec<Shape>,
    rect: &RectF,
    top_y: f32,
    bottom_y: f32,
    left_x: f32,
    right_x: f32,
    thickness: f32,
) {
    let half_t = thickness / 2.0;
    let half_gap_w = rect.width() * 0.2 / 2.0;
    let half_gap_h = rect.height() * 0.2 / 2.0;
    let cx = rect.center_x();
    let cy = rect.center_y();

    // 上边（两段）。
    shapes.push(Shape::Rect {
        x: rect.min_x,
        y: top_y - half_t,
        w: cx - half_gap_w - rect.min_x,
        h: thickness,
    });
    shapes.push(Shape::Rect {
        x: cx + half_gap_w,
        y: top_y - half_t,
        w: rect.max_x - (cx + half_gap_w),
        h: thickness,
    });
    // 下边。
    shapes.push(Shape::Rect {
        x: rect.min_x,
        y: bottom_y - half_t,
        w: cx - half_gap_w - rect.min_x,
        h: thickness,
    });
    shapes.push(Shape::Rect {
        x: cx + half_gap_w,
        y: bottom_y - half_t,
        w: rect.max_x - (cx + half_gap_w),
        h: thickness,
    });
    // 左边。
    shapes.push(Shape::Rect {
        x: left_x - half_t,
        y: rect.min_y,
        w: thickness,
        h: cy - half_gap_h - rect.min_y,
    });
    shapes.push(Shape::Rect {
        x: left_x - half_t,
        y: cy + half_gap_h,
        w: thickness,
        h: rect.max_y - (cy + half_gap_h),
    });
    // 右边。
    shapes.push(Shape::Rect {
        x: right_x - half_t,
        y: rect.min_y,
        w: thickness,
        h: cy - half_gap_h - rect.min_y,
    });
    shapes.push(Shape::Rect {
        x: right_x - half_t,
        y: cy + half_gap_h,
        w: thickness,
        h: rect.max_y - (cy + half_gap_h),
    });
}

/// 简单的线性同余 RNG，用于预览/覆盖层生成稳定随机球位置。
///
/// 预览与 overlay 必须使用相同的 RNG 实现与种子，才能生成相同的随机球布局。
#[derive(Debug, Clone, Copy)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
    }
}
