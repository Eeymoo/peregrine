import { useEffect, useState, useCallback } from "react";
import { useI18n, LANGUAGE_OPTIONS, detectLocale, type Locale } from "@/lib/i18n";
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Switch } from "@/components/ui/switch";
import {
  Card,
  CardContent,
} from "@/components/ui/card";
import appIcon from "../assets/icon.png";
import { getConfig, updatePreferences, getAppVersion, relaunchApp, checkForUpdate, downloadAndInstallUpdate } from "@/lib/api";
import type { AppConfig } from "@/types/config";

export default function SettingsApp() {
  const { t, locale, setLocale, resolvedLocale } = useI18n();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [autoSwitch, setAutoSwitchState] = useState<string>("ask");
  const [version, setVersion] = useState("");
  const [updateState, setUpdateState] = useState<{
    status: "idle" | "checking" | "available" | "upToDate" | "updating" | "failed";
    version?: string;
    body?: string;
    progress?: number;
  }>({ status: "idle" });

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

    // 简体中文用户首次启动自动启用中国大陆加速镜像（仅初始化一次，不覆盖用户选择）。
    const initMirror = async () => {
      try {
        const initialized = localStorage.getItem("cn_mirror_initialized");
        if (!initialized) {
          const cfg = await getConfig();
          const localeVal = cfg.settings?.locale ?? "auto";
          const isZh = localeVal === "zh-CN" ||
            (localeVal === "auto" && navigator.language.toLowerCase().startsWith("zh"));
          if (isZh) {
            await updatePreferences({ cn_mirror: true });
          }
          localStorage.setItem("cn_mirror_initialized", "1");
        }
      } catch { /* 静默失败 */ }
    };
    initMirror();

    // 启动时自动检测更新（静默，发现新版本才提示）。延迟 3 秒避免抢焦点。
    const autoCheck = async () => {
      try {
        await new Promise((r) => setTimeout(r, 3000));
        const cfg = await getConfig();
        const channel = cfg.settings?.update_channel ?? "stable";
        const cnMirror = cfg.settings?.cn_mirror ?? false;
        const mirrorUrl = cfg.settings?.mirror_url ?? "https://v4.gh-proxy.org";
        const result = await checkForUpdate(channel, cnMirror, mirrorUrl);
        if (result.available) {
          setUpdateState({ status: "available", version: result.version, body: result.body });
        }
      } catch { /* 静默失败 */ }
    };
    autoCheck();
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
          cn_mirror?: boolean;
          mirror_url?: string;
        }>("peregrine:settings-changed", (event) => {
          const { auto_switch_on_overlay, fullscreen_overlay, live_drag_preview, gpu_acceleration, update_channel, cn_mirror, mirror_url } = event.payload;
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
            if (cn_mirror !== undefined) {
              settings.cn_mirror = cn_mirror;
            }
            if (mirror_url !== undefined) {
              settings.mirror_url = mirror_url;
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
    <div className="h-screen flex flex-col bg-background text-foreground">
      <Tabs defaultValue="general" className="flex flex-col h-full">
        <div className="px-6 pt-5">
          <TabsList className="grid grid-cols-4 w-full">
            <TabsTrigger value="general">{t("settings.sectionGeneral")}</TabsTrigger>
            <TabsTrigger value="overlay">{t("settings.sectionOverlay")}</TabsTrigger>
            <TabsTrigger value="update">{t("settings.sectionUpdate")}</TabsTrigger>
            <TabsTrigger value="about">{t("settings.sectionAbout")}</TabsTrigger>
          </TabsList>
        </div>

        {/* ===== 通用 ===== */}
        <TabsContent value="general" className="flex-1 overflow-y-auto m-0 p-6">
          <Card>
            <CardContent className="space-y-6 pt-6">
              {/* 语言 */}
              <div className="flex items-center justify-between gap-4">
                <Label className="text-sm font-medium">{t("settings.language")}</Label>
                <Select value={locale} onValueChange={(v) => {
                  const next = v as Locale;
                  setLocale(next);
                  // 非中文语言强制关闭中国大陆加速镜像。
                  const resolved = next === "auto" ? detectLocale() : next;
                  if (resolved !== "zh-CN" && config) {
                    const newConfig: AppConfig = {
                      ...config,
                      settings: { ...config.settings, cn_mirror: false },
                    };
                    setConfig(newConfig);
                    updatePreferences({ cn_mirror: false }).catch(console.error);
                  }
                }}>
                  <SelectTrigger className="w-40 h-8 text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {LANGUAGE_OPTIONS.map((opt) => (
                      <SelectItem key={opt.value} value={opt.value}>
                        {opt.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* GPU 加速 */}
              <div className="flex items-center justify-between gap-4">
                <div className="space-y-0.5">
                  <Label className="text-sm font-medium">{t("settings.gpuAcceleration")}</Label>
                  <p className="text-xs text-muted-foreground">
                    {t("settings.gpuAccelerationHint")}
                  </p>
                </div>
                <Switch
                  checked={config?.settings?.gpu_acceleration ?? false}
                  onCheckedChange={async (v) => {
                    if (!config) return;
                    const newConfig: AppConfig = {
                      ...config,
                      settings: { ...config.settings, gpu_acceleration: v },
                    };
                    setConfig(newConfig);
                    await updatePreferences({ gpu_acceleration: v });
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
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* ===== 覆盖层 ===== */}
        <TabsContent value="overlay" className="flex-1 overflow-y-auto m-0 p-6">
          <Card>
            <CardContent className="space-y-6 pt-6">
              {/* 启动覆盖时的行为 */}
              <div className="flex items-start justify-between gap-4">
                <div className="space-y-0.5">
                  <Label className="text-sm font-medium">{t("settings.autoSwitch")}</Label>
                  <p className="text-xs text-muted-foreground">
                    {t("settings.autoSwitchHint")}
                  </p>
                </div>
                <Select value={autoSwitch} onValueChange={handleAutoSwitchChange}>
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
                  disabled={config?.settings?.fullscreen_overlay ?? true}
                  onCheckedChange={(v) => {
                    if (!config) return;
                    const newConfig: AppConfig = {
                      ...config,
                      settings: { ...config.settings, live_drag_preview: v },
                    };
                    setConfig(newConfig);
                    updatePreferences({ live_drag_preview: v }).catch(console.error);
                  }}
                />
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* ===== 更新 ===== */}
        <TabsContent value="update" className="flex-1 overflow-y-auto m-0 p-6">
          <Card>
            <CardContent className="space-y-6 pt-6">
              {/* 中国大陆加速（仅中文显示） */}
              {resolvedLocale === "zh-CN" && (
                <>
                  <div className="flex items-center justify-between gap-4">
                    <div className="space-y-0.5">
                      <Label className="text-sm font-medium">{t("settings.cnMirror")}</Label>
                      <p className="text-xs text-muted-foreground">
                        {t("settings.cnMirrorHint")}
                      </p>
                    </div>
                    <Switch
                      checked={config?.settings?.cn_mirror ?? false}
                      onCheckedChange={(v) => {
                        if (!config) return;
                        const newConfig: AppConfig = {
                          ...config,
                          settings: { ...config.settings, cn_mirror: v },
                        };
                        setConfig(newConfig);
                        updatePreferences({ cn_mirror: v }).catch(console.error);
                      }}
                    />
                  </div>

                  {/* 加速站选择 */}
                  {config?.settings?.cn_mirror && (
                    <div className="flex items-center justify-between gap-4">
                      <Label className="text-sm font-medium">{t("settings.mirrorSite")}</Label>
                      <Select
                        value={(() => {
                          const url = config?.settings?.mirror_url ?? "https://v4.gh-proxy.org";
                          const presets = [
                            "https://gh-proxy.org",
                            "https://v4.gh-proxy.org",
                            "https://v6.gh-proxy.org",
                            "https://cdn.gh-proxy.org",
                          ];
                          return presets.includes(url) ? url : "__custom__";
                        })()}
                        onValueChange={(v) => {
                          if (!config) return;
                          if (v === "__custom__") {
                            // 选「自定义」时清空 mirror_url，触发输入框显示。
                            const newConfig: AppConfig = {
                              ...config,
                              settings: { ...config.settings, mirror_url: "" },
                            };
                            setConfig(newConfig);
                            return;
                          }
                          const newConfig: AppConfig = {
                            ...config,
                            settings: { ...config.settings, mirror_url: v },
                          };
                          setConfig(newConfig);
                          updatePreferences({ mirror_url: v }).catch(console.error);
                        }}
                      >
                        <SelectTrigger className="w-48 h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="https://gh-proxy.org">gh-proxy.org</SelectItem>
                          <SelectItem value="https://v4.gh-proxy.org">v4.gh-proxy.org（推荐）</SelectItem>
                          <SelectItem value="https://v6.gh-proxy.org">v6.gh-proxy.org</SelectItem>
                          <SelectItem value="https://cdn.gh-proxy.org">cdn.gh-proxy.org</SelectItem>
                          <SelectItem value="__custom__">{t("settings.mirrorCustom")}</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  )}

                  {/* 自定义镜像地址输入 */}
                  {config?.settings?.cn_mirror &&
                    ![
                      "https://gh-proxy.org",
                      "https://v4.gh-proxy.org",
                      "https://v6.gh-proxy.org",
                      "https://cdn.gh-proxy.org",
                    ].includes(config?.settings?.mirror_url ?? "") && (
                    <div className="flex items-center justify-between gap-4">
                      <Label className="text-sm font-medium">{t("settings.mirrorCustomUrl")}</Label>
                      <input
                        type="text"
                        className="flex h-8 w-48 rounded-md border border-input bg-background px-2 text-xs ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        value={config?.settings?.mirror_url ?? ""}
                        placeholder="https://your-mirror.example.com"
                        onChange={(e) => {
                          if (!config) return;
                          const newConfig: AppConfig = {
                            ...config,
                            settings: { ...config.settings, mirror_url: e.target.value },
                          };
                          setConfig(newConfig);
                        }}
                        onBlur={(e) => {
                          const val = e.target.value.trim();
                          if (val && config) {
                            updatePreferences({ mirror_url: val }).catch(console.error);
                          }
                        }}
                      />
                    </div>
                  )}
                </>
              )}

              {/* 更新通道 */}
              <div className="flex items-center justify-between gap-4">
                <Label className="text-sm font-medium">{t("settings.updateChannel")}</Label>
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
                  <SelectTrigger className="w-40 h-8 text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="stable">{t("settings.updateChannelStable")}</SelectItem>
                    <SelectItem value="prerelease">{t("settings.updateChannelPrerelease")}</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* 检查更新按钮 */}
              <Button
                className="w-full"
                size="xs"
                disabled={updateState.status === "checking" || updateState.status === "updating"}
                onClick={async () => {
                  setUpdateState({ status: "checking" });
                  try {
                    const channel = config?.settings?.update_channel ?? "stable";
                    const cnMirror = config?.settings?.cn_mirror ?? false;
                    const mirrorUrl = config?.settings?.mirror_url ?? "https://v4.gh-proxy.org";
                    const result = await checkForUpdate(channel, cnMirror, mirrorUrl);
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
                {updateState.status === "checking"
                  ? t("settings.checking") || "..."
                  : t("settings.checkUpdate")}
              </Button>

              {/* 结果区域 */}
              {updateState.status === "failed" && (
                <Card className="bg-muted/50 p-4">
                  <p className="text-sm text-red-500">{t("settings.updateFailed")}</p>
                </Card>
              )}

              {updateState.status === "upToDate" && (
                <Card className="bg-muted/50 p-4">
                  <p className="text-sm text-green-600">✓ {t("settings.updateUpToDate")}</p>
                </Card>
              )}

              {updateState.status === "updating" && (
                <Card className="bg-muted/50 p-4 space-y-2">
                  <p className="text-sm text-blue-500">{t("settings.updating")}</p>
                  <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
                    <div
                      className="h-full bg-blue-500 rounded-full transition-all"
                      style={{ width: updateState.progress ? `${updateState.progress}%` : "30%" }}
                    />
                  </div>
                  {updateState.progress !== undefined && (
                    <p className="text-xs text-muted-foreground text-right">
                      {Math.round(updateState.progress)}%
                    </p>
                  )}
                </Card>
              )}

              {updateState.status === "available" && (
                <Card className="bg-muted/50 p-4 space-y-3">
                  <p className="text-sm font-medium">
                    {t("settings.updateAvailable")}：v{updateState.version}
                  </p>
                  {updateState.body && (
                    <p className="text-sm text-muted-foreground whitespace-pre-wrap line-clamp-6">
                      {updateState.body}
                    </p>
                  )}
                  <div className="flex gap-2">
                    <Button
                      size="xs"
                      onClick={async () => {
                        setUpdateState({ status: "updating", progress: 0 });
                        try {
                          const channel = config?.settings?.update_channel ?? "stable";
                          const cnMirror = config?.settings?.cn_mirror ?? false;
                          const mirrorUrl = config?.settings?.mirror_url ?? "https://v4.gh-proxy.org";
                          await downloadAndInstallUpdate(channel, cnMirror, mirrorUrl, (downloaded, total) => {
                            if (total > 0) {
                              const pct = Math.min(100, Math.round((downloaded / total) * 100));
                              setUpdateState((s) => ({ ...s, progress: pct }));
                            }
                          });
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
                      size="xs"
                      onClick={() => setUpdateState({ status: "idle" })}
                    >
                      {t("settings.updateLater")}
                    </Button>
                  </div>
                </Card>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* ===== 关于 ===== */}
        <TabsContent value="about" className="flex-1 overflow-y-auto m-0 p-6">
          <Card>
            <CardContent className="space-y-6 pt-6">
              {/* 头部 */}
              <div className="text-center space-y-2">
                <img
                  src={appIcon}
                  alt="Peregrine"
                  className="w-16 h-16 mx-auto rounded-2xl"
                />
                <h2 className="text-xl font-bold">Peregrine</h2>
                <p className="text-sm text-muted-foreground leading-relaxed">
                  {t("settings.about.description")}
                </p>
              </div>

              <Separator />

              {/* 信息列表 */}
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">{t("settings.about.version")}</span>
                  <span>{version || "..."}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">{t("settings.about.publisher")}</span>
                  <span>Eeymoo</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">{t("settings.about.license")}</span>
                  <span>{t("license.mit")}</span>
                </div>
                <div className="flex justify-between items-center text-sm">
                  <span className="text-muted-foreground">{t("settings.about.repository")}</span>
                  <Button variant="link" size="xs" className="p-0 h-auto" onClick={() => {
                    if (typeof window !== "undefined" && window.open) {
                      window.open("https://github.com/Eeymoo/peregrine", "_blank");
                    }
                  }}>
                    GitHub ↗
                  </Button>
                </div>
              </div>

              <Button
                variant="outline"
                size="xs"
                className="w-full"
                onClick={() => {
                  const info = `Peregrine v${version}\n发行者: Eeymoo\n许可: MIT\n仓库: https://github.com/Eeymoo/peregrine`;
                  navigator.clipboard?.writeText(info).catch(() => {});
                }}
              >
                {t("settings.copyVersionInfo")}
              </Button>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
