import { useState } from "react";
import { useI18n } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Separator } from "@/components/ui/separator";
import { Preview } from "@/components/Preview";
import { StyleFields } from "@/components/StyleFields";
import { AutoSwitchDialog } from "@/components/config/AutoSwitchDialog";
import { UpdateDialog } from "@/components/config/UpdateDialog";
import { UpdateProgress } from "@/components/config/UpdateProgress";
import { TargetWindowSelect } from "@/components/config/TargetWindowSelect";
import { useConfigAppState } from "@/hooks/useConfigAppState";
import { useConfigSave } from "@/hooks/useConfigSave";
import { useOverlayActions } from "@/hooks/useOverlayActions";
import { useUpdate } from "@/hooks/useUpdate";
import { updatePreferences, getCurrentWebviewWindow } from "@/lib/api";
import type { CrosshairStyle } from "@/types/config";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const STYLES: CrosshairStyle[] = [
  "edge_rect",
  "cross",
  "large_cross",
  "corner_dots4",
  "corner_dots6",
  "corner_dots8",
  "ring",
  "custom_orb",
  "random_orb",
  "border_frame",
  // "custom_image", // 暂时隐藏，存在问题
  "edge_arrows",
  "grid",
];

export default function ConfigApp() {
  const { t } = useI18n();
  const { config, setConfig, overlayActive, setOverlayActive, loading, version } =
    useConfigAppState();
  const [showAutoSwitchDialog, setShowAutoSwitchDialog] = useState(false);
  const { profile, crosshair, updateCrosshair, updateProfileTargetWindow, colorCss } =
    useConfigSave(config, setConfig);
  const { handleStartOverlay, handleStopOverlay, saveAutoSwitchPreference } =
    useOverlayActions(config, setOverlayActive, () => setShowAutoSwitchDialog(true));

  const {
    updateAvailable,
    updating,
    updateProgress,
    setUpdateAvailable,
    startUpdate,
  } = useUpdate(config);

  const updateSettings = (patch: Parameters<typeof updatePreferences>[0]) => {
    if (!config) return;
    setConfig({ ...config, settings: { ...config.settings, ...patch } });
    updatePreferences(patch).catch(console.error);
  };

  if (loading || !config || !crosshair) {
    return (
      <div className="h-screen flex items-center justify-center text-muted-foreground">
        {t("config.loading")}
      </div>
    );
  }

  return (
    <div className="h-screen flex bg-background text-foreground overflow-hidden">
      {/* 左侧预览 */}
      <div className="flex-1 p-4 min-w-0">
        <Preview crosshair={crosshair} />
      </div>

      {/* 右侧设置面板 */}
      <div className="w-80 border-l bg-card p-4 flex flex-col gap-4 overflow-hidden">
        {/* 顶部固定区：样式 + 公共配置 */}
        <div className="space-y-3 shrink-0">
          <div className="space-y-2">
            <Label className="text-sm">{t("config.style")}</Label>
            <Select
              value={crosshair.style}
              onValueChange={(v) => updateCrosshair({ style: v as CrosshairStyle })}
            >
              <SelectTrigger className="h-8 text-sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {STYLES.map((s) => (
                  <SelectItem key={s} value={s} className="text-sm">
                    {t(`styles.${s}`)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-3">
            <div className="space-y-2">
              <div className="flex justify-between">
                <Label className="text-sm">{t("config.opacity")}</Label>
                <span className="text-sm text-muted-foreground">
                  {crosshair.opacity.toFixed(2)}
                </span>
              </div>
              <Slider
                value={[crosshair.opacity]}
                min={0}
                max={1}
                step={0.01}
                onValueChange={([v]) => updateCrosshair({ opacity: v })}
              />
            </div>

            <div className="flex items-center gap-3">
              <Label className="shrink-0 text-sm">{t("config.color")}</Label>
              <input
                type="color"
                value={colorCss}
                onChange={(e) => {
                  const hex = e.target.value;
                  const r = parseInt(hex.slice(1, 3), 16) / 255;
                  const g = parseInt(hex.slice(3, 5), 16) / 255;
                  const b = parseInt(hex.slice(5, 7), 16) / 255;
                  updateCrosshair({ color: [r, g, b, 1] });
                }}
                className="h-8 w-14 rounded border bg-transparent cursor-pointer"
              />
              <div className="flex gap-1">
                {(config.settings.quick_colors ?? []).map((qc, i) => {
                  const css = `rgb(${Math.round(qc[0] * 255)}, ${Math.round(qc[1] * 255)}, ${Math.round(qc[2] * 255)})`;
                  const isActive =
                    crosshair.color[0] === qc[0] &&
                    crosshair.color[1] === qc[1] &&
                    crosshair.color[2] === qc[2];
                  return (
                    <button
                      key={i}
                      type="button"
                      title={css}
                      onClick={() => updateCrosshair({ color: [...qc] })}
                      className="w-5 h-5 rounded-full border-2 transition-colors"
                      style={{
                        backgroundColor: css,
                        borderColor: isActive
                          ? "hsl(var(--primary))"
                          : "hsl(var(--border))",
                      }}
                    />
                  );
                })}
              </div>
            </div>
          </div>
        </div>

        <Separator className="shrink-0" />

        {/* 中间样式配置 */}
        <div className="flex-1 min-h-0 overflow-y-auto pr-1">
          <StyleFields crosshair={crosshair} onChange={updateCrosshair} />
        </div>

        <Separator className="shrink-0" />

        {/* 底部固定区 */}
        <div className="space-y-3 shrink-0">
          <div className="flex items-center gap-2">
            <Checkbox
              id="window-mode"
              checked={!config.settings.fullscreen_overlay}
              onCheckedChange={(v) => updateSettings({ fullscreen_overlay: !v })}
            />
            <Label htmlFor="window-mode" className="text-sm cursor-pointer">
              {t("overlaySettings.windowMode")}
            </Label>
          </div>

          {!config.settings.fullscreen_overlay && (
            <TargetWindowSelect
              value={profile?.target_window ?? ""}
              onChange={(v) => updateProfileTargetWindow(v === "__none__" ? "" : v)}
            />
          )}

          <div>
            {overlayActive ? (
              <Button
                variant="destructive"
                className="w-full h-8 text-sm"
                onClick={handleStopOverlay}
              >
                ■ {t("config.stopOverlay")}
              </Button>
            ) : (
              <Button
                className="w-full h-8 text-sm"
                onClick={handleStartOverlay}
                disabled={
                  !config.settings.fullscreen_overlay && !profile?.target_window
                }
              >
                ▶ {t("config.startOverlay")}
              </Button>
            )}
          </div>

          <div className="text-xs text-muted-foreground text-right">
            Peregrine v{version || "..."}
          </div>
        </div>
      </div>

      {showAutoSwitchDialog && (
        <AutoSwitchDialog
          onConfirm={(remember) => {
            setShowAutoSwitchDialog(false);
            if (remember) {
              saveAutoSwitchPreference("yes", profile?.target_window);
            } else if (profile?.target_window) {
              getCurrentWebviewWindow().destroy().catch(() => {});
            }
          }}
          onCancel={(remember) => {
            setShowAutoSwitchDialog(false);
            if (remember) {
              saveAutoSwitchPreference("no");
            }
          }}
          onCloseByEsc={handleStopOverlay}
        />
      )}

      {updateAvailable && !updating && (
        <UpdateDialog
          version={updateAvailable.version}
          body={updateAvailable.body}
          onUpdate={() => {
            setUpdateAvailable(null);
            startUpdate();
          }}
          onLater={() => setUpdateAvailable(null)}
        />
      )}

      {updating && <UpdateProgress progress={updateProgress} />}
    </div>
  );
}
