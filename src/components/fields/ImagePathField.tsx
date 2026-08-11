import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { pickImagePath } from "@/lib/api";
import { useI18n } from "@/lib/i18n";

/** 图片路径字段组件 Props。 */
interface ImagePathFieldProps {
  /** 字段标签文本。 */
  label: string;
  /** 当前路径值。 */
  value: string;
  /** 占位符文本。 */
  placeholder?: string;
  /** 是否禁用控件。 */
  disabled?: boolean;
  /** 值变化回调，参数为新路径。 */
  onChange: (v: string) => void;
}

/** 通用图片路径字段：第一行 label，第二行文本输入框 + "浏览" 按钮。
 *
 * 点击"浏览"按钮调用 IPC `pick_image_path` 打开文件选择对话框，
 * 用户选择成功后通过 onChange 回填路径。
 */
export function ImagePathField({
  label,
  value,
  placeholder,
  disabled,
  onChange,
}: ImagePathFieldProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-1">
      <Label className="text-xs font-medium">{label}</Label>
      <div className="flex gap-2">
        <input
          type="text"
          value={value}
          placeholder={placeholder}
          disabled={disabled}
          onChange={(e) => onChange(e.target.value)}
          className="flex-1 px-2 py-1 text-sm border rounded bg-background"
        />
        <Button
          size="sm"
          variant="outline"
          disabled={disabled}
          onClick={async () => {
            try {
              const path = await pickImagePath();
              if (path) onChange(path);
            } catch {
              // 用户取消或调用失败，invoke 包装负责 toast。
            }
          }}
        >
          {t("fields.browse")}
        </Button>
      </div>
    </div>
  );
}
