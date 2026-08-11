import { Slider } from "@/components/ui/slider";
import { Label } from "@/components/ui/label";

/** 滑块字段组件 Props。 */
interface SliderFieldProps {
  /** 字段标签文本，渲染在第一行左侧。 */
  label: string;
  /** 当前数值（真实值，例如透明度 0.5）。 */
  value: number;
  /** 最小值，默认 0。 */
  min?: number;
  /** 最大值，默认 100。 */
  max?: number;
  /** 步进值，默认 1。 */
  step?: number;
  /** 可选单位后缀（如 "%" / "x" / "°"），显示在数值输入框右侧。
   *
   * 传入 `format` 时本字段被忽略（format 负责完整格式化）。
   */
  unit?: string;
  /** 可选的数值显示格式化回调（如透明度 0.5 → "50%"）。
   *
   * 与 `parse` 配对使用时，数值输入框仍可编辑：
   * - 展示时调用 `format(value)` 渲染（如 0.5 → "50%"）；
   * - 键入时调用 `parse(text)` 反解为真实值（如 "50" → 0.5）。
   *
   * 未传入 `parse` 时退化为只读展示（旧行为，向后兼容）。
   */
  format?: (v: number) => string;
  /** 可选的反解析回调，与 `format` 配对。
   *
   * 传入时数值输入框在 format 模式下也可编辑；键入文本由本回调转回真实值。
   * 未传入而 `format` 存在时，输入框只读。
   */
  parse?: (text: string) => number;
  /** 是否禁用控件。 */
  disabled?: boolean;
  /** 数值变化回调，参数为真实值（如透明度 0.5）。 */
  onChange: (v: number) => void;
}

/** 通用滑块字段：第一行 label + 可编辑数值（borderless），第二行 Radix `<Slider>`。
 *
 * slider 和 number input 共享 value/onChange 双向同步：
 * - 拖拽滑块 → 数值输入框同步更新；
 * - 键入数值 → 滑块位置同步更新（Radix 自动 clamp 到 min/max）。
 *
 * - 不传 `format`：显示原始 value + 可选 unit，input 可编辑。
 * - 传 `format` 且传 `parse`：input 显示 `format(value)`（如 "50%"），键入由 parse 转回真实值。
 * - 传 `format` 但不传 `parse`：input 只读展示 `format(value)`（旧行为）。
 *
 * 输入框解析失败时 fallback 到 0。
 */
export function SliderField({
  label,
  value,
  min = 0,
  max = 100,
  step = 1,
  unit,
  format,
  parse,
  disabled,
  onChange,
}: SliderFieldProps) {
  // format + parse 双传：可编辑，显示与键入都做转换。
  const editableFormatted = format !== undefined && parse !== undefined;
  // 仅 format 不传 parse：只读。
  const readOnlyFormatted = format !== undefined && parse === undefined;

  // 处理键入：format 模式下剥离非数字字符后用 parse 转回真实值，
  // 普通模式直接 parseFloat。结果 clamp 到 [min, max]，避免越界值传给后端。
  const handleInput = (raw: string) => {
    if (raw.trim() === "") {
      onChange(min);
      return;
    }
    let num: number;
    if (editableFormatted) {
      // 容忍用户键入的 "%" 等后缀：剥离非数字字符（保留小数点与负号）。
      const cleaned = raw.replace(/[^0-9.\-]/g, "");
      num = parse!(cleaned);
    } else {
      num = parseFloat(raw);
    }
    if (!Number.isFinite(num)) {
      onChange(min);
      return;
    }
    // clamp 到 [min, max]，避免键入越界值（如 opacity 输 150 → clamp 到 100% → 1.0）。
    onChange(Math.min(max, Math.max(min, num)));
  };

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-2">
        <Label className="text-sm">{label}</Label>
        <div className="flex items-center gap-0.5">
          {readOnlyFormatted ? (
            // 只读 format 模式（未传 parse）：仅展示，不可编辑。
            <span className="text-sm text-muted-foreground w-16 text-right">
              {format!(value)}
            </span>
          ) : (
            <>
              <input
                type="text"
                inputMode="decimal"
                value={editableFormatted ? format!(value) : value}
                disabled={disabled}
                onChange={(e) => handleInput(e.target.value)}
                className="w-16 text-right text-sm bg-transparent border-b border-transparent focus:border-b focus:outline-none cursor-text"
              />
              {!format && unit && (
                <span className="text-xs text-muted-foreground">{unit}</span>
              )}
            </>
          )}
        </div>
      </div>
      <Slider
        value={[value]}
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        onValueChange={([v]) => onChange(v)}
      />
    </div>
  );
}
