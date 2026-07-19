//! 旧 `Crosshair` 配置到新 `Layer` 配置的迁移逻辑。
//!
//! 当 `load_or_create_default` 检测到旧格式配置（含 `crosshair` 字段且无 `layers`）
//! 时自动调用本模块，把 `Crosshair.style` 映射为对应内置物料的 `Layer` 实例，
//! 字段一对一搬运到 `params`。视觉效果与旧版本完全一致。

use crate::schema::{
    Anchor, BorderFrameStyle, Crosshair, CrosshairStyle, GridAlignment, Layer, LayerStyle,
    MaterialRef, RandomOrbMode, RingStyle, Transform2D,
};
use serde_json::{Map, Value, json};

/// 把旧 `Crosshair` 转换为单个 `Layer`（引用对应的内置物料）。
///
/// 字段映射规则见 `openspec/changes/four-layer-customization/specs/profile-migration/spec.md`。
pub fn migrate_crosshair_to_layer(crosshair: &Crosshair) -> Layer {
    let (material, params) = match crosshair.style {
        CrosshairStyle::EdgeRect | CrosshairStyle::CustomImage => {
            unreachable!("edge_rect 与 custom_image 由专用分支处理")
        }
        CrosshairStyle::Cross => migrate_cross(crosshair),
        CrosshairStyle::LargeCross => migrate_large_cross(crosshair),
        CrosshairStyle::CornerDots4 => migrate_corner_dots(crosshair, 4),
        CrosshairStyle::CornerDots6 => migrate_corner_dots(crosshair, 6),
        CrosshairStyle::CornerDots8 => migrate_corner_dots(crosshair, 8),
        CrosshairStyle::Ring => migrate_ring(crosshair),
        CrosshairStyle::CustomOrb => migrate_custom_orb(crosshair),
        CrosshairStyle::RandomOrb => migrate_random_orb(crosshair),
        CrosshairStyle::BorderFrame => migrate_border_frame(crosshair),
        CrosshairStyle::EdgeArrows => migrate_edge_arrows(crosshair),
        CrosshairStyle::Grid => migrate_grid(crosshair),
    };

    let mut layer = Layer::new(
        format!("migrated_{}", style_id(&crosshair.style)),
        style_display_name(&crosshair.style),
        material,
    );
    layer.params = Value::Object(params);
    layer.style = LayerStyle {
        color: crosshair.color,
        opacity: crosshair.opacity,
        ..LayerStyle::default()
    };
    layer.transform = Transform2D::default();
    layer
}

/// 把旧 `EdgeRect` 配置转换为 `builtin.edge_rect` 物料的 layer。
pub fn migrate_edge_rect(crosshair: &Crosshair) -> Layer {
    let mut params = Map::new();
    params.insert("size".into(), json!(crosshair.size));
    params.insert("secondary_size".into(), json!(crosshair.secondary_size));
    params.insert("anchor".into(), json!(anchor_to_str(crosshair.anchor)));
    params.insert("margin".into(), json!(crosshair.margin));
    params.insert("corner_radius".into(), json!(crosshair.corner_radius));

    build_layer(
        "edge_rect",
        "贴边矩形",
        MaterialRef::Builtin {
            id: "builtin.edge_rect".into(),
        },
        params,
        crosshair,
    )
}

/// 把旧 `CustomImage` 配置转换为 `builtin.image` 物料的 layer。
pub fn migrate_custom_image(crosshair: &Crosshair) -> Layer {
    let mut params = Map::new();
    params.insert("path".into(), json!(crosshair.image_path));
    params.insert("scale".into(), json!(crosshair.image_scale));
    params.insert("offset_x".into(), json!(crosshair.image_offset_x));
    params.insert("offset_y".into(), json!(crosshair.image_offset_y));
    params.insert("width".into(), json!(crosshair.size));
    params.insert("height".into(), json!(crosshair.size));

    build_layer(
        "image",
        "自定义图片",
        MaterialRef::Builtin {
            id: "builtin.image".into(),
        },
        params,
        crosshair,
    )
}

/// 迁移整个 `Profile`：保留 trigger / hotkey / target_window，
/// 把 crosshair 转为 layers[0]，同时保留 crosshair 字段供旧版 UI 编辑。
pub fn migrate_profile(profile: &crate::schema::Profile) -> crate::schema::Profile {
    let crosshair = match &profile.crosshair {
        Some(c) => c,
        None => {
            // 没有 crosshair 字段（可能已是新格式），直接返回克隆。
            return profile.clone();
        }
    };

    let layer = match crosshair.style {
        CrosshairStyle::EdgeRect => migrate_edge_rect(crosshair),
        CrosshairStyle::CustomImage => migrate_custom_image(crosshair),
        _ => migrate_crosshair_to_layer(crosshair),
    };

    crate::schema::Profile {
        crosshair: Some(crosshair.clone()),
        layers: vec![layer],
        trigger: profile.trigger.clone(),
        settings_hotkey: profile.settings_hotkey.clone(),
        target_window: profile.target_window.clone(),
    }
}

// ===== 各样式专用迁移函数 =====

fn migrate_cross(c: &Crosshair) -> (MaterialRef, Map<String, Value>) {
    let mut p = Map::new();
    p.insert("size".into(), json!(c.size));
    p.insert("thickness".into(), json!(c.thickness));
    p.insert("gap".into(), json!(c.gap));
    (
        MaterialRef::Builtin {
            id: "builtin.cross".into(),
        },
        p,
    )
}

fn migrate_large_cross(c: &Crosshair) -> (MaterialRef, Map<String, Value>) {
    let mut p = Map::new();
    p.insert("thickness".into(), json!(c.thickness));
    (
        MaterialRef::Builtin {
            id: "builtin.large_cross".into(),
        },
        p,
    )
}

fn migrate_corner_dots(c: &Crosshair, count: i64) -> (MaterialRef, Map<String, Value>) {
    let mut p = Map::new();
    p.insert("count".into(), json!(count));
    p.insert("offset".into(), json!(c.offset));
    p.insert("thickness".into(), json!(c.thickness));
    p.insert("radius".into(), json!(c.radius));
    (
        MaterialRef::Builtin {
            id: "builtin.corner_dots".into(),
        },
        p,
    )
}

fn migrate_ring(c: &Crosshair) -> (MaterialRef, Map<String, Value>) {
    let mut p = Map::new();
    p.insert("thickness".into(), json!(c.thickness));
    p.insert("ring_radius_pct".into(), json!(c.ring_radius_pct));
    p.insert("ring_style".into(), json!(ring_style_to_str(c.ring_style)));
    (
        MaterialRef::Builtin {
            id: "builtin.ring".into(),
        },
        p,
    )
}

fn migrate_custom_orb(c: &Crosshair) -> (MaterialRef, Map<String, Value>) {
    let mut p = Map::new();
    p.insert("radius".into(), json!(c.radius));
    p.insert("offset".into(), json!(c.offset));
    p.insert("orb_positions".into(), json!(c.orb_positions.0 as i64));
    p.insert("top_count".into(), json!(c.custom_orb_top_count));
    p.insert("bottom_count".into(), json!(c.custom_orb_bottom_count));
    p.insert("left_count".into(), json!(c.custom_orb_left_count));
    p.insert("right_count".into(), json!(c.custom_orb_right_count));
    (
        MaterialRef::Builtin {
            id: "builtin.custom_orb".into(),
        },
        p,
    )
}

fn migrate_random_orb(c: &Crosshair) -> (MaterialRef, Map<String, Value>) {
    let mut p = Map::new();
    p.insert("orb_count".into(), json!(c.random_orb_count));
    p.insert("offset".into(), json!(c.random_orb_offset));
    p.insert("jitter".into(), json!(c.random_orb_jitter));
    p.insert("radius_min".into(), json!(c.random_radius_min));
    p.insert("radius_max".into(), json!(c.random_radius_max));
    p.insert("center_deviation".into(), json!(c.random_center_deviation));
    p.insert("mode".into(), json!(random_mode_to_str(c.random_mode)));
    (
        MaterialRef::Builtin {
            id: "builtin.random_orb".into(),
        },
        p,
    )
}

fn migrate_border_frame(c: &Crosshair) -> (MaterialRef, Map<String, Value>) {
    let mut p = Map::new();
    p.insert("thickness".into(), json!(c.thickness));
    p.insert("offset".into(), json!(c.offset));
    p.insert(
        "frame_style".into(),
        json!(border_frame_style_to_str(c.border_frame_style)),
    );
    p.insert("inset".into(), json!(c.border_inset));
    (
        MaterialRef::Builtin {
            id: "builtin.border_frame".into(),
        },
        p,
    )
}

fn migrate_edge_arrows(c: &Crosshair) -> (MaterialRef, Map<String, Value>) {
    let mut p = Map::new();
    p.insert("size".into(), json!(c.size));
    p.insert("distance".into(), json!(c.arrow_distance));
    p.insert("width".into(), json!(c.arrow_width));
    p.insert("tail_per_edge".into(), json!(c.arrow_tail_per_edge));
    p.insert("tail_top".into(), json!(c.arrow_tail_top));
    p.insert("tail_bottom".into(), json!(c.arrow_tail_bottom));
    p.insert("tail_left".into(), json!(c.arrow_tail_left));
    p.insert("tail_right".into(), json!(c.arrow_tail_right));
    p.insert("positions_mask".into(), json!(c.orb_positions.0 as i64));
    (
        MaterialRef::Builtin {
            id: "builtin.edge_arrows".into(),
        },
        p,
    )
}

fn migrate_grid(c: &Crosshair) -> (MaterialRef, Map<String, Value>) {
    let mut p = Map::new();
    p.insert("grid_size".into(), json!(c.grid_size));
    p.insert("thickness".into(), json!(c.thickness));
    p.insert(
        "alignment".into(),
        json!(grid_alignment_to_str(c.grid_alignment)),
    );
    (
        MaterialRef::Builtin {
            id: "builtin.grid".into(),
        },
        p,
    )
}

// ===== 辅助函数 =====

fn build_layer(
    id_suffix: &str,
    name: &str,
    material: MaterialRef,
    params: Map<String, Value>,
    crosshair: &Crosshair,
) -> Layer {
    let mut layer = Layer::new(
        format!("migrated_{}", id_suffix),
        name.to_string(),
        material,
    );
    layer.params = Value::Object(params);
    layer.style = LayerStyle {
        color: crosshair.color,
        opacity: crosshair.opacity,
        ..LayerStyle::default()
    };
    layer
}

fn style_id(s: &CrosshairStyle) -> &'static str {
    match s {
        CrosshairStyle::Cross => "cross",
        CrosshairStyle::LargeCross => "large_cross",
        CrosshairStyle::CornerDots4 => "corner_dots4",
        CrosshairStyle::CornerDots6 => "corner_dots6",
        CrosshairStyle::CornerDots8 => "corner_dots8",
        CrosshairStyle::Ring => "ring",
        CrosshairStyle::CustomOrb => "custom_orb",
        CrosshairStyle::RandomOrb => "random_orb",
        CrosshairStyle::BorderFrame => "border_frame",
        CrosshairStyle::EdgeArrows => "edge_arrows",
        CrosshairStyle::Grid => "grid",
        CrosshairStyle::EdgeRect | CrosshairStyle::CustomImage => "edge_rect_or_image",
    }
}

fn style_display_name(s: &CrosshairStyle) -> &'static str {
    match s {
        CrosshairStyle::Cross => "准星",
        CrosshairStyle::LargeCross => "大准星",
        CrosshairStyle::CornerDots4 => "定位球4",
        CrosshairStyle::CornerDots6 => "定位球6",
        CrosshairStyle::CornerDots8 => "定位球8",
        CrosshairStyle::Ring => "中心环",
        CrosshairStyle::CustomOrb => "自定义定位球",
        CrosshairStyle::RandomOrb => "随机球",
        CrosshairStyle::BorderFrame => "边框",
        CrosshairStyle::EdgeArrows => "箭头",
        CrosshairStyle::Grid => "网格",
        CrosshairStyle::EdgeRect => "贴边矩形",
        CrosshairStyle::CustomImage => "自定义图片",
    }
}

fn anchor_to_str(a: Anchor) -> &'static str {
    match a {
        Anchor::Top => "top",
        Anchor::Bottom => "bottom",
        Anchor::Left => "left",
        Anchor::Right => "right",
        Anchor::Center => "center",
    }
}

fn ring_style_to_str(r: RingStyle) -> &'static str {
    match r {
        RingStyle::Solid => "solid",
        RingStyle::Dashed => "dashed",
        RingStyle::Double => "double",
    }
}

fn border_frame_style_to_str(b: BorderFrameStyle) -> &'static str {
    match b {
        BorderFrameStyle::Solid => "solid",
        BorderFrameStyle::Gap => "gap",
    }
}

fn random_mode_to_str(r: RandomOrbMode) -> &'static str {
    match r {
        RandomOrbMode::LockOnStart => "lock_on_start",
        RandomOrbMode::Reshuffle => "reshuffle",
    }
}

fn grid_alignment_to_str(g: GridAlignment) -> &'static str {
    match g {
        GridAlignment::Center => "center",
        GridAlignment::Edge => "edge",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Crosshair, CrosshairStyle, Profile};

    fn test_crosshair(style: CrosshairStyle) -> Crosshair {
        Crosshair::default_for_style(style)
    }

    fn assert_param(layer: &Layer, key: &str, expected: &Value) {
        let params = layer.params.as_object().expect("params is object");
        let actual = params
            .get(key)
            .unwrap_or_else(|| panic!("missing param '{}'", key));
        assert_eq!(
            actual, expected,
            "param '{}' expected {:?} got {:?}",
            key, expected, actual
        );
    }

    #[test]
    fn migrate_cross_style() {
        let c = test_crosshair(CrosshairStyle::Cross);
        let layer = migrate_crosshair_to_layer(&c);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.cross".into()
            }
        );
        assert_param(&layer, "size", &json!(24.0));
        assert_param(&layer, "thickness", &json!(2.0));
        assert_param(&layer, "gap", &json!(4.0));
    }

    #[test]
    fn migrate_edge_rect_style() {
        let c = test_crosshair(CrosshairStyle::EdgeRect);
        let layer = migrate_edge_rect(&c);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.edge_rect".into()
            }
        );
        assert_param(&layer, "size", &json!(180.0));
        assert_param(&layer, "secondary_size", &json!(24.0));
        assert_param(&layer, "anchor", &json!("top"));
        assert_param(&layer, "margin", &json!(16.0));
        assert_param(&layer, "corner_radius", &json!(12.0));
    }

    #[test]
    fn migrate_large_cross_style() {
        let c = test_crosshair(CrosshairStyle::LargeCross);
        let layer = migrate_crosshair_to_layer(&c);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.large_cross".into()
            }
        );
        assert_param(&layer, "thickness", &json!(2.0));
    }

    #[test]
    fn migrate_corner_dots_variants() {
        for (style, count) in [
            (CrosshairStyle::CornerDots4, 4),
            (CrosshairStyle::CornerDots6, 6),
            (CrosshairStyle::CornerDots8, 8),
        ] {
            let c = test_crosshair(style);
            let layer = migrate_crosshair_to_layer(&c);
            assert_eq!(
                layer.material,
                MaterialRef::Builtin {
                    id: "builtin.corner_dots".into()
                }
            );
            assert_param(&layer, "count", &json!(count));
        }
    }

    #[test]
    fn migrate_ring_style() {
        let c = test_crosshair(CrosshairStyle::Ring);
        let layer = migrate_crosshair_to_layer(&c);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.ring".into()
            }
        );
        assert_param(&layer, "ring_radius_pct", &json!(0.06_f32 as f64));
        assert_param(&layer, "ring_style", &json!("solid"));
    }

    #[test]
    fn migrate_custom_orb_style() {
        let c = test_crosshair(CrosshairStyle::CustomOrb);
        let layer = migrate_crosshair_to_layer(&c);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.custom_orb".into()
            }
        );
        assert_param(&layer, "radius", &json!(6.0));
        assert_param(&layer, "orb_positions", &json!(3));
        assert_param(&layer, "top_count", &json!(3));
    }

    #[test]
    fn migrate_random_orb_style() {
        let c = test_crosshair(CrosshairStyle::RandomOrb);
        let layer = migrate_crosshair_to_layer(&c);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.random_orb".into()
            }
        );
        assert_param(&layer, "orb_count", &json!(3));
        assert_param(&layer, "offset", &json!(80.0));
        assert_param(&layer, "radius_min", &json!(4.0));
        assert_param(&layer, "mode", &json!("lock_on_start"));
    }

    #[test]
    fn migrate_border_frame_style() {
        let c = test_crosshair(CrosshairStyle::BorderFrame);
        let layer = migrate_crosshair_to_layer(&c);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.border_frame".into()
            }
        );
        assert_param(&layer, "thickness", &json!(6.0));
        assert_param(&layer, "offset", &json!(24.0));
        assert_param(&layer, "frame_style", &json!("solid"));
        assert_param(&layer, "inset", &json!(false));
    }

    #[test]
    fn migrate_edge_arrows_style() {
        let c = test_crosshair(CrosshairStyle::EdgeArrows);
        let layer = migrate_crosshair_to_layer(&c);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.edge_arrows".into()
            }
        );
        assert_param(&layer, "size", &json!(16.0));
        assert_param(&layer, "distance", &json!(60.0));
    }

    #[test]
    fn migrate_grid_style() {
        let c = test_crosshair(CrosshairStyle::Grid);
        let layer = migrate_crosshair_to_layer(&c);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.grid".into()
            }
        );
        assert_param(&layer, "grid_size", &json!(120.0));
        assert_param(&layer, "thickness", &json!(2.0));
        assert_param(&layer, "alignment", &json!("center"));
    }

    #[test]
    fn migrate_custom_image_style() {
        let mut c = test_crosshair(CrosshairStyle::CustomImage);
        c.image_path = "/test/cross.png".to_string();
        c.image_scale = 1.5;
        let layer = migrate_custom_image(&c);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.image".into()
            }
        );
        assert_param(&layer, "path", &json!("/test/cross.png"));
        assert_param(&layer, "scale", &json!(1.5));
    }

    #[test]
    fn migrate_preserves_color_and_opacity() {
        let mut c = test_crosshair(CrosshairStyle::Cross);
        c.color = [1.0, 0.0, 0.0, 1.0]; // 红色
        c.opacity = 0.42;
        let layer = migrate_crosshair_to_layer(&c);
        assert_eq!(layer.style.color, [1.0, 0.0, 0.0, 1.0]);
        assert!((layer.style.opacity - 0.42).abs() < 1e-6);
    }

    #[test]
    fn migrate_profile_preserves_trigger_and_hotkey() {
        let mut profile = Profile::default_profile();
        profile.settings_hotkey = "F9".to_string();
        profile.target_window = "My Game".to_string();
        profile.trigger.enabled = false;
        profile.trigger.process_names = vec!["game.exe".to_string()];

        let migrated = migrate_profile(&profile);
        assert_eq!(migrated.layers.len(), 1);
        assert_eq!(migrated.settings_hotkey, "F9");
        assert_eq!(migrated.target_window, "My Game");
        assert!(!migrated.trigger.enabled);
        assert_eq!(migrated.trigger.process_names, vec!["game.exe".to_string()]);
    }

    #[test]
    fn migrate_all_styles_succeeds() {
        // 通过 migrate_crosshair_to_layer 处理的样式（不含 EdgeRect 和 CustomImage）。
        let styles = [
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
        ];
        for style in styles {
            let c = test_crosshair(style);
            let layer = migrate_crosshair_to_layer(&c);
            assert!(!layer.id.is_empty());
            assert!(!layer.name.is_empty());
        }

        // EdgeRect 与 CustomImage 由专用函数处理。
        let c_edge = test_crosshair(CrosshairStyle::EdgeRect);
        let layer = migrate_edge_rect(&c_edge);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.edge_rect".into()
            }
        );

        let mut c_img = test_crosshair(CrosshairStyle::CustomImage);
        c_img.image_path = "/x.png".to_string();
        let layer = migrate_custom_image(&c_img);
        assert_eq!(
            layer.material,
            MaterialRef::Builtin {
                id: "builtin.image".into()
            }
        );
    }
}
