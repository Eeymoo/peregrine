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
    // focusTargetWindow 失败不阻塞隐藏窗口流程；invoke 包装会 toast 提示。
    focusTargetWindow(targetWindow).catch(() => {});
    await getCurrentWebviewWindow().destroy();
  }, []);

  const handleStartOverlay = useCallback(async () => {
    const isFullscreen = config?.settings.fullscreen_overlay ?? true;
    if (!isFullscreen && !profile?.target_window) return;
    // 渲染不变量前置兜底：无可渲染内容时不发起请求（后端 start_overlay 硬校验兜底）。
    // 判定与渲染路径一致：layers 非空只看可见层；crosshair 仅在 layers 为空时兜底。
    const hasRenderable =
      (profile?.layers ?? []).length > 0
        ? (profile?.layers ?? []).some((l) => l.visible)
        : !!profile?.crosshair;
    if (!hasRenderable) return;
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
    } catch {
      // invoke 包装负责 toast；这里不再 console.error。
    }
  }, [
    config?.settings.fullscreen_overlay,
    config?.settings.auto_switch_on_overlay,
    profile?.target_window,
    profile?.crosshair,
    profile?.layers,
    setOverlayActive,
    hideAndSwitch,
    onAskAutoSwitch,
  ]);

  const handleStopOverlay = useCallback(async () => {
    try {
      await stopOverlay();
      setOverlayActive(false);
    } catch {
      // invoke 包装负责 toast。
    }
  }, [setOverlayActive]);

  const saveAutoSwitchPreference = useCallback(
    (value: "yes" | "no", targetWindow?: string) => {
      // updatePreferences 失败由 invoke 包装 toast；这里不阻塞 hideAndSwitch。
      updatePreferences({ auto_switch_on_overlay: value }).catch(() => {});
      if (targetWindow) {
        hideAndSwitch(targetWindow).catch(() => {});
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
