// 四层架构类型定义：与 crates/config/src/schema.rs 保持同步。

/** 基础图元类型（与 Rust `Element` 对应）。 */
export type Element =
  | { type: "rect"; x: number; y: number; w: number; h: number }
  | { type: "circle"; cx: number; cy: number; radius: number }
  | {
      type: "circle_stroke";
      cx: number;
      cy: number;
      radius: number;
      thickness: number;
    }
  | {
      type: "dashed_circle";
      cx: number;
      cy: number;
      radius: number;
      thickness: number;
      dash_len: number;
      gap_len: number;
    }
  | {
      type: "triangle";
      x1: number;
      y1: number;
      x2: number;
      y2: number;
      x3: number;
      y3: number;
    }
  | { type: "polygon"; points: [number, number][] }
  | {
      type: "line";
      x1: number;
      y1: number;
      x2: number;
      y2: number;
      thickness: number;
    }
  | { type: "text"; x: number; y: number; content: string; font_size: number }
  | { type: "image"; path: string; x: number; y: number; w: number; h: number };

/** 物料引用：图层所用的物料来源。 */
export type MaterialRef =
  | { kind: "builtin"; id: string }
  | { kind: "user"; name: string };

/** 图层几何变换。 */
export interface Transform2D {
  offset_x: number;
  offset_y: number;
  scale: number;
  rotation_deg: number;
}

/** 混合模式（首期仅 Normal）。 */
export type BlendMode = "normal";

/** 图层级样式。 */
export interface LayerStyle {
  color: [number, number, number, number];
  opacity: number;
  blend_mode: BlendMode;
}

/** 单个图层。 */
export interface Layer {
  id: string;
  name: string;
  material: MaterialRef;
  /** 图层参数（JSON 对象，覆盖物料 defaults）。 */
  params: Record<string, unknown>;
  style: LayerStyle;
  transform: Transform2D;
  visible: boolean;
  locked: boolean;
}

/** 物料 schema 参数定义（由物料脚本 `schema()` 返回）。 */
export interface MaterialSchemaEntry {
  key: string;
  label: string;
  widget:
    | "number"
    | "slider"
    | "color"
    | "select"
    | "toggle"
    | "image_path"
    | "text";
  min?: number;
  max?: number;
  step?: number;
  options?: { value: string | number; label: string }[];
  default?: unknown;
}

/** 物料信息（IPC `list_materials` 返回）。 */
export interface MaterialInfo {
  id: string;
  display_name: string;
  builtin: boolean;
  is_dynamic: boolean;
  defaults: Record<string, unknown>;
  schema: MaterialSchemaEntry[];
}

/** IPC `build_shapes` 返回的图元（含图层颜色/不透明度）。 */
export interface BuiltShape {
  element: Element;
  color: [number, number, number, number];
  opacity: number;
}

/** 图层 patch（部分更新）。 */
export interface LayerPatch {
  name?: string;
  params?: Record<string, unknown>;
  style?: Partial<LayerStyle>;
  transform?: Partial<Transform2D>;
  visible?: boolean;
  locked?: boolean;
}

// ===== 旧 Crosshair 类型（保留供迁移期使用，新代码不应再用） =====
// 这些类型仅用于解析可能的旧配置，不应在前端业务代码中使用。

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

// ===== Profile / AppConfig =====

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

/** 新格式 Profile：crosshair 为可选（旧格式兼容），layers 必有。 */
export interface Profile {
  /** 旧格式字段（新格式为 null）。 */
  crosshair?: Crosshair | null;
  /** 新格式：图层列表。 */
  layers: Layer[];
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
