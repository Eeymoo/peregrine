import { useEffect, useState, useCallback, useMemo } from "react";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Checkbox } from "@/components/ui/checkbox";
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
  focusTargetWindow,
  updatePreferences,
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
  // "custom_image", // 暂时隐藏，存在问题
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
  const [showAutoSwitchDialog, setShowAutoSwitchDialog] = useState(false);
  const [rememberChoice, setRememberChoice] = useState(false);

  useEffect(() => {
    getConfig()
      .then(setConfig)
      .catch(console.error)
      .finally(() => setLoading(false));
    refreshWindows();
  }, []);

  /** 监听后端 settings 变更（来自设置窗口），同步 React state。 */
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<{ auto_switch_on_overlay?: string; locale?: string }>(
          "peregrine:settings-changed",
          (event) => {
            const { auto_switch_on_overlay } = event.payload;
            if (auto_switch_on_overlay !== undefined) {
              setConfig((prev) => {
                if (!prev) return prev;
                return { ...prev, settings: { ...prev.settings, auto_switch_on_overlay } };
              });
            }
          },
        );
      } catch { /* 非 Tauri 环境忽略 */ }
    })();
    return () => unlisten?.();
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

  /** 更新应用级偏好设置（仅更新指定字段，不覆盖整个配置）。 */
  const updateSettings = useCallback((patch: Partial<AppConfig["settings"]>) => {
    if (!config) return;
    const newConfig = {
      ...config,
      settings: { ...config.settings, ...patch },
    };
    setConfig(newConfig);
    updatePreferences(patch).catch(console.error);
  }, [config]);

  /** 隐藏配置窗口并切换焦点到目标游戏窗口。 */
  const hideAndSwitch = useCallback(async (targetWindow: string) => {
    await getCurrentWebviewWindow().hide();
    focusTargetWindow(targetWindow).catch(console.error);
  }, []);

  const handleStartOverlay = useCallback(async () => {
    if (!profile?.target_window) return;
    try {
      await startOverlay(profile.target_window);
      setOverlayActive(true);

      const pref = config?.settings.auto_switch_on_overlay ?? "ask";
      if (pref === "yes") {
        await hideAndSwitch(profile.target_window);
      } else if (pref === "no") {
        // 不隐藏，不做操作。
      } else {
        // 未设置偏好（ask），弹出对话框。
        setShowAutoSwitchDialog(true);
      }
    } catch (e) {
      console.error(e);
    }
  }, [profile?.target_window, config?.settings.auto_switch_on_overlay, hideAndSwitch]);

  /** 对话框确认：隐藏配置窗口并切换焦点，同时按勾选状态保存偏好。 */
  const handleDialogConfirm = useCallback(async () => {
    if (rememberChoice) {
      updateSettings({ auto_switch_on_overlay: "yes" });
    }
    setShowAutoSwitchDialog(false);
    if (profile?.target_window) {
      await hideAndSwitch(profile.target_window);
    }
  }, [rememberChoice, profile?.target_window, hideAndSwitch, updateSettings]);

  /** 对话框取消：保持配置窗口显示，同时按勾选状态保存偏好。 */
  const handleDialogCancel = useCallback(async () => {
    if (rememberChoice) {
      updateSettings({ auto_switch_on_overlay: "no" });
    }
    setShowAutoSwitchDialog(false);
  }, [rememberChoice, updateSettings]);

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
              <Button className="w-full h-8 text-sm" onClick={handleStartOverlay} disabled={!profile.target_window}>
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

      {/* 自动切换确认对话框 */}
      {showAutoSwitchDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
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
              <Button variant="outline" size="sm" onClick={handleDialogCancel}>
                {t("overlay.keepConfig")}
              </Button>
              <Button size="sm" onClick={handleDialogConfirm}>
                {t("overlay.switchToGame")}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
