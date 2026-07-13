import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useI18n } from "@/lib/i18n";
import { getConfig, getAppVersion, getOverlayActive } from "@/lib/api";
import { useSettingsSync } from "@/hooks/useSettingsSync";
import { useInitMirror } from "@/hooks/useSettingsSync";
import type { AppConfig } from "@/types/config";

export function useConfigAppState() {
  const { t } = useI18n();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [overlayActive, setOverlayActive] = useState(false);
  const [loading, setLoading] = useState(true);
  const [version, setVersion] = useState("");

  useEffect(() => {
    getCurrentWebviewWindow().setTitle(`${t("app.title")} ${t("config.title")}`).catch(() => {});
  }, [t]);

  useEffect(() => {
    getConfig()
      .then(setConfig)
      .catch(console.error)
      .finally(() => setLoading(false));
    getAppVersion().then(setVersion).catch(() => {});
    getOverlayActive().then(setOverlayActive).catch(() => {});
  }, []);

  useSettingsSync(setConfig);
  useInitMirror();

  return {
    config,
    setConfig,
    overlayActive,
    setOverlayActive,
    loading,
    version,
  };
}
