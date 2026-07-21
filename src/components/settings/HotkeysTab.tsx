import { useI18n } from "@/lib/i18n";
import { Label } from "@/components/ui/label";
import { Kbd } from "@/components/ui/kbd";
import { useState } from "react";
import type { HotkeyAction, HotkeyBindings } from "@/types/config";

/** 所有可绑定的快捷键动作（与 Rust HotkeyAction 枚举一一对应）。 */
export const HOTKEY_ACTIONS: HotkeyAction[] = [
  "toggle_overlay",
  "start_overlay",
  "stop_overlay",
  "cycle_color_next",
  "cycle_color_prev",
  "set_color_1",
  "set_color_2",
  "set_color_3",
  "set_color_4",
  "set_color_5",
];

interface HotkeysTabProps {
  bindings: HotkeyBindings;
  onChange: (bindings: HotkeyBindings) => void;
}

export function HotkeysTab({ bindings, onChange }: HotkeysTabProps) {
  const { t } = useI18n();

  return (
    <div className="space-y-4">
      <div className="space-y-0.5">
        <Label className="text-sm font-medium">{t("hotkeys.title")}</Label>
        <p className="text-xs text-muted-foreground">{t("hotkeys.hint")}</p>
      </div>
      <div className="space-y-1.5">
        {HOTKEY_ACTIONS.map((action) => (
          <HotkeyRow
            key={action}
            action={action}
            bindings={bindings}
            onChange={onChange}
          />
        ))}
      </div>
    </div>
  );
}

/** 快捷键录制行：点击输入框 → 按下组合键捕获 → Esc 清除。 */
function HotkeyRow({
  action,
  bindings,
  onChange,
}: {
  action: HotkeyAction;
  bindings: HotkeyBindings;
  onChange: (bindings: HotkeyBindings) => void;
}) {
  const { t } = useI18n();
  const [recording, setRecording] = useState(false);

  const currentValue = bindings.find(([a]) => a === action)?.[1] ?? "";
  const keyParts = currentValue ? currentValue.split("+") : [];

  const updateBinding = (key: string) => {
    // 移除同 action 的旧绑定和同 key 的其他绑定（避免重复）。
    let next = bindings.filter(([a, k]) => a !== action && k !== key);
    if (key) {
      next = [...next, [action, key] as [HotkeyAction, string]];
    }
    onChange(next);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.key === "Escape") {
      updateBinding("");
      setRecording(false);
      return;
    }
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.shiftKey) parts.push("Shift");
    if (e.altKey) parts.push("Alt");
    if (e.metaKey) parts.push("Super");
    let keyName = e.key;
    if (keyName === " ") keyName = "Space";
    else if (keyName.length === 1) keyName = keyName.toUpperCase();
    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;
    if (parts.length === 0) return;
    parts.push(keyName);
    updateBinding(parts.join("+"));
    setRecording(false);
  };

  return (
    <div className="flex items-center justify-between">
      <Label className="text-sm">{t(`hotkeyActions.${action}`)}</Label>
      {/* 录入区域：点击聚焦进入录入模式，显示 Kbd 标签 */}
      <div
        tabIndex={0}
        onFocus={() => setRecording(true)}
        onBlur={() => setRecording(false)}
        onKeyDown={handleKeyDown}
        className={`flex items-center gap-1 min-h-7 min-w-32 px-2 py-1 rounded-md cursor-pointer transition-colors ${
          recording ? "border border-primary bg-primary/10" : "border border-transparent hover:bg-muted"
        }`}
      >
        {recording ? (
          <span className="text-xs text-muted-foreground animate-pulse">按下组合键…</span>
        ) : keyParts.length > 0 ? (
          keyParts.map((part, i) => (
            <span key={i} className="flex items-center gap-1">
              {i > 0 && <span className="text-xs text-muted-foreground">+</span>}
              <Kbd>{part}</Kbd>
            </span>
          ))
        ) : (
          <span className="text-xs text-muted-foreground">{t("hotkeys.placeholder")}</span>
        )}
      </div>
    </div>
  );
}

export { HotkeyRow };
