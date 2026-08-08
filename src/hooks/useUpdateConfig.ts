import { useCallback } from "react";
import { updatePreferences } from "@/lib/api";
import type { AppConfig } from "@/types/config";

export function useUpdateConfig(config: AppConfig | null, setConfig: (cfg: AppConfig) => void) {
  const updateConfigSettings = useCallback(
    <K extends keyof AppConfig["settings"]>(key: K, value: AppConfig["settings"][K]) => {
      if (!config) return;
      const newConfig: AppConfig = {
        ...config,
        settings: { ...config.settings, [key]: value },
      };
      setConfig(newConfig);
      updatePreferences({ [key]: value } as Partial<AppConfig["settings"]>).catch(console.error);
    },
    [config, setConfig]
  );

  return { updateConfigSettings };
}
