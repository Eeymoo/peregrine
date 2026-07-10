import { useEffect, useRef } from "react";
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
        const dark = ((x / cell) | 0) + ((y / cell) | 0) % 2 === 0;
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

    const screen = { minX: px, minY: py, maxX: px + pw, maxY: py + ph };

    // CustomImage 单独处理
    if (crosshair.style === "custom_image") {
      drawCustomImage(ctx, crosshair, screen, t);
      return;
    }

    const shapes = buildShapes(screen, crosshair);
    const color = colorToCss(crosshair.color, crosshair.opacity);
    ctx.fillStyle = color;
    ctx.strokeStyle = color;

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
  }, [crosshair, aspectRatio]);

  return (
    <canvas
      ref={canvasRef}
      className="w-full h-full rounded-md"
      style={{ imageRendering: "auto" }}
    />
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

function drawCustomImage(
  ctx: CanvasRenderingContext2D,
  crosshair: Crosshair,
  screen: { minX: number; minY: number; maxX: number; maxY: number },
  t: (key: string) => string
) {
  const centerX = (screen.minX + screen.maxX) / 2 + crosshair.image_offset_x;
  const centerY = (screen.minY + screen.maxY) / 2 + crosshair.image_offset_y;

  if (!crosshair.image_path.trim()) {
    ctx.fillStyle = "#888";
    ctx.font = "14px sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(t("preview.placeholder"), centerX, centerY);
    return;
  }

  // 前端无法直接读取本地文件路径的图像，占位显示文件名
  ctx.fillStyle = colorToCss(crosshair.color, crosshair.opacity);
  ctx.font = "14px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  const name = crosshair.image_path.split(/[\\/]/).pop() || crosshair.image_path;
  ctx.fillText(name, centerX, centerY);
}
