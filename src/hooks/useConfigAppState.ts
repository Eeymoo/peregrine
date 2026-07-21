import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useI18n } from "@/lib/i18n";
import {
  getConfig,
  getAppVersion,
  getOverlayActive,
  listWindowTitles,
  listProfiles,
  setActiveProfile,
} from "@/lib/api";
import { useSettingsSync } from "@/hooks/useSettingsSync";
import { useInitMirror } from "@/hooks/useSettingsSync";
import { isLayerLegacyCompatible } from "@/lib/layers";
import type { AppConfig } from "@/types/config";

export function useConfigAppState() {
  const { t } = useI18n();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [windows, setWindows] = useState<string[]>([]);
  const [profiles, setProfiles] = useState<string[]>([]);
  const [overlayActive, setOverlayActive] = useState(false);
  const [loading, setLoading] = useState(true);
  const [version, setVersion] = useState("");
  // 默认根据当前 active profile 的兼容性选择模式：
  // 多图层配置 → 打开多图层模式；单图层兼容配置 → 打开单图层（旧版）模式。
  const [layersMode, setLayersMode] = useState(false);

  useEffect(() => {
    getCurrentWebviewWindow().setTitle(`${t("app.title")} ${t("config.title")}`).catch(() => {});
  }, [t]);

  useEffect(() => {
    getConfig()
      .then((cfg) => {
        setConfig(cfg);
        const profile = cfg.profiles[cfg.active_profile];
        const compatible =
          profile?.layers?.length === 1 && isLayerLegacyCompatible(profile.layers[0]);
        setLayersMode(!compatible);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
    refreshWindows();
    refreshProfiles();
    getAppVersion().then(setVersion).catch(() => {});
    getOverlayActive().then(setOverlayActive).catch(() => {});
  }, []);

  useSettingsSync(setConfig);
  useInitMirror();

  const refreshWindows = () => {
    listWindowTitles().then(setWindows).catch(console.error);
  };

  const refreshProfiles = () => {
    listProfiles().then(setProfiles).catch(console.error);
  };

  /** 切换 active profile：调后端后重新拉取完整配置与 profile 列表。 */
  const changeActiveProfile = async (name: string) => {
    await setActiveProfile(name);
    const fresh = await getConfig();
    setConfig(fresh);
    setProfiles(await listProfiles());
  };

  return {
    config,
    setConfig,
    windows,
    profiles,
    setProfiles,
    overlayActive,
    setOverlayActive,
    loading,
    version,
    layersMode,
    setLayersMode,
    refreshWindows,
    refreshProfiles,
    changeActiveProfile,
  };
}
