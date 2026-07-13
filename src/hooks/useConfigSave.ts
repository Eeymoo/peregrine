import { useCallback, useMemo, useRef } from "react";
import { saveConfig } from "@/lib/api";
import type { AppConfig, Crosshair } from "@/types/config";

export function useConfigSave(config: AppConfig | null, setConfig: (cfg: AppConfig) => void) {
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  /** 防抖保存配置：拖滑块等连续操作时只在停止后 300ms 写入一次。 */
  const debouncedSave = useCallback(
    (cfg: AppConfig) => {
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
      saveTimerRef.current = setTimeout(() => {
        saveConfig(cfg).catch(console.error);
      }, 300);
    },
    []
  );

  const profile = config?.profiles[config.active_profile];
  const crosshair = profile?.crosshair;

  const updateCrosshair = useCallback(
    (patch: Partial<Crosshair>) => {
      if (!config || !profile || !crosshair) return;
      const newCrosshair = { ...crosshair, ...patch };
      const newProfile = { ...profile, crosshair: newCrosshair };
      const newConfig = {
        ...config,
        profiles: { ...config.profiles, [config.active_profile]: newProfile },
      };
      setConfig(newConfig);
      debouncedSave(newConfig);
    },
    [config, profile, crosshair, debouncedSave, setConfig]
  );

  const updateProfileTargetWindow = useCallback(
    (targetWindow: string) => {
      if (!config || !profile) return;
      const newConfig = {
        ...config,
        profiles: {
          ...config.profiles,
          [config.active_profile]: {
            ...profile,
            target_window: targetWindow,
          },
        },
      };
      setConfig(newConfig);
      debouncedSave(newConfig);
    },
    [config, profile, setConfig, debouncedSave]
  );

  const colorCss = useMemo(() => {
    const [r, g, b] = crosshair?.color || [1, 1, 1];
    return `rgb(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`;
  }, [crosshair?.color]);

  return {
    profile,
    crosshair,
    updateCrosshair,
    updateProfileTargetWindow,
    colorCss,
  };
}
