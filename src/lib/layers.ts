/**
 * 图层与旧版 Crosshair 之间的双向映射工具。
 *
 * 单图层（旧版）UI 仍以 Crosshair 为编辑模型，底层渲染走 layers；
 * 这里提供 Crosshair → Layer 的同步映射与 Layer → Crosshair 的反向生成，
 * 保证两种 UI 模式读写同一份数据。
 */
import { getDefaultCrosshairForStyle } from "@/lib/presets";
import type {
  Anchor,
  BorderFrameStyle,
  Crosshair,
  CrosshairStyle,
  GridAlignment,
  Layer,
  LayerStyle,
  MaterialRef,
  RingStyle,
} from "@/types/config";

/** 可在单图层（旧版）UI 中编辑的内置物料 id 列表。 */
const LEGACY_MATERIAL_IDS = [
  "builtin.cross",
  "builtin.edge_rect",
  "builtin.large_cross",
  "builtin.corner_dots",
  "builtin.ring",
  "builtin.custom_orb",
  "builtin.random_orb",
  "builtin.border_frame",
  "builtin.edge_arrows",
  "builtin.grid",
  "builtin.image",
];

/** 判断单个图层是否可在单图层（旧版）UI 中编辑。 */
export function isLayerLegacyCompatible(layer: Layer | undefined): boolean {
  if (!layer) return false;
  if (layer.material.kind !== "builtin") return false;
  if (layer.transform.offset_x !== 0 || layer.transform.offset_y !== 0) return false;
  if (layer.transform.scale !== 1 || layer.transform.rotation_deg !== 0) return false;
  if (layer.style.blend_mode !== "normal") return false;
  return LEGACY_MATERIAL_IDS.includes(layer.material.id);
}

/** 判断当前 profile 是否可在单图层（旧版）UI 中编辑（恰好一个兼容图层）。 */
export function isProfileLegacyCompatible(
  profile: { layers: Layer[] } | undefined | null,
): boolean {
  if (!profile) return false;
  return profile.layers.length === 1 && isLayerLegacyCompatible(profile.layers[0]);
}

/** 把 Crosshair 样式映射到内置物料引用。 */
export function crosshairToMaterial(style: CrosshairStyle): MaterialRef {
  switch (style) {
    case "edge_rect":
      return { kind: "builtin", id: "builtin.edge_rect" };
    case "custom_image":
      return { kind: "builtin", id: "builtin.image" };
    case "cross":
      return { kind: "builtin", id: "builtin.cross" };
    case "large_cross":
      return { kind: "builtin", id: "builtin.large_cross" };
    case "corner_dots4":
    case "corner_dots6":
    case "corner_dots8":
      return { kind: "builtin", id: "builtin.corner_dots" };
    case "ring":
      return { kind: "builtin", id: "builtin.ring" };
    case "custom_orb":
      return { kind: "builtin", id: "builtin.custom_orb" };
    case "random_orb":
      return { kind: "builtin", id: "builtin.random_orb" };
    case "border_frame":
      return { kind: "builtin", id: "builtin.border_frame" };
    case "edge_arrows":
      return { kind: "builtin", id: "builtin.edge_arrows" };
    case "grid":
      return { kind: "builtin", id: "builtin.grid" };
    default:
      return { kind: "builtin", id: "builtin.cross" };
  }
}

/** 把 Crosshair 字段转换为对应物料的 params。 */
export function crosshairToParams(crosshair: Crosshair): Record<string, unknown> {
  switch (crosshair.style) {
    case "edge_rect":
      return {
        size: crosshair.size,
        secondary_size: crosshair.secondary_size,
        anchor: crosshair.anchor,
        margin: crosshair.margin,
        corner_radius: crosshair.corner_radius,
      };
    case "custom_image":
      return {
        path: crosshair.image_path,
        scale: crosshair.image_scale,
        offset_x: crosshair.image_offset_x,
        offset_y: crosshair.image_offset_y,
        width: crosshair.size,
        height: crosshair.size,
      };
    case "cross":
      return {
        size: crosshair.size,
        thickness: crosshair.thickness,
        gap: crosshair.gap,
      };
    case "large_cross":
      return {
        thickness: crosshair.thickness,
      };
    case "corner_dots4":
    case "corner_dots6":
    case "corner_dots8":
      return {
        count: styleToCornerCount(crosshair.style),
        offset: crosshair.offset,
        thickness: crosshair.thickness,
        radius: crosshair.radius,
      };
    case "ring":
      return {
        radius_pct: crosshair.ring_radius_pct,
        thickness: crosshair.thickness,
        style: crosshair.ring_style,
      };
    case "custom_orb":
      return {
        radius: crosshair.radius,
        offset: crosshair.offset,
        top_count: crosshair.custom_orb_top_count,
        bottom_count: crosshair.custom_orb_bottom_count,
        left_count: crosshair.custom_orb_left_count,
        right_count: crosshair.custom_orb_right_count,
        orb_positions: crosshair.orb_positions,
      };
    case "random_orb":
      return {
        count: crosshair.random_orb_count,
        offset: crosshair.random_orb_offset,
        jitter: crosshair.random_orb_jitter,
        radius_min: crosshair.random_radius_min,
        radius_max: crosshair.random_radius_max,
      };
    case "border_frame":
      return {
        thickness: crosshair.thickness,
        offset: crosshair.offset,
        style: crosshair.border_frame_style,
        inset: crosshair.border_inset,
      };
    case "edge_arrows":
      return {
        size: crosshair.size,
        arrow_width: crosshair.arrow_width,
        distance: crosshair.arrow_distance,
        tail_per_edge: crosshair.arrow_tail_per_edge,
        tail_top: crosshair.arrow_tail_top,
        tail_bottom: crosshair.arrow_tail_bottom,
        tail_left: crosshair.arrow_tail_left,
        tail_right: crosshair.arrow_tail_right,
      };
    case "grid":
      return {
        size: crosshair.grid_size,
        thickness: crosshair.thickness,
        alignment: crosshair.grid_alignment,
      };
    default:
      return {};
  }
}

function styleToCornerCount(style: CrosshairStyle): number {
  switch (style) {
    case "corner_dots6":
      return 6;
    case "corner_dots8":
      return 8;
    default:
      return 4;
  }
}

/** 以 Crosshair 为蓝本创建默认单图层。 */
export function createDefaultLayer(crosshair: Crosshair): Layer {
  return {
    id: crypto.randomUUID(),
    name: "crosshair",
    material: crosshairToMaterial(crosshair.style),
    params: crosshairToParams(crosshair),
    style: {
      color: crosshair.color,
      opacity: crosshair.opacity,
      blend_mode: "normal",
    },
    transform: {
      offset_x: 0,
      offset_y: 0,
      scale: 1,
      rotation_deg: 0,
    },
    visible: true,
    locked: false,
  };
}

/**
 * 把旧版 Crosshair 同步映射到 layers[0]。
 * 这样旧 UI 编辑 crosshair 时，底层渲染仍然走 layers。
 */
export function syncCrosshairToLayer(
  crosshair: Crosshair,
  layer: Layer,
  fallbackName: string,
): Layer {
  const material = crosshairToMaterial(crosshair.style);
  const params = crosshairToParams(crosshair);
  const style: LayerStyle = {
    color: crosshair.color,
    opacity: crosshair.opacity,
    blend_mode: layer.style.blend_mode,
  };
  return {
    ...layer,
    name: layer.name || fallbackName,
    material,
    params,
    style,
    transform: layer.transform ?? {
      offset_x: 0,
      offset_y: 0,
      scale: 1,
      rotation_deg: 0,
    },
  };
}

/** 从内置物料 id 推断 CrosshairStyle。 */
export function materialIdToStyle(materialId: string): CrosshairStyle {
  switch (materialId) {
    case "builtin.edge_rect":
      return "edge_rect";
    case "builtin.image":
      return "custom_image";
    case "builtin.large_cross":
      return "large_cross";
    case "builtin.corner_dots":
      return "corner_dots4";
    case "builtin.ring":
      return "ring";
    case "builtin.custom_orb":
      return "custom_orb";
    case "builtin.random_orb":
      return "random_orb";
    case "builtin.border_frame":
      return "border_frame";
    case "builtin.edge_arrows":
      return "edge_arrows";
    case "builtin.grid":
      return "grid";
    case "builtin.cross":
    default:
      return "cross";
  }
}

/**
 * 从 layers[0] 反向生成 Crosshair，用于旧版单图层 UI。
 * 无法精确还原的字段使用默认值。
 */
export function layerToCrosshair(layer: Layer): Crosshair | null {
  const style = materialIdToStyle(
    layer.material.kind === "builtin" ? layer.material.id : "builtin.cross",
  );
  const params = layer.params;
  const base = getDefaultCrosshairForStyle(style);
  const color = layer.style?.color ?? base.color;
  const opacity = layer.style?.opacity ?? base.opacity;
  const newBase: Crosshair = { ...base, color, opacity, style };

  switch (style) {
    case "edge_rect":
      return {
        ...newBase,
        size: (params.size as number) ?? newBase.size,
        secondary_size: (params.secondary_size as number) ?? newBase.secondary_size,
        anchor: (params.anchor as Anchor) ?? newBase.anchor,
        margin: (params.margin as number) ?? newBase.margin,
        corner_radius: (params.corner_radius as number) ?? newBase.corner_radius,
      };
    case "custom_image":
      return {
        ...newBase,
        image_path: (params.path as string) ?? newBase.image_path,
        image_scale: (params.scale as number) ?? newBase.image_scale,
        image_offset_x: (params.offset_x as number) ?? newBase.image_offset_x,
        image_offset_y: (params.offset_y as number) ?? newBase.image_offset_y,
        size: (params.width as number) ?? newBase.size,
      };
    case "cross":
      return {
        ...newBase,
        size: (params.size as number) ?? newBase.size,
        thickness: (params.thickness as number) ?? newBase.thickness,
        gap: (params.gap as number) ?? newBase.gap,
      };
    case "large_cross":
      return {
        ...newBase,
        thickness: (params.thickness as number) ?? newBase.thickness,
      };
    case "corner_dots4":
    case "corner_dots6":
    case "corner_dots8": {
      const count = (params.count as number) ?? 4;
      const styleName: CrosshairStyle =
        count === 6 ? "corner_dots6" : count === 8 ? "corner_dots8" : "corner_dots4";
      return {
        ...newBase,
        style: styleName,
        offset: (params.offset as number) ?? newBase.offset,
        thickness: (params.thickness as number) ?? newBase.thickness,
        radius: (params.radius as number) ?? newBase.radius,
      };
    }
    case "ring":
      return {
        ...newBase,
        ring_radius_pct: (params.radius_pct as number) ?? newBase.ring_radius_pct,
        thickness: (params.thickness as number) ?? newBase.thickness,
        ring_style: (params.style as RingStyle) ?? newBase.ring_style,
      };
    case "custom_orb":
      return {
        ...newBase,
        radius: (params.radius as number) ?? newBase.radius,
        offset: (params.offset as number) ?? newBase.offset,
        orb_positions: (params.orb_positions as number) ?? newBase.orb_positions,
        custom_orb_top_count:
          (params.top_count as number) ?? newBase.custom_orb_top_count,
        custom_orb_bottom_count:
          (params.bottom_count as number) ?? newBase.custom_orb_bottom_count,
        custom_orb_left_count:
          (params.left_count as number) ?? newBase.custom_orb_left_count,
        custom_orb_right_count:
          (params.right_count as number) ?? newBase.custom_orb_right_count,
      };
    case "random_orb":
      return {
        ...newBase,
        random_orb_count: (params.count as number) ?? newBase.random_orb_count,
        random_orb_offset: (params.offset as number) ?? newBase.random_orb_offset,
        random_orb_jitter: (params.jitter as number) ?? newBase.random_orb_jitter,
        random_radius_min: (params.radius_min as number) ?? newBase.random_radius_min,
        random_radius_max: (params.radius_max as number) ?? newBase.random_radius_max,
      };
    case "border_frame":
      return {
        ...newBase,
        thickness: (params.thickness as number) ?? newBase.thickness,
        offset: (params.offset as number) ?? newBase.offset,
        border_frame_style:
          (params.style as BorderFrameStyle) ?? newBase.border_frame_style,
        border_inset: (params.inset as boolean) ?? newBase.border_inset,
      };
    case "edge_arrows":
      return {
        ...newBase,
        size: (params.size as number) ?? newBase.size,
        arrow_width: (params.arrow_width as number) ?? newBase.arrow_width,
        arrow_distance: (params.distance as number) ?? newBase.arrow_distance,
        arrow_tail_per_edge:
          (params.tail_per_edge as boolean) ?? newBase.arrow_tail_per_edge,
        arrow_tail_top: (params.tail_top as number) ?? newBase.arrow_tail_top,
        arrow_tail_bottom:
          (params.tail_bottom as number) ?? newBase.arrow_tail_bottom,
        arrow_tail_left: (params.tail_left as number) ?? newBase.arrow_tail_left,
        arrow_tail_right:
          (params.tail_right as number) ?? newBase.arrow_tail_right,
      };
    case "grid":
      return {
        ...newBase,
        grid_size: (params.size as number) ?? newBase.grid_size,
        thickness: (params.thickness as number) ?? newBase.thickness,
        grid_alignment:
          (params.alignment as GridAlignment) ?? newBase.grid_alignment,
      };
    default:
      return newBase;
  }
}
