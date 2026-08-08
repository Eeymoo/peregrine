import { useEffect, useState } from "react";
import { getConfig, updatePreferences, getAppVersion } from "@/lib/api";
import type { AppConfig } from "@/types/config";

export function useConfig() {
  const [config, setConfig] = useState<AppConfig | null>(null);

  useEffect(() => {
    getConfig()
      .then(setConfig)
      .catch(console.error);
  }, []);

  return { config, setConfig };
}

export function useAppVersion() {
  const [version, setVersion] = useState("");

  useEffect(() => {
    getAppVersion().then(setVersion).catch(() => {});
  }, []);

  return version;
}

export function useInitMirror() {
  useEffect(() => {
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
}
