import { useEffect, useState, useCallback, useMemo } from "react";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Preview } from "@/components/Preview";
import { StyleFields } from "@/components/StyleFields";
import { useI18n } from "@/lib/i18n";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  getConfig,
  saveConfig,
  listWindowTitles,
  startOverlay,
  stopOverlay,
} from "@/lib/api";
import type { AppConfig, Crosshair, CrosshairStyle } from "@/types/config";

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
  "custom_image",
  "edge_arrows",
];

export default function ConfigApp() {
  const { t } = useI18n();

  useEffect(() => {
    getCurrentWebviewWindow().setTitle(`${t("app.title")} ${t("config.title")}`).catch(() => {});
  }, [t]);

  const [config, setConfig] = useState<AppConfig | null>(null);
  const [windows, setWindows] = useState<string[]>([]);
  const [overlayActive, setOverlayActive] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getConfig()
      .then(setConfig)
      .catch(console.error)
      .finally(() => setLoading(false));
    refreshWindows();
  }, []);

  const profile = config?.profiles[config.active_profile];
  const crosshair = profile?.crosshair;

  const updateCrosshair = useCallback((patch: Partial<Crosshair>) => {
    if (!config || !profile || !crosshair) return;
    const newCrosshair = { ...crosshair, ...patch };
    const newProfile = { ...profile, crosshair: newCrosshair };
    const newConfig = {
      ...config,
      profiles: { ...config.profiles, [config.active_profile]: newProfile },
    };
    setConfig(newConfig);
    saveConfig(newConfig).catch(console.error);
  }, [config, profile, crosshair]);

  const refreshWindows = useCallback(() => {
    listWindowTitles().then(setWindows).catch(console.error);
  }, []);

  const handleStartOverlay = useCallback(async () => {
    if (!profile?.target_window) return;
    try {
      await startOverlay(profile.target_window);
      setOverlayActive(true);
    } catch (e) {
      console.error(e);
    }
  }, [profile?.target_window]);

  const handleStopOverlay = useCallback(async () => {
    try {
      await stopOverlay();
      setOverlayActive(false);
    } catch (e) {
      console.error(e);
    }
  }, []);

  const colorCss = useMemo(() => {
    const [r, g, b] = crosshair?.color || [1, 1, 1];
    return `rgb(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`;
  }, [crosshair?.color]);

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

      {/* 右侧设置面板：固定顶部与底部，中间样式配置自适应 */}
      <div className="w-80 border-l bg-card p-4 flex flex-col gap-4 overflow-hidden">
        {/* 顶部固定区：样式 + 公共配置 */}
        <div className="space-y-3 shrink-0">
          {/* 样式选择 */}
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

          {/* 公共配置 */}
          <div className="space-y-3">
            <div className="space-y-2">
              <div className="flex justify-between">
                <Label className="text-sm">{t("config.opacity")}</Label>
                <span className="text-sm text-muted-foreground">{crosshair.opacity.toFixed(2)}</span>
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
            </div>
          </div>
        </div>

        <Separator className="shrink-0" />

        {/* 中间样式配置：默认随窗口高度自适应，内容过多时才滚动 */}
        <div className="flex-1 min-h-0 overflow-y-auto pr-1">
          <StyleFields crosshair={crosshair} onChange={updateCrosshair} />
        </div>

        <Separator className="shrink-0" />

        {/* 底部固定区：目标窗口 + 开始/停止覆盖 */}
        <div className="space-y-3 shrink-0">
          {/* 目标窗口 */}
          <div className="space-y-2">
            <div className="flex justify-between items-center">
              <Label className="text-sm">{t("config.targetWindow")}</Label>
              <Button variant="ghost" size="sm" onClick={refreshWindows} className="h-8 text-sm px-2">
                {t("config.refresh")}
              </Button>
            </div>
            <Select
              value={profile.target_window || "__none__"}
              onValueChange={(v) => {
                const newConfig = {
                  ...config,
                  profiles: {
                    ...config.profiles,
                    [config.active_profile]: {
                      ...profile,
                      target_window: v === "__none__" ? "" : v,
                    },
                  },
                };
                setConfig(newConfig);
                saveConfig(newConfig).catch(console.error);
              }}
            >
              <SelectTrigger className="h-8 text-sm">
                <SelectValue placeholder={t("config.none")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__none__" className="text-sm">{t("config.none")}</SelectItem>
                {windows.map((w) => (
                  <SelectItem key={w} value={w} className="text-sm">
                    {w.length > 30 ? w.slice(0, 30) + "…" : w}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* 开始/停止覆盖 */}
          <div>
            {overlayActive ? (
              <Button variant="destructive" className="w-full h-8 text-sm" onClick={handleStopOverlay}>
                ■ {t("config.stopOverlay")}
              </Button>
            ) : (
              <Button className="w-full h-8 text-sm" onClick={handleStartOverlay}>
                ▶ {t("config.startOverlay")}
              </Button>
            )}
          </div>

          {/* 底部信息 */}
          <div className="text-xs text-muted-foreground text-right">
            Peregrine v0.1.1
          </div>
        </div>
      </div>
    </div>
  );
}
