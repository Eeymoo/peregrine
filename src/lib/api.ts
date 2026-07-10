import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "@/types/config";

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke("save_config", { config });
}

export async function listWindowTitles(): Promise<string[]> {
  return invoke<string[]>("list_window_titles");
}

export async function startOverlay(targetWindow: string): Promise<void> {
  return invoke("start_overlay", { targetWindow });
}

export async function stopOverlay(): Promise<void> {
  return invoke("stop_overlay");
}

export async function pickImagePath(): Promise<string | null> {
  return invoke<string | null>("pick_image_path");
}

export async function getOverlayActive(): Promise<boolean> {
  return invoke<boolean>("get_overlay_active");
}

export async function focusTargetWindow(targetWindow: string): Promise<void> {
  return invoke("focus_target_window", { targetWindow });
}

/** 更新应用级偏好设置（locale / auto_switch_on_overlay）。 */
export async function updatePreferences(
  preferences: Partial<AppConfig["settings"]>,
): Promise<void> {
  return invoke("update_preferences", { preferences });
}
