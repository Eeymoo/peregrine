import { useI18n } from "@/lib/i18n";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { updatePreferences } from "@/lib/api";
import type { AppConfig } from "@/types/config";

interface OverlayTabProps {
  config: AppConfig | null;
  autoSwitch: string;
  onAutoSwitchChange: (value: string) => void;
  setConfig: (cfg: AppConfig) => void;
}

export function OverlayTab({
  config,
  autoSwitch,
  onAutoSwitchChange,
  setConfig,
}: OverlayTabProps) {
  const { t } = useI18n();

  const updateSetting = <K extends keyof AppConfig["settings"]>(
    key: K,
    value: AppConfig["settings"][K]
  ) => {
    if (!config) return;
    const newConfig: AppConfig = {
      ...config,
      settings: { ...config.settings, [key]: value },
    };
    setConfig(newConfig);
    updatePreferences({ [key]: value } as Partial<AppConfig["settings"]>).catch(console.error);
  };

  return (
    <div className="space-y-6">
      {/* 启动覆盖时的行为 */}
      <div className="flex items-start justify-between gap-4">
        <div className="space-y-0.5">
          <Label className="text-sm font-medium">{t("settings.autoSwitch")}</Label>
          <p className="text-xs text-muted-foreground">{t("settings.autoSwitchHint")}</p>
        </div>
        <Select value={autoSwitch} onValueChange={onAutoSwitchChange}>
          <SelectTrigger className="w-44 h-8 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="ask">{t("settings.autoSwitchAsk")}</SelectItem>
            <SelectItem value="yes">{t("overlay.switchYes")}</SelectItem>
            <SelectItem value="no">{t("overlay.switchNo")}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* 拖拽时实时显示 */}
      <div className="flex items-center justify-between gap-4">
        <div className="space-y-0.5">
          <Label className="text-sm font-medium">
            {t("overlaySettings.liveDragPreview")}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t("overlaySettings.liveDragPreviewHint")}
          </p>
        </div>
        <Switch
          checked={config?.settings?.live_drag_preview ?? false}
          onCheckedChange={(v) => updateSetting("live_drag_preview", v)}
        />
      </div>

      {/* 抗锯齿 */}
      <div className="flex items-center justify-between gap-4">
        <div className="space-y-0.5">
          <Label className="text-sm font-medium">
            {t("overlaySettings.antialiasing")}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t("overlaySettings.antialiasingHint")}
          </p>
        </div>
        <Switch
          checked={config?.settings?.antialiasing ?? true}
          onCheckedChange={(v) => updateSetting("antialiasing", v)}
        />
      </div>

      {/* 渲染后端 */}
      <div className="flex items-center justify-between gap-4">
        <div className="space-y-0.5">
          <Label className="text-sm font-medium">
            {t("overlaySettings.rendererBackend")}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t("overlaySettings.rendererBackendHint")}
          </p>
        </div>
        <Select
          value={config?.settings?.renderer_backend ?? "cpu"}
          onValueChange={(v) => updateSetting("renderer_backend", v as "cpu" | "svg")}
        >
          <SelectTrigger className="w-32 h-8 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="cpu">{t("overlaySettings.rendererBackendCpu")}</SelectItem>
            <SelectItem value="svg">{t("overlaySettings.rendererBackendSvg")}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* 快捷颜色 */}
      <div className="space-y-2">
        <Label className="text-sm font-medium">{t("quickColors.title")}</Label>
        <p className="text-xs text-muted-foreground">{t("quickColors.hint")}</p>
        <div className="flex gap-3 pt-1">
          {(config?.settings?.quick_colors ?? [
            [1, 1, 1, 1], [0, 1, 0, 1], [0.2, 0.5, 1, 1], [1, 0, 0, 1], [1, 0.5, 0, 1],
          ]).map((qc, i) => {
            const css = `rgb(${Math.round(qc[0] * 255)}, ${Math.round(qc[1] * 255)}, ${Math.round(qc[2] * 255)})`;
            const hex = `#${Math.round(qc[0] * 255).toString(16).padStart(2, "0")}${Math.round(qc[1] * 255).toString(16).padStart(2, "0")}${Math.round(qc[2] * 255).toString(16).padStart(2, "0")}`;
            return (
              <div key={i} className="flex flex-col items-center gap-1">
                <input
                  type="color"
                  value={hex}
                  onChange={(e) => {
                    if (!config) return;
                    const h = e.target.value;
                    const r = parseInt(h.slice(1, 3), 16) / 255;
                    const g = parseInt(h.slice(3, 5), 16) / 255;
                    const b = parseInt(h.slice(5, 7), 16) / 255;
                    const newColors = [...(config.settings.quick_colors ?? [])];
                    newColors[i] = [r, g, b, 1];
                    updateSetting("quick_colors", newColors);
                  }}
                  className="w-8 h-8 rounded-full cursor-pointer border-2"
                  style={{ backgroundColor: css }}
                />
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
