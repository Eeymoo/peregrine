import { useEffect, useRef, useState } from "react";
import type { BuiltShape, Element } from "@/types/config";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "@/lib/i18n";

interface PreviewProps {
  /** 触发重绘的依赖（如图层列表 / 参数变化时由父组件传入新值）。 */
  previewKey: unknown;
  /** 预览宽高比（默认 16:9）。 */
  aspectRatio?: number;
}

/**
 * 预览组件：通过 Tauri IPC 调用后端 `build_shapes_ipc` 获取图元列表，
 * 在 Canvas 上绘制。所有几何计算都在 Rust 侧（物料脚本），前端零几何逻辑。
 *
 * 调用节流 16ms（60fps）避免拖拽滑块时打爆 IPC。
 */
export function Preview({ previewKey, aspectRatio = 16 / 9 }: PreviewProps) {
  const { t } = useI18n();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [shapes, setShapes] = useState<BuiltShape[]>([]);
  const [loading, setLoading] = useState(false);
  const [sizeTick, setSizeTick] = useState(0);

  // 监听容器尺寸变化。
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setSizeTick((n) => n + 1));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // 通过 IPC 获取图元列表（节流 16ms）。
  useEffect(() => {
    let cancelled = false;
    setLoading(true);

    // 计算预览用的"虚拟屏幕"尺寸（参考短边 1080p）。
    const REFERENCE_SHORT = 1080;
    const realH = aspectRatio >= 1 ? REFERENCE_SHORT : REFERENCE_SHORT / aspectRatio;
    const realW = aspectRatio >= 1 ? REFERENCE_SHORT * aspectRatio : REFERENCE_SHORT;

    const timer = setTimeout(async () => {
      try {
        const result = await invoke<BuiltShape[]>("build_shapes_ipc", {
          screenW: realW,
          screenH: realH,
        });
        if (!cancelled) {
          setShapes(result);
        }
      } catch (err) {
        console.error("build_shapes_ipc failed:", err);
        if (!cancelled) {
          setShapes([]);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }, 16);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [previewKey, aspectRatio]);

  // 在 Canvas 上绘制图元列表。
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    // 清空背景。
    ctx.clearRect(0, 0, rect.width, rect.height);

    // 棋盘格背景。
    const cell = 20;
    for (let y = 0; y < rect.height; y += cell) {
      for (let x = 0; x < rect.width; x += cell) {
        const dark = (((x / cell) | 0) + ((y / cell) | 0)) % 2 === 0;
        ctx.fillStyle = dark ? "#1a1a1a" : "#2a2a2a";
        ctx.fillRect(x, y, cell, cell);
      }
    }

    // 计算预览区域（contain 缩放）。
    const availRatio = rect.width / rect.height;
    let pw: number, ph: number;
    if (availRatio > aspectRatio) {
      ph = rect.height;
      pw = ph * aspectRatio;
    } else {
      pw = rect.width;
      ph = pw / aspectRatio;
    }
    const px = (rect.width - pw) / 2;
    const py = (rect.height - ph) / 2;

    ctx.strokeStyle = "#666";
    ctx.lineWidth = 1;
    ctx.strokeRect(px, py, pw, ph);

    // 把虚拟屏幕坐标 (0,0)-(realW,realH) 映射到预览区域 (px,py)-(px+pw,py+ph)。
    const REFERENCE_SHORT = 1080;
    const realH_inner = aspectRatio >= 1 ? REFERENCE_SHORT : REFERENCE_SHORT / aspectRatio;
    const realW_inner = aspectRatio >= 1 ? REFERENCE_SHORT * aspectRatio : REFERENCE_SHORT;
    const sx = pw / realW_inner;
    const sy = ph / realH_inner;

    ctx.save();
    ctx.beginPath();
    ctx.rect(px, py, pw, ph);
    ctx.clip();
    ctx.translate(px, py);
    ctx.scale(sx, sy);

    for (const { element, color, opacity } of shapes) {
      drawElement(ctx, element, color, opacity);
    }

    ctx.restore();

    // 加载指示。
    if (loading) {
      ctx.fillStyle = "rgba(0, 0, 0, 0.3)";
      ctx.fillRect(rect.width - 60, 8, 52, 18);
      ctx.fillStyle = "#fff";
      ctx.font = "10px sans-serif";
      ctx.textAlign = "right";
      ctx.textBaseline = "top";
      ctx.fillText("rendering...", rect.width - 12, 12);
    }

    // 空状态提示。
    if (shapes.length === 0 && !loading) {
      ctx.fillStyle = "#888";
      ctx.font = "14px sans-serif";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(t("preview.placeholder"), rect.width / 2, rect.height / 2);
    }
  }, [shapes, aspectRatio, t, sizeTick, loading]);

  return (
    <div ref={containerRef} className="w-full h-full">
      <canvas
        ref={canvasRef}
        className="w-full h-full rounded-md"
        style={{ imageRendering: "auto" }}
      />
    </div>
  );
}

/** 把 RGBA + opacity 转换为 CSS rgba 字符串。 */
function colorToCss(color: [number, number, number, number], opacity: number): string {
  const r = Math.round(color[0] * 255);
  const g = Math.round(color[1] * 255);
  const b = Math.round(color[2] * 255);
  const a = color[3] * opacity;
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

/** 绘制单个 Element 图元。 */
function drawElement(
  ctx: CanvasRenderingContext2D,
  element: Element,
  color: [number, number, number, number],
  opacity: number,
) {
  const css = colorToCss(color, opacity);
  ctx.fillStyle = css;
  ctx.strokeStyle = css;

  switch (element.type) {
    case "rect":
      ctx.fillRect(element.x, element.y, element.w, element.h);
      break;
    case "circle":
      ctx.beginPath();
      ctx.arc(element.cx, element.cy, element.radius, 0, Math.PI * 2);
      ctx.fill();
      break;
    case "circle_stroke":
      ctx.lineWidth = element.thickness;
      ctx.beginPath();
      ctx.arc(element.cx, element.cy, element.radius, 0, Math.PI * 2);
      ctx.stroke();
      break;
    case "dashed_circle":
      drawDashedCircle(
        ctx,
        element.cx,
        element.cy,
        element.radius,
        element.thickness,
        element.dash_len,
        element.gap_len,
      );
      break;
    case "triangle":
      ctx.beginPath();
      ctx.moveTo(element.x1, element.y1);
      ctx.lineTo(element.x2, element.y2);
      ctx.lineTo(element.x3, element.y3);
      ctx.closePath();
      ctx.fill();
      break;
    case "polygon": {
      const pts = element.points;
      if (pts.length < 3) break;
      ctx.beginPath();
      ctx.moveTo(pts[0][0], pts[0][1]);
      for (let i = 1; i < pts.length; i++) {
        ctx.lineTo(pts[i][0], pts[i][1]);
      }
      ctx.closePath();
      ctx.fill();
      break;
    }
    case "line":
      ctx.lineWidth = element.thickness;
      ctx.lineCap = "round";
      ctx.beginPath();
      ctx.moveTo(element.x1, element.y1);
      ctx.lineTo(element.x2, element.y2);
      ctx.stroke();
      break;
    case "text":
      ctx.font = `${element.font_size}px sans-serif`;
      ctx.textAlign = "left";
      ctx.textBaseline = "alphabetic";
      ctx.fillText(element.content, element.x, element.y);
      break;
    case "image":
      // 图片渲染：预览阶段用占位矩形（实际 blit 在 overlay 阶段）。
      ctx.strokeStyle = css;
      ctx.lineWidth = 1;
      ctx.strokeRect(element.x, element.y, element.w, element.h);
      ctx.fillStyle = css;
      ctx.font = "12px sans-serif";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      const name = element.path.split(/[\\/]/).pop() || element.path;
      ctx.fillText(`[${name}]`, element.x + element.w / 2, element.y + element.h / 2);
      break;
  }
}

function drawDashedCircle(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  radius: number,
  thickness: number,
  dashLen: number,
  gapLen: number,
) {
  const circumference = 2 * Math.PI * radius;
  const unit = dashLen + gapLen;
  const segments = Math.ceil(circumference / unit);
  const step = (2 * Math.PI) / segments;
  ctx.lineWidth = thickness;
  for (let i = 0; i < segments; i++) {
    const startAngle = i * step;
    const endAngle = startAngle + step * (dashLen / unit);
    ctx.beginPath();
    ctx.arc(cx, cy, radius, startAngle, endAngle);
    ctx.stroke();
  }
}
