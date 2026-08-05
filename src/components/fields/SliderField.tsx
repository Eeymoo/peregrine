import { Slider } from "@/components/ui/slider";
import { Label } from "@/components/ui/label";

/** 滑块字段组件 Props。 */
interface SliderFieldProps {
  /** 字段标签文本，渲染在第一行左侧。 */
  label: string;
  /** 当前数值。 */
  value: number;
  /** 最小值，默认 0。 */
  min?: number;
  /** 最大值，默认 100。 */
  max?: number;
  /** 步进值，默认 1。 */
  step?: number;
  /** 可选单位后缀（如 "%" / "x" / "°"），显示在数值输入框右侧。 */
  unit?: string;
  /** 是否禁用控件。 */
  disabled?: boolean;
  /** 数值变化回调，参数为新值。 */
  onChange: (v: number) => void;
}

/** 通用滑块字段：第一行 label + 可编辑数值（borderless），第二行 Radix `<Slider>`。
 *
 * slider 和 number input 共享 value/onChange 双向同步：
 * - 拖拽滑块 → 数值输入框同步更新；
 * - 键入数值 → 滑块位置同步更新（Radix 自动 clamp 到 min/max）。
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
  disabled,
  onChange,
}: SliderFieldProps) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-2">
        <Label className="text-sm">{label}</Label>
        <div className="flex items-center gap-0.5">
          <input
            type="number"
            value={value}
            min={min}
            max={max}
            step={step}
            disabled={disabled}
            onChange={(e) => {
              const raw = e.target.value;
              if (raw.trim() === "") {
                onChange(0);
                return;
              }
              const num = parseFloat(raw);
              onChange(Number.isFinite(num) ? num : 0);
            }}
            className="w-16 text-right text-sm bg-transparent border-b border-transparent focus:border-b focus:outline-none cursor-text"
          />
          {unit && <span className="text-xs text-muted-foreground">{unit}</span>}
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
