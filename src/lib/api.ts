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

/** 获取应用版本号（从 Cargo.toml / tauri.conf.json 继承）。 */
export async function getAppVersion(): Promise<string> {
  return invoke<string>("get_app_version");
}

/** 重启应用（GPU 加速等设置变更后需要重建 WebView2）。 */
export async function relaunchApp(): Promise<void> {
  return invoke("relaunch_app");
}
