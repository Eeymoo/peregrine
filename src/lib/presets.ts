import type { Crosshair, CrosshairStyle } from "@/types/config";

/**
 * 为指定样式生成一套开箱即用的默认准心参数。
 *
 * 切换样式时调用此函数重置参数，避免旧样式的尺寸/偏移在新样式下不可用。
 * 该函数是 Rust 侧 `Crosshair::default_for_style` 的前端镜像，默认值必须保持一致。
 */
export function getDefaultCrosshairForStyle(style: CrosshairStyle): Crosshair {
  const base: Crosshair = {
    style,
    size: 16,
    secondary_size: 80,
    thickness: 2,
    radius: 0,
    offset: 0,
    color: [1, 1, 1, 1],
    opacity: 0.6,
    gap: 4,
    corner_radius: 4,
    anchor: "top",
    margin: 0,
    ring_radius_pct: 0.05,
    ring_style: "solid",
    orb_positions: 0b0011,
    random_mode: "lock_on_start",
    random_center_deviation: 0.2,
    random_radius_min: 4,
    random_radius_max: 12,
    random_orb_x: 0,
    random_orb_y: 0,
    border_frame_style: "solid",
    border_inset: true,
    custom_orb_top_count: 3,
    custom_orb_bottom_count: 3,
    custom_orb_left_count: 3,
    custom_orb_right_count: 3,
    random_orb_count: 3,
    random_orb_offset: 100,
    random_orb_jitter: 40,
    image_path: "",
    image_scale: 1,
    image_offset_x: 0,
    image_offset_y: 0,
    arrow_distance: 0,
    arrow_width: 0,
    arrow_tail_per_edge: false,
    arrow_tail_top: 0,
    arrow_tail_bottom: 0,
    arrow_tail_left: 0,
    arrow_tail_right: 0,
    grid_size: 80,
    grid_alignment: "center",
  };

  switch (style) {
    case "edge_rect":
      base.size = 180;
      base.secondary_size = 24;
      base.thickness = 4;
      base.anchor = "top";
      base.margin = 16;
      base.corner_radius = 12;
      break;
    case "cross":
      base.size = 24;
      base.thickness = 2;
      base.gap = 4;
      base.opacity = 0.8;
      break;
    case "large_cross":
      base.thickness = 2;
      base.opacity = 0.5;
      break;
    case "corner_dots4":
    case "corner_dots6":
    case "corner_dots8":
      base.offset = 40;
      base.thickness = 3;
      base.radius = 0;
      base.opacity = 0.7;
      break;
    case "ring":
      base.thickness = 2;
      base.ring_radius_pct = 0.06;
      base.ring_style = "solid";
      base.opacity = 0.8;
      break;
    case "custom_orb":
      base.radius = 6;
      base.offset = 30;
      base.orb_positions = 0b0011;
      base.custom_orb_top_count = 3;
      base.custom_orb_bottom_count = 3;
      base.custom_orb_left_count = 3;
      base.custom_orb_right_count = 3;
      base.opacity = 0.7;
      break;
    case "random_orb":
      base.random_orb_count = 3;
      base.random_orb_offset = 80;
      base.random_orb_jitter = 30;
      base.random_radius_min = 4;
      base.random_radius_max = 10;
      base.opacity = 0.6;
      break;
    case "border_frame":
      base.thickness = 6;
      base.offset = 24;
      base.border_frame_style = "solid";
      base.border_inset = false;
      base.opacity = 0.5;
      break;
    case "edge_arrows":
      base.size = 16;
      base.arrow_distance = 60;
      base.arrow_width = 0;
      base.arrow_tail_per_edge = false;
      base.orb_positions = 0;
      base.opacity = 0.7;
      break;
    case "grid":
      base.grid_size = 120;
      base.thickness = 2;
      base.grid_alignment = "center";
      base.opacity = 0.35;
      break;
    case "custom_image":
      base.size = 64;
      base.image_scale = 1;
      base.image_offset_x = 0;
      base.image_offset_y = 0;
      base.opacity = 0.9;
      break;
  }

  return base;
}
