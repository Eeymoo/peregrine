export type CrosshairStyle =
  | "edge_rect"
  | "cross"
  | "large_cross"
  | "corner_dots4"
  | "corner_dots6"
  | "corner_dots8"
  | "ring"
  | "custom_orb"
  | "random_orb"
  | "border_frame"
  | "custom_image"
  | "edge_arrows"
  | "grid";

export type Anchor = "top" | "bottom" | "left" | "right" | "center";

export type RingStyle = "solid" | "dashed" | "double";

export type BorderFrameStyle = "solid" | "gap";

export type RandomOrbMode = "lock_on_start" | "reshuffle";

export type GridAlignment = "center" | "edge";

export type HotkeyAction =
  | "toggle_overlay"
  | "start_overlay"
  | "stop_overlay"
  | "cycle_color_next"
  | "cycle_color_prev"
  | "set_color_1"
  | "set_color_2"
  | "set_color_3"
  | "set_color_4"
  | "set_color_5";

export type HotkeyBindings = [HotkeyAction, string][];

export interface Crosshair {
  style: CrosshairStyle;
  size: number;
  secondary_size: number;
  thickness: number;
  radius: number;
  offset: number;
  color: [number, number, number, number];
  opacity: number;
  gap: number;
  corner_radius: number;
  anchor: Anchor;
  margin: number;
  ring_radius_pct: number;
  ring_style: RingStyle;
  orb_positions: number;
  random_mode: RandomOrbMode;
  random_center_deviation: number;
  random_radius_min: number;
  random_radius_max: number;
  random_orb_x: number;
  random_orb_y: number;
  border_frame_style: BorderFrameStyle;
  border_inset: boolean;
  custom_orb_top_count: number;
  custom_orb_bottom_count: number;
  custom_orb_left_count: number;
  custom_orb_right_count: number;
  random_orb_count: number;
  random_orb_offset: number;
  random_orb_jitter: number;
  image_path: string;
  image_scale: number;
  image_offset_x: number;
  image_offset_y: number;
  arrow_distance: number;
  arrow_width: number;
  arrow_tail_per_edge: boolean;
  arrow_tail_top: number;
  arrow_tail_bottom: number;
  arrow_tail_left: number;
  arrow_tail_right: number;
  grid_size: number;
  grid_alignment: GridAlignment;
}

export interface TriggerRule {
  enabled: boolean;
  process_names: string[];
}

export interface Profile {
  crosshair: Crosshair;
  trigger: TriggerRule;
  settings_hotkey: string;
  target_window: string;
}

export interface AppConfig {
  active_profile: string;
  profiles: Record<string, Profile>;
  settings: AppSettings;
}

export interface AppSettings {
  auto_switch_on_overlay: string;
  locale: string;
  fullscreen_overlay: boolean;
  live_drag_preview: boolean;
  gpu_acceleration: boolean;
  update_channel: string;
  cn_mirror: boolean;
  mirror_url: string;
  antialiasing: boolean;
  renderer_backend: "cpu" | "svg";
  quick_colors: [number, number, number, number][];
  hotkey_bindings: HotkeyBindings;
}
