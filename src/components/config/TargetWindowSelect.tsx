import { useI18n } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { listWindowTitles } from "@/lib/api";
import { useEffect, useState } from "react";

interface TargetWindowSelectProps {
  value: string;
  onChange: (value: string) => void;
}

export function TargetWindowSelect({ value, onChange }: TargetWindowSelectProps) {
  const { t } = useI18n();
  const [windows, setWindows] = useState<string[]>([]);

  const refreshWindows = () => {
    listWindowTitles().then(setWindows).catch(console.error);
  };

  useEffect(() => {
    refreshWindows();
  }, []);

  return (
    <div className="space-y-2">
      <div className="flex justify-between items-center">
        <Label className="text-sm">{t("config.targetWindow")}</Label>
        <Button
          variant="ghost"
          size="sm"
          onClick={refreshWindows}
          className="h-8 text-sm px-2"
        >
          {t("config.refresh")}
        </Button>
      </div>
      <Select value={value || "__none__"} onValueChange={onChange}>
        <SelectTrigger className="h-8 text-sm">
          <SelectValue placeholder={t("config.none")} />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="__none__" className="text-sm">
            {t("config.none")}
          </SelectItem>
          {windows.map((w) => (
            <SelectItem key={w} value={w} className="text-sm">
              {w.length > 30 ? w.slice(0, 30) + "…" : w}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
