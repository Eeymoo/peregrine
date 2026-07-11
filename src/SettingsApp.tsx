import { useEffect, useState, useCallback } from "react";
import { useI18n, LANGUAGE_OPTIONS, type Locale } from "@/lib/i18n";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Separator } from "@/components/ui/separator";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { getConfig, updatePreferences, getAppVersion, relaunchApp, checkForUpdate, downloadAndInstallUpdate } from "@/lib/api";
import type { AppConfig } from "@/types/config";
import { Checkbox } from "@/components/ui/checkbox";

export default function SettingsApp() {
  const { t, locale, setLocale } = useI18n();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [autoSwitch, setAutoSwitchState] = useState<string>("ask");
  const [version, setVersion] = useState("");
  const [updateState, setUpdateState] = useState<{ status: "idle" | "checking" | "available" | "upToDate" | "updating" | "failed"; version?: string; body?: string }>({ status: "idle" });

  useEffect(() => {
    getCurrentWebviewWindow().setTitle(`${t("app.title")} ${t("settings.title")}`).catch(() => {});
  }, [t]);

  // 初始化时读取配置。
  useEffect(() => {
    getConfig()
      .then((cfg) => {
        setConfig(cfg);
        setAutoSwitchState(cfg.settings?.auto_switch_on_overlay ?? "ask");
      })
      .catch(console.error);
    getAppVersion().then(setVersion).catch(() => {});
  }, []);

  /** 监听后端 settings 变更（来自托盘或配置窗口），同步本窗口状态。 */
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<{
          auto_switch_on_overlay?: string;
          locale?: string;
          fullscreen_overlay?: boolean;
          live_drag_preview?: boolean;
          gpu_acceleration?: boolean;
          update_channel?: string;
        }>("peregrine:settings-changed", (event) => {
          const { auto_switch_on_overlay, fullscreen_overlay, live_drag_preview, gpu_acceleration, update_channel } = event.payload;
          if (auto_switch_on_overlay !== undefined) {
            setAutoSwitchState(auto_switch_on_overlay);
          }
          setConfig((prev) => {
            if (!prev) return prev;
            const settings = { ...prev.settings };
            if (auto_switch_on_overlay !== undefined) {
              settings.auto_switch_on_overlay = auto_switch_on_overlay;
            }
            if (fullscreen_overlay !== undefined) {
              settings.fullscreen_overlay = fullscreen_overlay;
            }
            if (live_drag_preview !== undefined) {
              settings.live_drag_preview = live_drag_preview;
            }
            if (gpu_acceleration !== undefined) {
              settings.gpu_acceleration = gpu_acceleration;
            }
            if (update_channel !== undefined) {
              settings.update_channel = update_channel;
            }
            return { ...prev, settings };
          });
        });
      } catch { /* 非 Tauri 环境忽略 */ }
    })();
    return () => unlisten?.();
  }, []);

  const handleAutoSwitchChange = useCallback((value: string) => {
    setAutoSwitchState(value);
    if (!config) return;
    const newConfig: AppConfig = {
      ...config,
      settings: { ...config.settings, auto_switch_on_overlay: value },
    };
    setConfig(newConfig);
    updatePreferences({ auto_switch_on_overlay: value }).catch(console.error);
  }, [config]);

  return (
    <div className="h-screen flex flex-col bg-background text-foreground p-6">
      <h1 className="text-xl font-semibold">{t("settings.title")}</h1>

      <Separator className="my-4" />

      {/* 语言设置 */}
      <div className="space-y-2">
        <Label className="text-sm">{t("settings.language")}</Label>
        <Select value={locale} onValueChange={(v) => setLocale(v as Locale)}>
          <SelectTrigger className="h-8 text-sm">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {LANGUAGE_OPTIONS.map((opt) => (
              <SelectItem key={opt.value} value={opt.value} className="text-sm">
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <Separator className="my-4" />

      {/* 开始覆盖时自动切换设置 */}
      <div className="space-y-2">
        <Label className="text-sm">{t("settings.autoSwitch")}</Label>
        <p className="text-xs text-muted-foreground">{t("settings.autoSwitchHint")}</p>
        <Select value={autoSwitch} onValueChange={handleAutoSwitchChange}>
          <SelectTrigger className="h-8 text-sm">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="ask" className="text-sm">{t("settings.autoSwitchAsk")}</SelectItem>
            <SelectItem value="yes" className="text-sm">{t("overlay.switchYes")}</SelectItem>
            <SelectItem value="no" className="text-sm">{t("overlay.switchNo")}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <Separator className="my-4" />

      {/* 拖拽时实时显示（仅窗口模式生效） */}
      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <Checkbox
            id="live-drag-preview"
            checked={config?.settings?.live_drag_preview ?? false}
            disabled={config?.settings?.fullscreen_overlay ?? true}
            onCheckedChange={(v) => {
              if (!config) return;
              const newConfig: AppConfig = {
                ...config,
                settings: { ...config.settings, live_drag_preview: v === true },
              };
              setConfig(newConfig);
              updatePreferences({ live_drag_preview: v === true }).catch(console.error);
            }}
          />
          <Label htmlFor="live-drag-preview" className="text-sm cursor-pointer">
            {t("overlaySettings.liveDragPreview")}
          </Label>
        </div>
      </div>

      <Separator className="my-4" />

      {/* GPU 硬件加速（重启应用后生效） */}
      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <Checkbox
            id="gpu-acceleration"
            checked={config?.settings?.gpu_acceleration ?? false}
            onCheckedChange={async (v) => {
              if (!config) return;
              const newValue = v === true;
              const newConfig: AppConfig = {
                ...config,
                settings: { ...config.settings, gpu_acceleration: newValue },
              };
              setConfig(newConfig);
              await updatePreferences({ gpu_acceleration: newValue });
              // GPU 加速设置变更后，弹窗询问是否立即重启。
              try {
                const { ask } = await import("@tauri-apps/plugin-dialog");
                const confirmed = await ask(t("settings.gpuRestartDesc"), {
                  title: t("settings.gpuRestartTitle"),
                  okLabel: t("settings.gpuRestartNow"),
                  cancelLabel: t("settings.gpuRestartLater"),
                  kind: "info",
                });
                if (confirmed) {
                  await relaunchApp();
                }
              } catch (e) { console.error("[GPU] dialog/relaunch failed:", e); }
            }}
          />
          <Label htmlFor="gpu-acceleration" className="text-sm cursor-pointer">
            {t("settings.gpuAcceleration")}
          </Label>
        </div>
        <p className="text-xs text-muted-foreground">
          {t("settings.gpuAccelerationHint")}
        </p>
      </div>

      <Separator className="my-4" />

      {/* 更新通道 + 检查更新 */}
      <div className="space-y-2">
        <Label className="text-sm">{t("settings.updateChannel")}</Label>
        <Select
          value={config?.settings?.update_channel ?? "stable"}
          onValueChange={(v) => {
            if (!config) return;
            const newConfig: AppConfig = {
              ...config,
              settings: { ...config.settings, update_channel: v },
            };
            setConfig(newConfig);
            updatePreferences({ update_channel: v }).catch(console.error);
          }}
        >
          <SelectTrigger className="h-8 text-sm">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="stable" className="text-sm">{t("settings.updateChannelStable")}</SelectItem>
            <SelectItem value="prerelease" className="text-sm">{t("settings.updateChannelPrerelease")}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <Separator className="my-4" />

      <div className="space-y-3">
        <h2 className="text-lg font-medium">{t("settings.about.title")}</h2>
        <p className="text-sm text-muted-foreground">
          {t("settings.about.description")}
        </p>
        <ul className="text-sm space-y-1 text-muted-foreground">
          <li>{t("settings.about.version")}：{version || "..."}</li>
          <li>{t("settings.about.publisher")}：Eeymoo</li>
          <li>{t("settings.about.license")}：{t("license.mit")}</li>
          <li>{t("settings.about.repository")}：https://github.com/Eeymoo/peregrine</li>
        </ul>

        {/* 检查更新按钮 */}
        <Button
          variant="outline"
          size="sm"
          disabled={updateState.status === "checking" || updateState.status === "updating"}
          onClick={async () => {
            setUpdateState({ status: "checking" });
            try {
              const channel = config?.settings?.update_channel ?? "stable";
              const result = await checkForUpdate(channel);
              if (result.available) {
                setUpdateState({ status: "available", version: result.version, body: result.body });
              } else {
                setUpdateState({ status: "upToDate" });
              }
            } catch (e) {
              console.error("[Update] check failed:", e);
              setUpdateState({ status: "failed" });
            }
          }}
        >
          {updateState.status === "checking" ? "..." : t("settings.checkUpdate")}
        </Button>

        {/* 更新状态提示 */}
        {updateState.status === "upToDate" && (
          <p className="text-xs text-green-600">{t("settings.updateUpToDate")}</p>
        )}
        {updateState.status === "failed" && (
          <p className="text-xs text-red-500">{t("settings.updateFailed")}</p>
        )}
        {updateState.status === "updating" && (
          <p className="text-xs text-blue-500">{t("settings.updating")}</p>
        )}

        {/* 发现新版本对话框 */}
        {updateState.status === "available" && (
          <div className="border rounded-lg p-3 space-y-2 bg-muted/50">
            <p className="text-sm font-medium">
              {t("settings.updateAvailable")}：v{updateState.version}
            </p>
            {updateState.body && (
              <p className="text-xs text-muted-foreground whitespace-pre-wrap line-clamp-6">
                {updateState.body}
              </p>
            )}
            <div className="flex gap-2">
              <Button
                size="sm"
                onClick={async () => {
                  setUpdateState((s) => ({ ...s, status: "updating" }));
                  try {
                    await downloadAndInstallUpdate();
                  } catch (e) {
                    console.error("[Update] download failed:", e);
                    setUpdateState({ status: "failed" });
                  }
                }}
              >
                {t("settings.updateNow")}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setUpdateState({ status: "idle" })}
              >
                {t("settings.updateLater")}
              </Button>
            </div>
          </div>
        )}
      </div>

      <div className="mt-auto text-xs text-muted-foreground text-right">
        Peregrine v{version || "..."}
      </div>
    </div>
  );
}
