import type { Crosshair } from "@/types/config";

export interface RectF {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

export type Shape =
  | { type: "rect"; x: number; y: number; w: number; h: number }
  | { type: "circle"; cx: number; cy: number; radius: number }
  | { type: "circleStroke"; cx: number; cy: number; radius: number; thickness: number }
  | { type: "dashedCircle"; cx: number; cy: number; radius: number; thickness: number; dashLen: number; gapLen: number }
  | { type: "triangle"; x1: number; y1: number; x2: number; y2: number; x3: number; y3: number };

const OrbPosition = {
  TOP: 0b0001,
  BOTTOM: 0b0010,
  LEFT: 0b0100,
  RIGHT: 0b1000,
};

class SimpleRng {
  private state: bigint;
  constructor(seed: number) {
    // 与 Rust 侧 SimpleRng 使用相同的 64-bit LCG 种子。
    this.state = BigInt(Math.max(seed, 1));
  }
  nextU64(): bigint {
    // 与 crates/peregrine/src/shapes.rs 保持一致：
    // state = state * 6364136223846793005 + 1
    const MULT = 6364136223846793005n;
    this.state = (this.state * MULT + 1n) & 0xffffffffffffffffn;
    return this.state;
  }
  nextF32(): number {
    return Number(this.nextU64() & 0x00ffffffn) / 0x01000000;
  }
}

export function buildShapes(screen: RectF, crosshair: Crosshair): Shape[] {
  const cx = (screen.minX + screen.maxX) / 2;
  const cy = (screen.minY + screen.maxY) / 2;
  const shapes: Shape[] = [];

  switch (crosshair.style) {
    case "cross": {
      const arm = crosshair.size;
      const halfGap = crosshair.gap / 2;
      const thickness = crosshair.thickness;
      shapes.push({ type: "rect", x: cx - arm, y: cy - thickness / 2, w: arm - halfGap, h: thickness });
      shapes.push({ type: "rect", x: cx + halfGap, y: cy - thickness / 2, w: arm - halfGap, h: thickness });
      shapes.push({ type: "rect", x: cx - thickness / 2, y: cy - arm, w: thickness, h: arm - halfGap });
      shapes.push({ type: "rect", x: cx - thickness / 2, y: cy + halfGap, w: thickness, h: arm - halfGap });
      break;
    }
    case "large_cross": {
      const thickness = crosshair.thickness;
      shapes.push({ type: "rect", x: screen.minX, y: cy - thickness / 2, w: screen.maxX - screen.minX, h: thickness });
      shapes.push({ type: "rect", x: cx - thickness / 2, y: screen.minY, w: thickness, h: screen.maxY - screen.minY });
      break;
    }
    case "edge_rect": {
      const w = crosshair.size;
      const h = crosshair.secondary_size;
      const margin = crosshair.margin;
      let px = cx, py = cy;
      switch (crosshair.anchor) {
        case "top": py = screen.minY + h / 2 + margin; break;
        case "bottom": py = screen.maxY - h / 2 - margin; break;
        case "left": px = screen.minX + w / 2 + margin; break;
        case "right": px = screen.maxX - w / 2 - margin; break;
        case "center": default: break;
      }
      shapes.push({ type: "rect", x: px - w / 2, y: py - h / 2, w, h });
      break;
    }
    case "corner_dots4":
    case "corner_dots6":
    case "corner_dots8": {
      const configuredOffset = crosshair.offset > 0 ? crosshair.offset : crosshair.size;
      const offset = Math.min(configuredOffset, (screen.maxX - screen.minX) / 4, (screen.maxY - screen.minY) / 4);
      const radius = crosshair.radius > 0 ? crosshair.radius : crosshair.thickness * 3;
      const corners: [number, number][] = [
        [screen.minX + offset, screen.minY + offset],
        [screen.maxX - offset, screen.minY + offset],
        [screen.minX + offset, screen.maxY - offset],
        [screen.maxX - offset, screen.maxY - offset],
      ];
      for (const [x, y] of corners) shapes.push({ type: "circle", cx: x, cy: y, radius });
      if (crosshair.style === "corner_dots6" || crosshair.style === "corner_dots8") {
        shapes.push({ type: "circle", cx, cy: screen.minY + offset, radius });
        shapes.push({ type: "circle", cx, cy: screen.maxY - offset, radius });
      }
      if (crosshair.style === "corner_dots8") {
        shapes.push({ type: "circle", cx: screen.minX + offset, cy, radius });
        shapes.push({ type: "circle", cx: screen.maxX - offset, cy, radius });
      }
      break;
    }
    case "ring": {
      const radius = (screen.maxY - screen.minY) * crosshair.ring_radius_pct;
      const thickness = crosshair.thickness;
      switch (crosshair.ring_style) {
        case "solid":
          shapes.push({ type: "circleStroke", cx, cy, radius, thickness });
          break;
        case "dashed":
          shapes.push({ type: "dashedCircle", cx, cy, radius, thickness, dashLen: 4, gapLen: 4 });
          break;
        case "double":
          shapes.push({ type: "circleStroke", cx, cy, radius: radius - 2, thickness: 1 });
          shapes.push({ type: "dashedCircle", cx, cy, radius: radius + 2, thickness: 1, dashLen: 4, gapLen: 4 });
          break;
      }
      break;
    }
    case "custom_orb": {
      const radius = Math.max(crosshair.radius, 1);
      const offset = crosshair.offset;
      if (crosshair.orb_positions & OrbPosition.TOP) {
        edgeOrbShapes(shapes, screen.minX, screen.minY + offset, screen.maxX, screen.minY + offset, crosshair.custom_orb_top_count, radius);
      }
      if (crosshair.orb_positions & OrbPosition.BOTTOM) {
        edgeOrbShapes(shapes, screen.minX, screen.maxY - offset, screen.maxX, screen.maxY - offset, crosshair.custom_orb_bottom_count, radius);
      }
      if (crosshair.orb_positions & OrbPosition.LEFT) {
        edgeOrbShapes(shapes, screen.minX + offset, screen.minY, screen.minX + offset, screen.maxY, crosshair.custom_orb_left_count, radius);
      }
      if (crosshair.orb_positions & OrbPosition.RIGHT) {
        edgeOrbShapes(shapes, screen.maxX - offset, screen.minY, screen.maxX - offset, screen.maxY, crosshair.custom_orb_right_count, radius);
      }
      break;
    }
    case "random_orb": {
      // 与 Rust 侧 seed 计算保持一致（u64 wrapping_add）。
      const seed =
        (BigInt(Math.trunc(crosshair.random_orb_offset * 1000)) &
          0xffffffffffffffffn) +
        (BigInt(Math.trunc(crosshair.random_orb_jitter * 100)) &
          0xffffffffffffffffn) +
        (BigInt(Math.trunc(crosshair.random_radius_min * 10)) &
          0xffffffffffffffffn) +
        (BigInt(Math.trunc(crosshair.random_radius_max * 10)) &
          0xffffffffffffffffn) +
        (BigInt(crosshair.random_orb_count) & 0xffffffffffffffffn);
      const rng = new SimpleRng(Number(seed)); // 配置值保证在 JS 安全整数范围内
      const count = crosshair.random_orb_count;
      const offset = crosshair.random_orb_offset;
      const jitter = crosshair.random_orb_jitter;
      const minR = crosshair.random_radius_min;
      const maxR = crosshair.random_radius_max;
      for (let edge = 0; edge < 4; edge++) {
        for (let i = 0; i < count; i++) {
          const radius = minR + rng.nextF32() * (maxR - minR);
          const j = (rng.nextF32() - 0.5) * 2 * jitter;
          let x = 0, y = 0;
          switch (edge) {
            case 0: x = screen.minX + rng.nextF32() * (screen.maxX - screen.minX) + j; y = screen.minY + offset + j; break;
            case 1: x = screen.minX + rng.nextF32() * (screen.maxX - screen.minX) + j; y = screen.maxY - offset + j; break;
            case 2: y = screen.minY + rng.nextF32() * (screen.maxY - screen.minY) + j; x = screen.minX + offset + j; break;
            default: y = screen.minY + rng.nextF32() * (screen.maxY - screen.minY) + j; x = screen.maxX - offset + j; break;
          }
          shapes.push({ type: "circle", cx: x, cy: y, radius });
        }
      }
      break;
    }
    case "border_frame": {
      const thickness = crosshair.thickness;
      const offset = crosshair.offset;
      const topY = screen.minY + offset;
      const bottomY = screen.maxY - offset;
      const leftX = screen.minX + offset;
      const rightX = screen.maxX - offset;
      if (crosshair.border_frame_style === "solid") {
        solidFrameShapes(shapes, screen, topY, bottomY, leftX, rightX, thickness);
      } else {
        gapFrameShapes(shapes, screen, topY, bottomY, leftX, rightX, thickness);
      }
      break;
    }
    case "edge_arrows": {
      const BASE_SHORT = 1080;
      const screenShort = Math.min(screen.maxX - screen.minX, screen.maxY - screen.minY);
      const scale = screenShort / BASE_SHORT;
      const triSize = Math.max(crosshair.size, 4) * scale;
      const triHalf = triSize * 0.6;
      const defaultTail = Math.max(crosshair.arrow_distance, 0) * scale;
      const [tailTop, tailBottom, tailLeft, tailRight] = crosshair.arrow_tail_per_edge
        ? [
            Math.max(crosshair.arrow_tail_top, 0) * scale,
            Math.max(crosshair.arrow_tail_bottom, 0) * scale,
            Math.max(crosshair.arrow_tail_left, 0) * scale,
            Math.max(crosshair.arrow_tail_right, 0) * scale,
          ]
        : [defaultTail, defaultTail, defaultTail, defaultTail];
      const tailHalf = crosshair.arrow_width > 0 ? Math.min((crosshair.arrow_width / 2) * scale, triHalf) : triHalf;

      const pos = crosshair.orb_positions;
      const allOff = !(pos & OrbPosition.TOP) && !(pos & OrbPosition.BOTTOM) && !(pos & OrbPosition.LEFT) && !(pos & OrbPosition.RIGHT);
      const showTop = allOff || !!(pos & OrbPosition.TOP);
      const showBottom = allOff || !!(pos & OrbPosition.BOTTOM);
      const showLeft = allOff || !!(pos & OrbPosition.LEFT);
      const showRight = allOff || !!(pos & OrbPosition.RIGHT);

      if (showTop) {
        const baseY = screen.minY + tailTop;
        const tipY = baseY + triSize;
        if (tailTop > 0) shapes.push({ type: "rect", x: cx - tailHalf, y: screen.minY, w: tailHalf * 2, h: tailTop });
        shapes.push({ type: "triangle", x1: cx, y1: tipY, x2: cx - triHalf, y2: baseY, x3: cx + triHalf, y3: baseY });
      }
      if (showBottom) {
        const baseY = screen.maxY - tailBottom;
        const tipY = baseY - triSize;
        if (tailBottom > 0) shapes.push({ type: "rect", x: cx - tailHalf, y: baseY, w: tailHalf * 2, h: tailBottom });
        shapes.push({ type: "triangle", x1: cx, y1: tipY, x2: cx - triHalf, y2: baseY, x3: cx + triHalf, y3: baseY });
      }
      if (showLeft) {
        const baseX = screen.minX + tailLeft;
        const tipX = baseX + triSize;
        if (tailLeft > 0) shapes.push({ type: "rect", x: screen.minX, y: cy - tailHalf, w: tailLeft, h: tailHalf * 2 });
        shapes.push({ type: "triangle", x1: tipX, y1: cy, x2: baseX, y2: cy - triHalf, x3: baseX, y3: cy + triHalf });
      }
      if (showRight) {
        const baseX = screen.maxX - tailRight;
        const tipX = baseX - triSize;
        if (tailRight > 0) shapes.push({ type: "rect", x: baseX, y: cy - tailHalf, w: tailRight, h: tailHalf * 2 });
        shapes.push({ type: "triangle", x1: tipX, y1: cy, x2: baseX, y2: cy - triHalf, x3: baseX, y3: cy + triHalf });
      }
      break;
    }
    case "custom_image":
      break;
    case "grid": {
      const cell = Math.max(crosshair.grid_size ?? 80, 10);
      const thickness = crosshair.thickness;
      const halfT = thickness / 2;
      const cols = Math.max(1, Math.ceil((screen.maxX - screen.minX) / cell));
      const rows = Math.max(1, Math.ceil((screen.maxY - screen.minY) / cell));
      if (crosshair.grid_alignment === "edge") {
        // 贴边：拉伸填满整个屏幕，无空隙。
        const cellW = (screen.maxX - screen.minX) / cols;
        const cellH = (screen.maxY - screen.minY) / rows;
        for (let i = 0; i <= cols; i++) {
          const x = screen.minX + cellW * i;
          shapes.push({ type: "rect", x: x - halfT, y: screen.minY, w: thickness, h: screen.maxY - screen.minY });
        }
        for (let i = 0; i <= rows; i++) {
          const y = screen.minY + cellH * i;
          shapes.push({ type: "rect", x: screen.minX, y: y - halfT, w: screen.maxX - screen.minX, h: thickness });
        }
      } else {
        // 居中：正方形格子居中。
        const totalW = cell * cols;
        const totalH = cell * rows;
        const offsetX = ((screen.maxX - screen.minX) - totalW) / 2;
        const offsetY = ((screen.maxY - screen.minY) - totalH) / 2;
        for (let i = 1; i < cols; i++) {
          const x = screen.minX + offsetX + cell * i;
          shapes.push({ type: "rect", x: x - halfT, y: screen.minY + offsetY, w: thickness, h: totalH });
        }
        for (let i = 1; i < rows; i++) {
          const y = screen.minY + offsetY + cell * i;
          shapes.push({ type: "rect", x: screen.minX + offsetX, y: y - halfT, w: totalW, h: thickness });
        }
      }
      break;
    }
    default:
      break;
  }

  return shapes;
}

function edgeOrbShapes(shapes: Shape[], x0: number, y0: number, x1: number, y1: number, count: number, radius: number) {
  if (count === 0) return;
  if (count === 1) {
    shapes.push({ type: "circle", cx: (x0 + x1) / 2, cy: (y0 + y1) / 2, radius });
    return;
  }
  const step = 1 / (count + 1);
  for (let i = 1; i <= count; i++) {
    const t = i * step;
    shapes.push({ type: "circle", cx: x0 + (x1 - x0) * t, cy: y0 + (y1 - y0) * t, radius });
  }
}

function solidFrameShapes(shapes: Shape[], rect: RectF, topY: number, bottomY: number, leftX: number, rightX: number, thickness: number) {
  const halfT = thickness / 2;
  const w = rect.maxX - rect.minX;
  const h = rect.maxY - rect.minY;
  shapes.push({ type: "rect", x: rect.minX, y: topY - halfT, w, h: thickness });
  shapes.push({ type: "rect", x: rect.minX, y: bottomY - halfT, w, h: thickness });
  shapes.push({ type: "rect", x: leftX - halfT, y: rect.minY, w: thickness, h });
  shapes.push({ type: "rect", x: rightX - halfT, y: rect.minY, w: thickness, h });
}

function gapFrameShapes(shapes: Shape[], rect: RectF, topY: number, bottomY: number, leftX: number, rightX: number, thickness: number) {
  const halfT = thickness / 2;
  const halfGapW = (rect.maxX - rect.minX) * 0.2 / 2;
  const halfGapH = (rect.maxY - rect.minY) * 0.2 / 2;
  const cx = (rect.minX + rect.maxX) / 2;
  const cy = (rect.minY + rect.maxY) / 2;
  shapes.push({ type: "rect", x: rect.minX, y: topY - halfT, w: cx - halfGapW - rect.minX, h: thickness });
  shapes.push({ type: "rect", x: cx + halfGapW, y: topY - halfT, w: rect.maxX - (cx + halfGapW), h: thickness });
  shapes.push({ type: "rect", x: rect.minX, y: bottomY - halfT, w: cx - halfGapW - rect.minX, h: thickness });
  shapes.push({ type: "rect", x: cx + halfGapW, y: bottomY - halfT, w: rect.maxX - (cx + halfGapW), h: thickness });
  shapes.push({ type: "rect", x: leftX - halfT, y: rect.minY, w: thickness, h: cy - halfGapH - rect.minY });
  shapes.push({ type: "rect", x: leftX - halfT, y: cy + halfGapH, w: thickness, h: rect.maxY - (cy + halfGapH) });
  shapes.push({ type: "rect", x: rightX - halfT, y: rect.minY, w: thickness, h: cy - halfGapH - rect.minY });
  shapes.push({ type: "rect", x: rightX - halfT, y: cy + halfGapH, w: thickness, h: rect.maxY - (cy + halfGapH) });
}

export function colorToCss(color: [number, number, number, number], opacity: number): string {
  const r = Math.round(color[0] * 255);
  const g = Math.round(color[1] * 255);
  const b = Math.round(color[2] * 255);
  const a = color[3] * opacity;
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}
