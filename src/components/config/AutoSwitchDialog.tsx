import { useState } from "react";
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { useI18n } from "@/lib/i18n";

interface AutoSwitchDialogProps {
  onConfirm: (remember: boolean) => void;
  onCancel: (remember: boolean) => void;
  onCloseByEsc: () => void;
}

export function AutoSwitchDialog({
  onConfirm,
  onCancel,
  onCloseByEsc,
}: AutoSwitchDialogProps) {
  const { t } = useI18n();
  const [rememberChoice, setRememberChoice] = useState(false);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          onCloseByEsc();
        }
      }}
      tabIndex={-1}
      autoFocus
    >
      <div className="bg-card border rounded-lg shadow-lg p-6 max-w-sm w-full mx-4 space-y-4">
        <h2 className="text-base font-semibold">{t("overlay.autoSwitchTitle")}</h2>
        <p className="text-sm text-muted-foreground">{t("overlay.autoSwitchDesc")}</p>
        <div className="flex items-center gap-2">
          <Checkbox
            id="remember-choice"
            checked={rememberChoice}
            onCheckedChange={(v) => setRememberChoice(v === true)}
          />
          <Label htmlFor="remember-choice" className="text-sm cursor-pointer">
            {t("overlay.rememberChoice")}
          </Label>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button variant="outline" size="sm" onClick={() => onCancel(rememberChoice)}>
            {t("overlay.keepConfig")}
          </Button>
          <Button size="sm" onClick={() => onConfirm(rememberChoice)}>
            {t("overlay.switchToGame")}
          </Button>
        </div>
      </div>
    </div>
  );
}
