import { Label } from "@/components/ui/label";

/** 数值字段组件 Props。 */
interface NumberFieldProps {
  /** 字段标签文本。 */
  label: string;
  /** 当前数值。 */
  value: number;
  /** 最小值。 */
  min?: number;
  /** 最大值。 */
  max?: number;
  /** 步进值。 */
  step?: number;
  /** 是否禁用控件。 */
  disabled?: boolean;
  /** 数值变化回调，参数为新值。 */
  onChange: (v: number) => void;
}

/** 通用数值字段：第一行 label，第二行纯 `<input type="number">`（无滑块）。
 *
 * 与 SliderField 的区别：本组件只提供精确数值输入，不渲染滑块，
 * 适用于如图像宽高、坐标、位掩码等非连续调节场景。
 */
export function NumberField({
  label,
  value,
  min,
  max,
  step,
  disabled,
  onChange,
}: NumberFieldProps) {
  return (
    <div className="space-y-1">
      <Label className="text-xs font-medium">{label}</Label>
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
        className="w-full px-2 py-1 text-sm border rounded bg-background"
      />
    </div>
  );
}
