import { useEffect, useRef, useState } from "react";
import type { Crosshair } from "@/types/config";
import { buildShapes, colorToCss } from "@/lib/shapes";
import { useI18n } from "@/lib/i18n";

interface PreviewProps {
  crosshair: Crosshair;
  aspectRatio?: number;
}

export function Preview({ crosshair, aspectRatio = 16 / 9 }: PreviewProps) {
  const { t } = useI18n();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [, forceTick] = useState(0);

  // 监听容器尺寸变化（窗口拖拽、缩放等），强制重绘预览。
  const [sizeTick, setSizeTick] = useState(0);
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setSizeTick((n) => n + 1));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

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

    // 清空背景
    ctx.clearRect(0, 0, rect.width, rect.height);

    // 棋盘格背景
    const cell = 20;
    for (let y = 0; y < rect.height; y += cell) {
      for (let x = 0; x < rect.width; x += cell) {
        const dark = (((x / cell) | 0) + ((y / cell) | 0)) % 2 === 0;
        ctx.fillStyle = dark ? "#1a1a1a" : "#2a2a2a";
        ctx.fillRect(x, y, cell, cell);
      }
    }

    // 计算预览区域（contain 缩放）
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

    // 虚拟屏幕：以真实分辨率（短边 1080）构建准心，再缩放到预览区域。
    // 这样预览中准心的大小比例与实际 overlay 完全一致（所见即所得）。
    const REFERENCE_SHORT = 1080;
    let realW: number, realH: number;
    if (aspectRatio >= 1) {
      realH = REFERENCE_SHORT;
      realW = realH * aspectRatio;
    } else {
      realW = REFERENCE_SHORT;
      realH = realW / aspectRatio;
    }
    const virtualScreen = { minX: 0, minY: 0, maxX: realW, maxY: realH };
    const sx = pw / realW;
    const sy = ph / realH;

    // CustomImage 占位文本不需要缩放，直接在预览坐标系绘制
    if (crosshair.style === "custom_image") {
      const cx = px + pw / 2;
      const cy = py + ph / 2;
      if (!crosshair.image_path.trim()) {
        ctx.fillStyle = "#888";
        ctx.font = "14px sans-serif";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(t("preview.placeholder"), cx, cy);
      } else {
        ctx.fillStyle = colorToCss(crosshair.color, crosshair.opacity);
        ctx.font = "14px sans-serif";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        const name = crosshair.image_path.split(/[\\/]/).pop() || crosshair.image_path;
        ctx.fillText(name, cx, cy);
      }
      return;
    }

    // 在虚拟屏幕分辨率下生成图元，确保比例与实际 overlay 一致
    const shapes = buildShapes(virtualScreen, crosshair);
    const color = colorToCss(crosshair.color, crosshair.opacity);
    ctx.fillStyle = color;
    ctx.strokeStyle = color;

    // 应用缩放变换：虚拟坐标 → 预览区域
    ctx.save();
    ctx.translate(px, py);
    ctx.scale(sx, sy);

    for (const shape of shapes) {
      switch (shape.type) {
        case "rect":
          ctx.fillRect(shape.x, shape.y, shape.w, shape.h);
          break;
        case "circle":
          ctx.beginPath();
          ctx.arc(shape.cx, shape.cy, shape.radius, 0, Math.PI * 2);
          ctx.fill();
          break;
        case "circleStroke":
          ctx.lineWidth = shape.thickness;
          ctx.beginPath();
          ctx.arc(shape.cx, shape.cy, shape.radius, 0, Math.PI * 2);
          ctx.stroke();
          break;
        case "dashedCircle":
          drawDashedCircle(ctx, shape.cx, shape.cy, shape.radius, shape.thickness, shape.dashLen, shape.gapLen);
          break;
        case "triangle":
          ctx.beginPath();
          ctx.moveTo(shape.x1, shape.y1);
          ctx.lineTo(shape.x2, shape.y2);
          ctx.lineTo(shape.x3, shape.y3);
          ctx.closePath();
          ctx.fill();
          break;
      }
    }

    ctx.restore();
  }, [crosshair, aspectRatio, t, sizeTick]);

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

function drawDashedCircle(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  radius: number,
  thickness: number,
  dashLen: number,
  gapLen: number
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


