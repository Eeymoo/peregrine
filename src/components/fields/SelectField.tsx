import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

/** 下拉选项。 */
export interface SelectOption {
  /** 选项值。 */
  value: string;
  /** 选项显示文本。 */
  label: string;
}

/** 下拉字段组件 Props。 */
interface SelectFieldProps {
  /** 字段标签文本。 */
  label: string;
  /** 当前选中值。 */
  value: string;
  /** 选项列表。 */
  options: SelectOption[];
  /** 是否禁用控件。 */
  disabled?: boolean;
  /** 值变化回调，参数为新值。 */
  onChange: (v: string) => void;
}

/** 通用下拉字段：第一行 label，第二行 shadcn `<Select>`。 */
export function SelectField({
  label,
  value,
  options,
  disabled,
  onChange,
}: SelectFieldProps) {
  return (
    <div className="space-y-1">
      <Label className="text-xs font-medium">{label}</Label>
      <Select value={value} onValueChange={onChange} disabled={disabled}>
        <SelectTrigger className="h-8 text-sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {options.map((opt) => (
            <SelectItem key={opt.value} value={opt.value} className="text-sm">
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
