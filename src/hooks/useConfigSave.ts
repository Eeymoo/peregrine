import { useCallback, useEffect, useMemo, useRef } from "react";
import { saveConfig } from "@/lib/api";
import {
  createDefaultLayer,
  isProfileLegacyCompatible,
  layerToCrosshair,
  syncCrosshairToLayer,
} from "@/lib/layers";
import { getDefaultCrosshairForStyle } from "@/lib/presets";
import { useI18n } from "@/lib/i18n";
import type { AppConfig, Crosshair } from "@/types/config";

export function useConfigSave(
  config: AppConfig | null,
  setConfig: (cfg: AppConfig) => void,
) {
  const { t } = useI18n();
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  /** 防抖保存配置：拖滑块等连续操作时只在停止后 300ms 写入一次。 */
  const debouncedSave = useCallback((cfg: AppConfig) => {
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    saveTimerRef.current = setTimeout(() => {
      saveConfig(cfg).catch(console.error);
    }, 300);
  }, []);

  const profile = config?.profiles[config.active_profile];

  // 判断当前 active profile 是否可在单图层（旧版）UI 中编辑。
  const isLegacyCompatible = useMemo(
    () => isProfileLegacyCompatible(profile),
    [profile],
  );

  // 当后端配置是纯净新格式（crosshair 为 null）时，从 layers[0] 反向生成一个 crosshair，
  // 保证旧版单图层 UI 始终可用。后续编辑 crosshair 会同步回写 layers[0]。
  const crosshair = useMemo<Crosshair | null>(() => {
    const raw = profile?.crosshair ?? null;
    if (raw) return raw;
    const layer = profile?.layers?.[0];
    if (!layer) return null;
    return layerToCrosshair(layer);
  }, [profile]);

  // 用 ref 持有最新的 crosshair / config / profile，避免 useCallback 闭包陷阱：
  // 之前 updateCrosshair 的 deps 含 crosshair，连续拖动时下一次事件拿到的可能是旧闭包，
  // 导致「修改第二次的第一次才生效」的滞后现象。useEffect 同步 ref 始终指向最新值。
  const crosshairRef = useRef(crosshair);
  const configRef = useRef(config);
  const profileRef = useRef(profile);
  useEffect(() => {
    crosshairRef.current = crosshair;
  }, [crosshair]);
  useEffect(() => {
    configRef.current = config;
  }, [config]);
  useEffect(() => {
    profileRef.current = profile;
  }, [profile]);

  const hasLayers = (profile?.layers?.length ?? 0) > 0;

  // 稳定的 updateCrosshair：不依赖 config/profile/crosshair（全部走 ref），
  // 因此 useCallback 永远只创建一次，事件处理器始终拿到最新数据。
  const updateCrosshair = useCallback(
    (patch: Partial<Crosshair>, options?: { resetDefaults?: boolean }) => {
      const currentConfig = configRef.current;
      const currentProfile = profileRef.current;
      const currentCrosshair = crosshairRef.current;
      if (!currentConfig || !currentProfile) return;
      const newCrosshair =
        options?.resetDefaults && patch.style !== undefined
          ? getDefaultCrosshairForStyle(patch.style)
          : currentCrosshair
            ? { ...currentCrosshair, ...patch }
            : getDefaultCrosshairForStyle(patch.style ?? "cross");

      const newProfile = { ...currentProfile, crosshair: newCrosshair };
      // 如果已有 layers，同步更新 layers[0] 使其与 crosshair 一致。
      if (newProfile.layers.length > 0) {
        newProfile.layers = [
          syncCrosshairToLayer(
            newCrosshair,
            newProfile.layers[0],
            t(`styles.${newCrosshair.style}`),
          ),
          ...newProfile.layers.slice(1),
        ];
      } else {
        // 没有 layers 时创建一个单图层。
        newProfile.layers = [
          syncCrosshairToLayer(
            newCrosshair,
            createDefaultLayer(newCrosshair),
            t(`styles.${newCrosshair.style}`),
          ),
        ];
      }
      const newConfig = {
        ...currentConfig,
        profiles: { ...currentConfig.profiles, [currentConfig.active_profile]: newProfile },
      };
      // 立即同步 ref，避免同一事件循环内连续调用拿到旧值。
      configRef.current = newConfig;
      profileRef.current = newProfile;
      crosshairRef.current = newCrosshair;
      setConfig(newConfig);
      debouncedSave(newConfig);
    },
    [debouncedSave, setConfig, t],
  );

  const updateProfileTargetWindow = useCallback(
    (targetWindow: string) => {
      const currentConfig = configRef.current;
      const currentProfile = profileRef.current;
      if (!currentConfig || !currentProfile) return;
      const newConfig = {
        ...currentConfig,
        profiles: {
          ...currentConfig.profiles,
          [currentConfig.active_profile]: {
            ...currentProfile,
            target_window: targetWindow,
          },
        },
      };
      configRef.current = newConfig;
      profileRef.current = newConfig.profiles[currentConfig.active_profile];
      setConfig(newConfig);
      debouncedSave(newConfig);
    },
    [setConfig, debouncedSave],
  );

  const colorCss = useMemo(() => {
    const [r, g, b] = crosshair?.color || [1, 1, 1];
    return `#${Math.round(r * 255)
      .toString(16)
      .padStart(2, "0")}${Math.round(g * 255)
      .toString(16)
      .padStart(2, "0")}${Math.round(b * 255)
      .toString(16)
      .padStart(2, "0")}`;
  }, [crosshair?.color]);

  return {
    profile,
    crosshair,
    hasLayers,
    isLegacyCompatible,
    updateCrosshair,
    updateProfileTargetWindow,
    colorCss,
  };
}
