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

/** 检查是否有可用更新。
 * source: "github" 或 "gitee"。
 * channel: "stable"（正式版）或 "prerelease"（尝鲜版）。
 * 通过 Rust 自定义 command 实现动态 endpoint 切换。
 */
export async function checkForUpdate(
  source: string = "github",
  channel: string = "stable"
): Promise<{ available: boolean; version?: string; body?: string }> {
  return invoke<{ available: boolean; version?: string; body?: string }>(
    "check_update",
    { source, channel }
  );
}

/** 下载并安装更新，完成后自动重启。
 * source/channel 决定从哪个源下载。
 * onProgress: 可选回调，接收已下载字节数和总字节数。
 */
export async function downloadAndInstallUpdate(
  source: string,
  channel: string,
  onProgress?: (downloaded: number, total: number) => void
): Promise<void> {
  const { Channel } = await import("@tauri-apps/api/core");
  const channel_ = new Channel<{
    event: string;
    data: { contentLength?: number; chunkLength?: number };
  }>();
  let total = 0;
  let downloaded = 0;
  channel_.onmessage = (msg) => {
    switch (msg.event) {
      case "Started":
        total = msg.data.contentLength ?? 0;
        break;
      case "Progress":
        downloaded += msg.data.chunkLength ?? 0;
        break;
      case "Finished":
        downloaded = total;
        break;
    }
    if (onProgress) onProgress(downloaded, total);
  };
  await invoke("download_install_update", {
    source,
    channel,
    onEvent: channel_,
  });
}
