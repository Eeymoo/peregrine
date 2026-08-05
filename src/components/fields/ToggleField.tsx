import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";

/** 开关字段组件 Props。 */
interface ToggleFieldProps {
  /** 字段标签文本，渲染在 switch 左侧。 */
  label: string;
  /** 当前布尔值。 */
  value: boolean;
  /** 是否禁用控件。 */
  disabled?: boolean;
  /** 值变化回调，参数为新布尔值。 */
  onChange: (v: boolean) => void;
}

/** 通用开关字段：label + Switch 同行布局（label 左对齐，switch 右对齐）。
 *
 * 这是统一两行布局的合理例外：toggle 语义为开关，第二行单独放 switch 视觉突兀。
 */
export function ToggleField({ label, value, disabled, onChange }: ToggleFieldProps) {
  return (
    <div className="flex items-center justify-between">
      <Label className="text-xs font-medium">{label}</Label>
      <Switch checked={value} onCheckedChange={onChange} disabled={disabled} />
    </div>
  );
}
