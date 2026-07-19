import { useEffect, useState, useCallback, useMemo, useRef } from "react";
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
import { LayersEditor } from "@/components/LayersEditor";
import { DeveloperPanel } from "@/components/DeveloperPanel";
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
  getAppVersion,
  getOverlayActive,
  checkForUpdate,
  downloadAndInstallUpdate,
} from "@/lib/api";
import { getDefaultCrosshairForStyle } from "@/lib/presets";
import type { AppConfig, Crosshair, CrosshairStyle, Layer, LayerStyle, MaterialRef, Anchor, RingStyle, BorderFrameStyle, GridAlignment } from "@/types/config";

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

  useEffect(() => {
    getCurrentWebviewWindow().setTitle(`${t("app.title")} ${t("config.title")}`).catch(() => {});
  }, [t]);

  const [config, setConfig] = useState<AppConfig | null>(null);
  const [windows, setWindows] = useState<string[]>([]);
  const [overlayActive, setOverlayActive] = useState(false);
  const [loading, setLoading] = useState(true);
  const [showAutoSwitchDialog, setShowAutoSwitchDialog] = useState(false);
  const [rememberChoice, setRememberChoice] = useState(false);
  const [version, setVersion] = useState("");
  // 点击版本号 3 次后启用"开发者"tab，写入 localStorage 持久化。
  const [devTabUnlocked, setDevTabUnlocked] = useState<boolean>(
    () => localStorage.getItem("peregrine:dev-tab") === "1",
  );
  const [versionClickCount, setVersionClickCount] = useState(0);
  const [updateAvailable, setUpdateAvailable] = useState<{ version: string; body?: string } | null>(null);
  // 默认显示旧版准心 UI（单图层模式），点击顶部按钮可切换到图层编辑器。
  const [layersMode, setLayersMode] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [updateProgress, setUpdateProgress] = useState(0);

  useEffect(() => {
    getConfig()
      .then(setConfig)
      .catch(console.error)
      .finally(() => setLoading(false));
    refreshWindows();
    getAppVersion().then(setVersion).catch(() => {});
    getOverlayActive().then(setOverlayActive).catch(() => {});

    // 启动时自动检测更新（静默，发现新版本才弹窗）。延迟 3 秒避免抢焦点。
    const autoCheck = async () => {
      try {
        await new Promise((r) => setTimeout(r, 3000));
        const cfg = await getConfig();
        const channel = cfg.settings?.update_channel ?? "stable";
        const cnMirror = cfg.settings?.cn_mirror ?? false;
        const mirrorUrl = cfg.settings?.mirror_url ?? "https://v4.gh-proxy.org";
        const result = await checkForUpdate(channel, cnMirror, mirrorUrl);
        if (result.available) {
          setUpdateAvailable({ version: result.version || "", body: result.body });
        }
      } catch { /* 静默失败 */ }
    };
    autoCheck();
  }, []);

  // 点击版本号 3 次（连续，间隔 < 1.5s）解锁开发者 tab。
  // 解锁状态写入 localStorage，下次启动仍然有效；可在开发者 tab 里关闭。
  useEffect(() => {
    if (versionClickCount === 0) return;
    const timer = setTimeout(() => setVersionClickCount(0), 1500);
    if (versionClickCount >= 3) {
      setVersionClickCount(0);
      setDevTabUnlocked(true);
      localStorage.setItem("peregrine:dev-tab", "1");
    }
    return () => clearTimeout(timer);
  }, [versionClickCount]);

  /** 监听后端 settings 变更（来自托盘或设置窗口），同步 React state。 */
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
          cn_mirror?: boolean;
          mirror_url?: string;
          update_channel?: string;
          antialiasing?: boolean;
          renderer_backend?: "cpu" | "svg";
          quick_colors?: [number, number, number, number][];
          hotkey_bindings?: [string, string][];
        }>("peregrine:settings-changed", (event) => {
          const { auto_switch_on_overlay, fullscreen_overlay, live_drag_preview, cn_mirror, mirror_url, update_channel, antialiasing, renderer_backend, quick_colors, hotkey_bindings } = event.payload;
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
            if (cn_mirror !== undefined) {
              settings.cn_mirror = cn_mirror;
            }
            if (mirror_url !== undefined) {
              settings.mirror_url = mirror_url;
            }
            if (update_channel !== undefined) {
              settings.update_channel = update_channel;
            }
            if (antialiasing !== undefined) {
              settings.antialiasing = antialiasing;
            }
            if (renderer_backend !== undefined) {
              settings.renderer_backend = renderer_backend;
            }
            if (quick_colors !== undefined) {
              settings.quick_colors = quick_colors;
            }
            if (hotkey_bindings !== undefined) {
              settings.hotkey_bindings = hotkey_bindings as any;
            }
            return { ...prev, settings };
          });
        });
      } catch { /* 非 Tauri 环境忽略 */ }
    })();
    return () => unlisten?.();
  }, []);

  const profile = config?.profiles[config.active_profile];
  // 当后端配置是纯净新格式（crosshair 为 null）时，从 layers[0] 反向生成一个 crosshair，
  // 保证旧版单图层 UI 始终可用。后续编辑 crosshair 会同步回写 layers[0]。
  const derivedCrosshair = useMemo<Crosshair | null>(() => {
    const raw = profile?.crosshair ?? null;
    if (raw) return raw;
    const layer = profile?.layers?.[0];
    if (!layer) return null;
    return layerToCrosshair(layer);
  }, [profile]);
  const crosshair = derivedCrosshair;
  const hasLayers = (profile?.layers?.length ?? 0) > 0;

  /** 防抖保存配置：拖滑块等连续操作时只在停止后 300ms 写入一次。 */
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const debouncedSave = useCallback((cfg: AppConfig) => {
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    saveTimerRef.current = setTimeout(() => {
      saveConfig(cfg).catch(console.error);
    }, 300);
  }, []);

  /**
   * 把旧版 Crosshair 同步映射到 layers[0]。
   * 这样旧 UI 编辑 crosshair 时，底层渲染仍然走 layers。
   */
  const syncCrosshairToLayer = useCallback((crosshair: Crosshair, layer: Layer): Layer => {
    const material = crosshairToMaterial(crosshair.style);
    const params = crosshairToParams(crosshair);
    const style: LayerStyle = {
      color: crosshair.color,
      opacity: crosshair.opacity,
      blend_mode: layer.style.blend_mode,
    };
    return {
      ...layer,
      name: layer.name || t(`styles.${crosshair.style}`),
      material,
      params,
      style,
        transform: layer.transform ?? { offset_x: 0, offset_y: 0, scale: 1, rotation_deg: 0 },
    };
  }, [t]);

  const updateCrosshair = useCallback((patch: Partial<Crosshair>, options?: { resetDefaults?: boolean }) => {
    if (!config || !profile) return;
    const currentCrosshair = crosshair;
    const newCrosshair = options?.resetDefaults && patch.style !== undefined
      ? getDefaultCrosshairForStyle(patch.style)
      : currentCrosshair
        ? { ...currentCrosshair, ...patch }
        : getDefaultCrosshairForStyle(patch.style ?? "cross");

    const newProfile = { ...profile, crosshair: newCrosshair };
    // 如果已有 layers，同步更新 layers[0] 使其与 crosshair 一致。
    if (newProfile.layers.length > 0) {
      newProfile.layers = [syncCrosshairToLayer(newCrosshair, newProfile.layers[0]), ...newProfile.layers.slice(1)];
    } else {
      // 没有 layers 时创建一个单图层。
      newProfile.layers = [syncCrosshairToLayer(newCrosshair, createDefaultLayer(newCrosshair))];
    }
    const newConfig = {
      ...config,
      profiles: { ...config.profiles, [config.active_profile]: newProfile },
    };
    setConfig(newConfig);
    debouncedSave(newConfig);
  }, [config, profile, crosshair, debouncedSave, syncCrosshairToLayer]);

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

  /** 销毁配置窗口并切换焦点到目标游戏窗口。
   *  使用 destroy 而非 hide，让 WebView2 渲染进程被回收（~30-50MB），
   *  下次从托盘打开时由 show_or_recreate_window 重建。 */
  const hideAndSwitch = useCallback(async (targetWindow: string) => {
    focusTargetWindow(targetWindow).catch(console.error);
    await getCurrentWebviewWindow().destroy();
  }, []);

  const handleStartOverlay = useCallback(async () => {
    // 全屏模式不需要目标窗口；窗口模式需要。
    const isFullscreen = config?.settings.fullscreen_overlay ?? true;
    if (!isFullscreen && !profile?.target_window) return;
    try {
      await startOverlay(profile?.target_window ?? "");
      setOverlayActive(true);

      const pref = config?.settings.auto_switch_on_overlay ?? "ask";
      if (pref === "yes") {
        if (profile?.target_window) {
          await hideAndSwitch(profile.target_window);
        }
      } else if (pref === "no") {
        // 不隐藏，不做操作。
      } else {
        // 未设置偏好（ask），弹出对话框。
        setShowAutoSwitchDialog(true);
      }
    } catch (e) {
      console.error(e);
    }
  }, [profile?.target_window, config?.settings.fullscreen_overlay, config?.settings.auto_switch_on_overlay, hideAndSwitch]);

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

  /** 对话框取消（保持配置窗口）：不停止覆盖，保持配置窗口显示，按勾选状态保存偏好。 */
  const handleDialogCancel = useCallback(async () => {
    if (rememberChoice) {
      updateSettings({ auto_switch_on_overlay: "no" });
    }
    setShowAutoSwitchDialog(false);
  }, [rememberChoice, updateSettings]);

  /** ESC 键关闭对话框：停止覆盖（等同点击停止覆盖按钮）。 */
  const handleDialogEsc = useCallback(async () => {
    setShowAutoSwitchDialog(false);
    try {
      await stopOverlay();
      setOverlayActive(false);
    } catch (e) {
      console.error(e);
    }
  }, []);

  /** ESC 键监听：仅在对话框显示时生效。 */
  useEffect(() => {
    if (!showAutoSwitchDialog) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        handleDialogEsc();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showAutoSwitchDialog, handleDialogEsc]);

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

  // 加载状态：必须有 config。
  if (loading || !config) {
    return (
      <div className="h-screen flex items-center justify-center text-muted-foreground">
        {t("config.loading")}
      </div>
    );
  }

  // 异常情况（既没有 crosshair 又没有 layers）：显示错误并提供切换到图层编辑器。
  if (!crosshair && !hasLayers) {
    return (
      <div className="h-screen flex flex-col items-center justify-center text-muted-foreground gap-4">
        <span className="text-lg">{t("config.invalidFormat")}</span>
        <button
          className="text-xs px-3 py-1 border rounded hover:bg-accent"
          onClick={() => setLayersMode(true)}
        >
          {t("config.switchToLayersFallback")}
        </button>
      </div>
    );
  }

  // 到这里 crosshair 必为非 null（derivedCrosshair 已从 layers[0] 反向生成）。
  const ch = crosshair!;

  // 图层编辑器模式：显示全新 UI。
  if (layersMode) {
    return (
      <div className="h-screen flex flex-col bg-background text-foreground">
        <LayersEditor
          config={config}
          overlayActive={overlayActive}
          windows={windows}
          onStartOverlay={handleStartOverlay}
          onStopOverlay={handleStopOverlay}
          onRefreshWindows={refreshWindows}
          onUpdateSettings={updateSettings}
          onConfigChange={setConfig}
          onSwitchSingleLayer={() => setLayersMode(false)}
        />
      </div>
    );
  }

  return (
    <div className="h-screen flex bg-background text-foreground overflow-hidden">
      {/* 左侧预览 */}
      <div className="flex-1 p-4 min-w-0 min-h-0 relative">
        <Preview previewKey={profile?.layers} />
        {/* 切换到多图层按钮 */}
        <button
          onClick={() => setLayersMode(true)}
          className="absolute top-6 right-6 text-xs px-3 py-1.5 bg-primary text-primary-foreground rounded shadow hover:bg-primary/90 z-10"
          title={t("layers.switchToLayers")}
        >
          {t("layers.switchToLayers")} →
        </button>
      </div>

      {/* 右侧设置面板：顶部固定、中间滚动、底部固定 */}
      <div className="w-80 border-l bg-card p-4 flex flex-col gap-4 overflow-hidden h-screen">
        {/* 顶部固定区：样式 + 公共配置 */}
        <div className="space-y-3 shrink-0">
          {/* 样式选择 */}
          <div className="space-y-2">
            <Label className="text-sm">{t("config.style")}</Label>
            <Select
              value={ch.style}
              onValueChange={(v) => updateCrosshair({ style: v as CrosshairStyle }, { resetDefaults: true })}
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
                <span className="text-sm text-muted-foreground">{ch.opacity.toFixed(2)}</span>
              </div>
              <Slider
                value={[ch.opacity]}
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

            {/* 快捷颜色：点击色块直接设置准心颜色 */}
            <div className="space-y-2">
              <Label className="text-sm">{t("quickColors.title")}</Label>
              <div className="flex gap-1 flex-wrap">
                {(config.settings.quick_colors ?? []).map((qc, i) => {
                  const css = `rgb(${Math.round(qc[0] * 255)}, ${Math.round(qc[1] * 255)}, ${Math.round(qc[2] * 255)})`;
                  const isActive = ch.color[0] === qc[0] && ch.color[1] === qc[1] && ch.color[2] === qc[2];
                  return (
                    <button
                      key={i}
                      type="button"
                      title={css}
                      onClick={() => updateCrosshair({ color: [...qc] })}
                      className="w-5 h-5 rounded-full border-2 transition-colors"
                      style={{
                        backgroundColor: css,
                        borderColor: isActive ? "hsl(var(--primary))" : "hsl(var(--border))",
                      }}
                    />
                  );
                })}
              </div>
            </div>
          </div>
        </div>

        <Separator className="shrink-0" />

        {/* 中间样式配置：默认随窗口高度自适应，内容过多时才滚动 */}
        <div className="flex-1 min-h-0 overflow-y-auto pr-1">
          <StyleFields crosshair={ch} onChange={updateCrosshair} />
        </div>

        <Separator className="shrink-0" />

        {/* 底部固定区：覆盖模式 + 目标窗口 + 开始/停止覆盖 */}
        <div className="space-y-3 shrink-0">
          {/* 窗口模式勾选（默认全屏，勾选切换为窗口模式）。覆盖层激活时禁止切换，避免状态不一致。 */}
          <div className="flex items-center gap-2">
            <Checkbox
              id="window-mode"
              checked={!config.settings.fullscreen_overlay}
              disabled={overlayActive}
              title={overlayActive ? t("overlay.windowModeBlockedHint") : undefined}
              onCheckedChange={(v) => updateSettings({ fullscreen_overlay: !v })}
            />
            <Label
              htmlFor="window-mode"
              className={`text-sm ${overlayActive ? "text-muted-foreground cursor-not-allowed" : "cursor-pointer"}`}
            >
              {t("overlaySettings.windowMode")}
            </Label>
          </div>

          {/* 目标窗口（仅窗口模式时显示） */}
          {!config.settings.fullscreen_overlay && (
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <Label className="text-sm">{t("config.targetWindow")}</Label>
                <Button variant="ghost" size="sm" onClick={refreshWindows} className="h-8 text-sm px-2">
                  {t("config.refresh")}
                </Button>
              </div>
              <Select
                value={profile?.target_window || "__none__"}
                onValueChange={(v) => {
                  if (!profile) return;
                  const newConfig: AppConfig = {
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
                  debouncedSave(newConfig);
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
          )}

          {/* 开始/停止覆盖 */}
          <div>
            {overlayActive ? (
              <Button variant="destructive" className="w-full h-8 text-sm" onClick={handleStopOverlay}>
                ■ {t("config.stopOverlay")}
              </Button>
            ) : (
              <Button className="w-full h-8 text-sm" onClick={handleStartOverlay} disabled={!config.settings.fullscreen_overlay && !profile?.target_window}>
                ▶ {t("config.startOverlay")}
              </Button>
            )}
          </div>
        </div>

        {/* 开发者面板（仅 devTabUnlocked=true 时显示） */}
        {devTabUnlocked && (
          <div className="shrink-0 border-t pt-2 mt-2 max-h-48 overflow-y-auto">
            <DeveloperPanel
              config={config}
              version={version}
              onClose={() => {
                setDevTabUnlocked(false);
                localStorage.removeItem("peregrine:dev-tab");
              }}
            />
          </div>
        )}

        {/* 底部信息 */}
        <div
          className="text-xs text-muted-foreground text-right cursor-pointer select-none shrink-0"
          onClick={() => {
            setVersionClickCount((n) => n + 1);
          }}
          title={devTabUnlocked ? t("developer.toggle") : t("developer.unlockHint")}
        >
          Peregrine v{version || "..."}
          {versionClickCount > 0 && versionClickCount < 3 && (
            <span className="ml-1 text-[10px] opacity-60">
              ({3 - versionClickCount} {t("developer.remaining")})
            </span>
          )}
          {devTabUnlocked && <span className="ml-1 text-[10px] text-yellow-500">{t("developer.tag")}</span>}
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

      {/* 发现新版本对话框 */}
      {updateAvailable && !updating && (
        <div className="fixed top-4 right-4 z-50 bg-card border rounded-lg shadow-lg p-4 max-w-xs space-y-2">
          <p className="text-sm font-medium">
            {t("settings.updateAvailable")}：v{updateAvailable.version}
          </p>
          {updateAvailable.body && (
            <p className="text-xs text-muted-foreground whitespace-pre-wrap line-clamp-4">
              {updateAvailable.body}
            </p>
          )}
          <div className="flex gap-2">
            <Button
              size="sm"
              onClick={async () => {
                setUpdateAvailable(null);
                setUpdating(true);
                setUpdateProgress(0);
                try {
                  const channel = config?.settings?.update_channel ?? "stable";
                  const cnMirror = config?.settings?.cn_mirror ?? false;
                  const mirrorUrl = config?.settings?.mirror_url ?? "https://v4.gh-proxy.org";
                  await downloadAndInstallUpdate(channel, cnMirror, mirrorUrl, (downloaded, total) => {
                    if (total > 0) {
                      setUpdateProgress(Math.min(100, Math.round((downloaded / total) * 100)));
                    }
                  });
                } catch (e) {
                  console.error("[Update] download failed:", e);
                  setUpdating(false);
                }
              }}
            >
              {t("settings.updateNow")}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setUpdateAvailable(null)}
            >
              {t("settings.updateLater")}
            </Button>
          </div>
        </div>
      )}

      {/* 更新下载进度 */}
      {updating && (
        <div className="fixed top-4 right-4 z-50 bg-card border rounded-lg shadow-lg p-4 max-w-xs space-y-2">
          <p className="text-xs text-blue-500">{t("settings.updating")}</p>
          <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full bg-blue-500 rounded-full transition-all"
              style={{ width: `${updateProgress || 30}%` }}
            />
          </div>
          {updateProgress > 0 && (
            <p className="text-xs text-muted-foreground text-right">{updateProgress}%</p>
          )}
        </div>
      )}
    </div>
  );
}

/** 把 Crosshair 样式映射到内置物料引用。 */
function crosshairToMaterial(style: CrosshairStyle): MaterialRef {
  switch (style) {
    case "edge_rect":
      return { kind: "builtin", id: "builtin.edge_rect" };
    case "custom_image":
      return { kind: "builtin", id: "builtin.image" };
    case "cross":
      return { kind: "builtin", id: "builtin.cross" };
    case "large_cross":
      return { kind: "builtin", id: "builtin.large_cross" };
    case "corner_dots4":
      return { kind: "builtin", id: "builtin.corner_dots" };
    case "corner_dots6":
      return { kind: "builtin", id: "builtin.corner_dots" };
    case "corner_dots8":
      return { kind: "builtin", id: "builtin.corner_dots" };
    case "ring":
      return { kind: "builtin", id: "builtin.ring" };
    case "custom_orb":
      return { kind: "builtin", id: "builtin.custom_orb" };
    case "random_orb":
      return { kind: "builtin", id: "builtin.random_orb" };
    case "border_frame":
      return { kind: "builtin", id: "builtin.border_frame" };
    case "edge_arrows":
      return { kind: "builtin", id: "builtin.edge_arrows" };
    case "grid":
      return { kind: "builtin", id: "builtin.grid" };
    default:
      return { kind: "builtin", id: "builtin.cross" };
  }
}

/** 把 Crosshair 字段转换为对应物料的 params。 */
function crosshairToParams(crosshair: Crosshair): Record<string, unknown> {
  switch (crosshair.style) {
    case "edge_rect":
      return {
        size: crosshair.size,
        secondary_size: crosshair.secondary_size,
        anchor: crosshair.anchor,
        margin: crosshair.margin,
        corner_radius: crosshair.corner_radius,
      };
    case "custom_image":
      return {
        path: crosshair.image_path,
        scale: crosshair.image_scale,
        offset_x: crosshair.image_offset_x,
        offset_y: crosshair.image_offset_y,
        width: crosshair.size,
        height: crosshair.size,
      };
    case "cross":
      return {
        size: crosshair.size,
        thickness: crosshair.thickness,
        gap: crosshair.gap,
      };
    case "large_cross":
      return {
        thickness: crosshair.thickness,
      };
    case "corner_dots4":
    case "corner_dots6":
    case "corner_dots8":
      return {
        count: styleToCornerCount(crosshair.style),
        offset: crosshair.offset,
        thickness: crosshair.thickness,
        radius: crosshair.radius,
      };
    case "ring":
      return {
        radius_pct: crosshair.ring_radius_pct,
        thickness: crosshair.thickness,
        style: crosshair.ring_style,
      };
    case "custom_orb":
      return {
        radius: crosshair.radius,
        offset: crosshair.offset,
        top_count: crosshair.custom_orb_top_count,
        bottom_count: crosshair.custom_orb_bottom_count,
        left_count: crosshair.custom_orb_left_count,
        right_count: crosshair.custom_orb_right_count,
        orb_positions: crosshair.orb_positions,
      };
    case "random_orb":
      return {
        count: crosshair.random_orb_count,
        offset: crosshair.random_orb_offset,
        jitter: crosshair.random_orb_jitter,
        radius_min: crosshair.random_radius_min,
        radius_max: crosshair.random_radius_max,
      };
    case "border_frame":
      return {
        thickness: crosshair.thickness,
        offset: crosshair.offset,
        style: crosshair.border_frame_style,
        inset: crosshair.border_inset,
      };
    case "edge_arrows":
      return {
        size: crosshair.size,
        arrow_width: crosshair.arrow_width,
        distance: crosshair.arrow_distance,
        tail_per_edge: crosshair.arrow_tail_per_edge,
        tail_top: crosshair.arrow_tail_top,
        tail_bottom: crosshair.arrow_tail_bottom,
        tail_left: crosshair.arrow_tail_left,
        tail_right: crosshair.arrow_tail_right,
      };
    case "grid":
      return {
        size: crosshair.grid_size,
        thickness: crosshair.thickness,
        alignment: crosshair.grid_alignment,
      };
    default:
      return {};
  }
}

function styleToCornerCount(style: CrosshairStyle): number {
  switch (style) {
    case "corner_dots4":
      return 4;
    case "corner_dots6":
      return 6;
    case "corner_dots8":
      return 8;
    default:
      return 4;
  }
}

function createDefaultLayer(crosshair: Crosshair): Layer {
  return {
    id: crypto.randomUUID(),
    name: "crosshair",
    material: crosshairToMaterial(crosshair.style),
    params: crosshairToParams(crosshair),
    style: {
      color: crosshair.color,
      opacity: crosshair.opacity,
      blend_mode: "normal",
    },
    transform: {
      offset_x: 0,
      offset_y: 0,
      scale: 1,
      rotation_deg: 0,
    },
    visible: true,
    locked: false,
  };
}

/** 从内置物料 id 推断 CrosshairStyle。 */
function materialIdToStyle(materialId: string): CrosshairStyle {
  switch (materialId) {
    case "builtin.edge_rect":
      return "edge_rect";
    case "builtin.image":
      return "custom_image";
    case "builtin.large_cross":
      return "large_cross";
    case "builtin.corner_dots":
      return "corner_dots4";
    case "builtin.ring":
      return "ring";
    case "builtin.custom_orb":
      return "custom_orb";
    case "builtin.random_orb":
      return "random_orb";
    case "builtin.border_frame":
      return "border_frame";
    case "builtin.edge_arrows":
      return "edge_arrows";
    case "builtin.grid":
      return "grid";
    case "builtin.cross":
    default:
      return "cross";
  }
}

/**
 * 从 layers[0] 反向生成 Crosshair，用于旧版单图层 UI。
 * 无法精确还原的字段使用默认值。
 */
function layerToCrosshair(layer: Layer): Crosshair | null {
  const style = materialIdToStyle(
    layer.material.kind === "builtin" ? layer.material.id : "builtin.cross",
  );
  const params = layer.params;
  const base = getDefaultCrosshairForStyle(style);
  const color = layer.style?.color ?? base.color;
  const opacity = layer.style?.opacity ?? base.opacity;
  const newBase: Crosshair = { ...base, color, opacity, style };

  switch (style) {
    case "edge_rect":
      return {
        ...newBase,
        size: (params.size as number) ?? newBase.size,
        secondary_size: (params.secondary_size as number) ?? newBase.secondary_size,
        anchor: (params.anchor as Anchor) ?? newBase.anchor,
        margin: (params.margin as number) ?? newBase.margin,
        corner_radius: (params.corner_radius as number) ?? newBase.corner_radius,
      };
    case "custom_image":
      return {
        ...newBase,
        image_path: (params.path as string) ?? newBase.image_path,
        image_scale: (params.scale as number) ?? newBase.image_scale,
        image_offset_x: (params.offset_x as number) ?? newBase.image_offset_x,
        image_offset_y: (params.offset_y as number) ?? newBase.image_offset_y,
        size: (params.width as number) ?? newBase.size,
      };
    case "cross":
      return {
        ...newBase,
        size: (params.size as number) ?? newBase.size,
        thickness: (params.thickness as number) ?? newBase.thickness,
        gap: (params.gap as number) ?? newBase.gap,
      };
    case "large_cross":
      return {
        ...newBase,
        thickness: (params.thickness as number) ?? newBase.thickness,
      };
    case "corner_dots4":
    case "corner_dots6":
    case "corner_dots8": {
      const count = (params.count as number) ?? 4;
      const styleName: CrosshairStyle = count === 6 ? "corner_dots6" : count === 8 ? "corner_dots8" : "corner_dots4";
      return {
        ...newBase,
        style: styleName,
        offset: (params.offset as number) ?? newBase.offset,
        thickness: (params.thickness as number) ?? newBase.thickness,
        radius: (params.radius as number) ?? newBase.radius,
      };
    }
    case "ring":
      return {
        ...newBase,
        ring_radius_pct: (params.radius_pct as number) ?? newBase.ring_radius_pct,
        thickness: (params.thickness as number) ?? newBase.thickness,
        ring_style: (params.style as RingStyle) ?? newBase.ring_style,
      };
    case "custom_orb":
      return {
        ...newBase,
        radius: (params.radius as number) ?? newBase.radius,
        offset: (params.offset as number) ?? newBase.offset,
        orb_positions: (params.orb_positions as number) ?? newBase.orb_positions,
        custom_orb_top_count: (params.top_count as number) ?? newBase.custom_orb_top_count,
        custom_orb_bottom_count: (params.bottom_count as number) ?? newBase.custom_orb_bottom_count,
        custom_orb_left_count: (params.left_count as number) ?? newBase.custom_orb_left_count,
        custom_orb_right_count: (params.right_count as number) ?? newBase.custom_orb_right_count,
      };
    case "random_orb":
      return {
        ...newBase,
        random_orb_count: (params.count as number) ?? newBase.random_orb_count,
        random_orb_offset: (params.offset as number) ?? newBase.random_orb_offset,
        random_orb_jitter: (params.jitter as number) ?? newBase.random_orb_jitter,
        random_radius_min: (params.radius_min as number) ?? newBase.random_radius_min,
        random_radius_max: (params.radius_max as number) ?? newBase.random_radius_max,
      };
    case "border_frame":
      return {
        ...newBase,
        thickness: (params.thickness as number) ?? newBase.thickness,
        offset: (params.offset as number) ?? newBase.offset,
        border_frame_style: (params.style as BorderFrameStyle) ?? newBase.border_frame_style,
        border_inset: (params.inset as boolean) ?? newBase.border_inset,
      };
    case "edge_arrows":
      return {
        ...newBase,
        size: (params.size as number) ?? newBase.size,
        arrow_width: (params.arrow_width as number) ?? newBase.arrow_width,
        arrow_distance: (params.distance as number) ?? newBase.arrow_distance,
        arrow_tail_per_edge: (params.tail_per_edge as boolean) ?? newBase.arrow_tail_per_edge,
        arrow_tail_top: (params.tail_top as number) ?? newBase.arrow_tail_top,
        arrow_tail_bottom: (params.tail_bottom as number) ?? newBase.arrow_tail_bottom,
        arrow_tail_left: (params.tail_left as number) ?? newBase.arrow_tail_left,
        arrow_tail_right: (params.tail_right as number) ?? newBase.arrow_tail_right,
      };
    case "grid":
      return {
        ...newBase,
        grid_size: (params.size as number) ?? newBase.grid_size,
        thickness: (params.thickness as number) ?? newBase.thickness,
        grid_alignment: (params.alignment as GridAlignment) ?? newBase.grid_alignment,
      };
    default:
      return newBase;
  }
}
