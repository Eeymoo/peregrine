import { Label } from "@/components/ui/label";

/** 文本字段组件 Props。 */
interface TextFieldProps {
  /** 字段标签文本。 */
  label: string;
  /** 当前文本值。 */
  value: string;
  /** 占位符文本。 */
  placeholder?: string;
  /** 是否禁用控件。 */
  disabled?: boolean;
  /** 文本变化回调，参数为新值。 */
  onChange: (v: string) => void;
}

/** 通用文本字段：第一行 label，第二行纯 `<input type="text">`。 */
export function TextField({
  label,
  value,
  placeholder,
  disabled,
  onChange,
}: TextFieldProps) {
  return (
    <div className="space-y-1">
      <Label className="text-xs font-medium">{label}</Label>
      <input
        type="text"
        value={value}
        placeholder={placeholder}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
        className="w-full px-2 py-1 text-sm border rounded bg-background"
      />
    </div>
  );
}
