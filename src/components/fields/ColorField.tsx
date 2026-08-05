import { useState, useEffect } from "react";
import { Label } from "@/components/ui/label";

/** RGBA 颜色元组（每个分量 0..1）。 */
export type Rgba = [number, number, number, number];

/** 将 RGBA 元组转换为 `#rrggbb` hex 字符串（忽略 alpha）。 */
export function rgbaToHex(color: Rgba | undefined): string {
  if (!color) return "#ffffff";
  const r = Math.round(color[0] * 255);
  const g = Math.round(color[1] * 255);
  const b = Math.round(color[2] * 255);
  return `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}`;
}

/** 将 `#rrggbb` hex 字符串转换为 RGBA 元组，保留原 alpha（默认 1）。 */
export function hexToRgba(hex: string, alpha = 1): Rgba {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  return [r, g, b, alpha];
}

/** 判断字符串是否为合法 `#rrggbb` hex 颜色。 */
function isValidHex(s: string): boolean {
  return /^#[0-9a-fA-F]{6}$/.test(s);
}

/** 颜色字段组件 Props。 */
interface ColorFieldProps {
  /** 字段标签文本。 */
  label: string;
  /** 当前颜色（RGBA 元组）。 */
  value: Rgba;
  /** 是否禁用控件。 */
  disabled?: boolean;
  /** 可选快捷颜色列表，渲染为色块按钮。 */
  quickColors?: Rgba[];
  /** 颜色变化回调，参数为新 RGBA 元组。 */
  onChange: (v: Rgba) => void;
}

/** 通用颜色字段：第一行 label + 可编辑 hex 输入框；第二行 color picker + 可选快捷色块。
 *
 * hex 输入框解析失败（非法 hex）时保持当前值不变，不触发 onChange。
 */
export function ColorField({
  label,
  value,
  disabled,
  quickColors,
  onChange,
}: ColorFieldProps) {
  const currentHex = rgbaToHex(value);
  // 本地输入框状态：允许用户输入中间态（如 "#ff"），失焦/Enter 时校验。
  const [hexInput, setHexInput] = useState(currentHex);

  // 外部 value 变化时同步输入框（如点击 picker / 快捷色）。
  useEffect(() => {
    setHexInput(currentHex);
  }, [currentHex]);

  const commitHex = (raw: string) => {
    const trimmed = raw.trim();
    if (isValidHex(trimmed)) {
      onChange(hexToRgba(trimmed, value[3]));
    } else {
      // 解析失败：回退到当前值，不触发 onChange。
      setHexInput(currentHex);
    }
  };

  const isColorMatch = (qc: Rgba) =>
    value[0] === qc[0] && value[1] === qc[1] && value[2] === qc[2];

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between gap-2">
        <Label className="text-xs font-medium">{label}</Label>
        <input
          type="text"
          value={hexInput}
          disabled={disabled}
          onChange={(e) => setHexInput(e.target.value)}
          onBlur={(e) => commitHex(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              (e.target as HTMLInputElement).blur();
            }
          }}
          className="w-20 px-1 py-0.5 text-xs border rounded bg-background"
        />
      </div>
      <div className="flex gap-2 items-center">
        <input
          type="color"
          value={currentHex}
          disabled={disabled}
          onChange={(e) => onChange(hexToRgba(e.target.value, value[3]))}
          className="w-10 h-8 border rounded"
        />
        {quickColors && quickColors.length > 0 && (
          <div className="flex gap-1 flex-wrap ml-1">
            {quickColors.map((qc, i) => {
              const css = `rgb(${Math.round(qc[0] * 255)}, ${Math.round(qc[1] * 255)}, ${Math.round(qc[2] * 255)})`;
              return (
                <button
                  key={i}
                  type="button"
                  title={css}
                  disabled={disabled}
                  onClick={() => onChange([...qc] as Rgba)}
                  className={`w-5 h-5 rounded-full border-2 transition-colors ${disabled ? "opacity-50 cursor-not-allowed" : ""}`}
                  style={{
                    backgroundColor: css,
                    borderColor: isColorMatch(qc) ? "hsl(var(--primary))" : "hsl(var(--border))",
                  }}
                />
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
