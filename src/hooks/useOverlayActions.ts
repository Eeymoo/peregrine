import { useCallback } from "react";
import {
  startOverlay,
  stopOverlay,
  focusTargetWindow,
  getCurrentWebviewWindow,
} from "@/lib/api";
import { updatePreferences } from "@/lib/api";
import type { AppConfig } from "@/types/config";

export function useOverlayActions(
  config: AppConfig | null,
  setOverlayActive: (active: boolean) => void,
  onAskAutoSwitch: () => void
) {
  const profile = config?.profiles[config.active_profile];

  const hideAndSwitch = useCallback(async (targetWindow: string) => {
    focusTargetWindow(targetWindow).catch(console.error);
    await getCurrentWebviewWindow().destroy();
  }, []);

  const handleStartOverlay = useCallback(async () => {
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
        onAskAutoSwitch();
      }
    } catch (e) {
      console.error(e);
    }
  }, [
    config?.settings.fullscreen_overlay,
    config?.settings.auto_switch_on_overlay,
    profile?.target_window,
    setOverlayActive,
    hideAndSwitch,
    onAskAutoSwitch,
  ]);

  const handleStopOverlay = useCallback(async () => {
    try {
      await stopOverlay();
      setOverlayActive(false);
    } catch (e) {
      console.error(e);
    }
  }, [setOverlayActive]);

  const saveAutoSwitchPreference = useCallback(
    (value: "yes" | "no", targetWindow?: string) => {
      updatePreferences({ auto_switch_on_overlay: value }).catch(console.error);
      if (targetWindow) {
        hideAndSwitch(targetWindow).catch(console.error);
      }
    },
    [hideAndSwitch]
  );

  return {
    handleStartOverlay,
    handleStopOverlay,
    saveAutoSwitchPreference,
  };
}
