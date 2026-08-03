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
import { MATERIAL_RUNTIME_ENABLED } from "@/lib/feature";
import type { AppConfig } from "@/types/config";

export function useConfigAppState() {
  const { t } = useI18n();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [windows, setWindows] = useState<string[]>([]);
  const [profiles, setProfiles] = useState<string[]>([]);
  const [overlayActive, setOverlayActive] = useState(false);
  const [loading, setLoading] = useState(true);
  const [version, setVersion] = useState("");
  // 模式持久化（2026-08-03 修订）：layersMode 持久化到 localStorage，
  // 启动时无差别恢复关闭前的模式（单图层关→单图层开，多图层关→多图层开），
  // 无持久化值时默认单图层。恢复后仍套用兼容性规则（见下方加载逻辑）。
  const LAYERS_MODE_KEY = "peregrine:layers-mode";
  const [layersMode, setLayersModeState] = useState(
    () => localStorage.getItem(LAYERS_MODE_KEY) === "1",
  );
  /** 切换编辑器模式并写回 localStorage 持久化值。 */
  const setLayersMode = (v: boolean) => {
    setLayersModeState(v);
    localStorage.setItem(LAYERS_MODE_KEY, v ? "1" : "0");
  };

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
        // MATERIAL_RUNTIME_ENABLED 门控：启用时使用真实兼容性判定——
        // 恢复为单图层且 active profile 不兼容时强制切多图层（写回持久化值）；
        // 软关闭期间恒为兼容，不做强制切换。
        const effectiveCompatible = MATERIAL_RUNTIME_ENABLED ? compatible : true;
        // 模式恢复优先：仅当恢复为单图层且不兼容时才强制切到多图层（写回持久化值）。
        if (!layersMode && !effectiveCompatible) {
          setLayersMode(true);
        }
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
