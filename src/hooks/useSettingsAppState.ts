import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useI18n } from "@/lib/i18n";
import { getConfig, getAppVersion, updatePreferences } from "@/lib/api";
import type { AppConfig } from "@/types/config";

export function useSettingsAppState() {
  const { t, locale, setLocale, resolvedLocale } = useI18n();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [autoSwitch, setAutoSwitchState] = useState<string>("ask");
  const [version, setVersion] = useState("");

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
          const isZh =
            localeVal === "zh-CN" ||
            (localeVal === "auto" && navigator.language.toLowerCase().startsWith("zh"));
          if (isZh) {
            await updatePreferences({ cn_mirror: true });
          }
          localStorage.setItem("cn_mirror_initialized", "1");
        }
      } catch {
        /* 静默失败 */
      }
    };
    initMirror();
  }, []);

  /** 监听后端 settings 变更（来自托盘或配置窗口），同步本窗口状态。 */
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
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
        antialiasing?: boolean;
        renderer_backend?: "cpu" | "svg";
        quick_colors?: [number, number, number, number][];
        hotkey_bindings?: [string, string][];
      }>("peregrine:settings-changed", (event) => {
        const {
          auto_switch_on_overlay,
          fullscreen_overlay,
          live_drag_preview,
          gpu_acceleration,
          update_channel,
          cn_mirror,
          mirror_url,
          antialiasing,
          renderer_backend,
          quick_colors,
          hotkey_bindings,
        } = event.payload;
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
    })();
    return () => unlisten?.();
  }, []);

  return {
    t,
    locale,
    setLocale,
    resolvedLocale,
    config,
    setConfig,
    autoSwitch,
    setAutoSwitchState,
    version,
  };
}
