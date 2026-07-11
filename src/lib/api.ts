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
 * channel: "stable"（正式版）或 "prerelease"（尝鲜版）。
 * 正式版从 releases/latest/download/stable.json 检查。
 * 尝鲜版通过 GitHub API 查找最新 prerelease，再获取对应 tag 的 prerelease.json。
 */
export async function checkForUpdate(
  channel: string = "stable"
): Promise<{ available: boolean; version?: string; body?: string }> {
  const { check } = await import("@tauri-apps/plugin-updater");
  // updater 插件的 endpoints 在编译时固定，这里直接用默认 endpoint 检查。
  // 通道区分通过自定义逻辑实现：先查 GitHub API 获取最新版本信息。
  try {
    const update = await check();
    if (update) {
      // 正式版通道：直接返回。
      if (channel === "stable") {
        return { available: true, version: update.version, body: update.body };
      }
      // 尝鲜版通道：需要从 GitHub API 查最新 prerelease。
      // updater 默认 endpoint 指向 stable，这里单独查 prerelease。
    }
  } catch {
    // 默认 endpoint 可能不可达，继续走 API 方式。
  }

  // 通用 fallback：通过 GitHub API 查找对应通道的最新版本。
  const url =
    channel === "prerelease"
      ? "https://api.github.com/repos/Eeymoo/peregrine/releases"
      : "https://api.github.com/repos/Eeymoo/peregrine/releases/latest";

  const resp = await fetch(url);
  if (!resp.ok) return { available: false };

  const currentVersion = await getAppVersion();
  let latestTag: string;
  let notes: string;

  if (channel === "prerelease") {
    const releases = await resp.json();
    const preRelease = releases.find((r: any) => r.prerelease === true);
    if (!preRelease) return { available: false };
    latestTag = preRelease.tag_name;
    notes = preRelease.body || "";
  } else {
    const release = await resp.json();
    latestTag = release.tag_name;
    notes = release.body || "";
  }

  const latestVersion = latestTag.replace(/^v/, "");
  if (compareVersions(latestVersion, currentVersion) > 0) {
    return { available: true, version: latestVersion, body: notes };
  }
  return { available: false };
}

/** 下载并安装更新，完成后自动重启。
 * onProgress: 可选回调，接收已下载字节数和总字节数。
 */
export async function downloadAndInstallUpdate(
  onProgress?: (downloaded: number, total: number) => void
): Promise<void> {
  const { check } = await import("@tauri-apps/plugin-updater");
  const update = await check();
  if (update) {
    let total = 0;
    let downloaded = 0;
    await update.downloadAndInstall((event: any) => {
      switch (event.event) {
        case "Started":
          total = event.data.contentLength ?? 0;
          break;
        case "Progress":
          downloaded += event.data.chunkLength ?? 0;
          break;
        case "Finished":
          downloaded = total;
          break;
      }
      if (onProgress) onProgress(downloaded, total);
    });
    await relaunchApp();
  }
}

/** 简单的语义版本比较，返回 1（a>b）、0（a==b）、-1（a<b）。 */
function compareVersions(a: string, b: string): number {
  const parseVer = (v: string) => {
    const clean = v.replace(/^v/, "").split("-")[0];
    return clean.split(".").map((n) => parseInt(n, 10) || 0);
  };
  const va = parseVer(a);
  const vb = parseVer(b);
  for (let i = 0; i < Math.max(va.length, vb.length); i++) {
    const da = va[i] || 0;
    const db = vb[i] || 0;
    if (da > db) return 1;
    if (da < db) return -1;
  }
  return 0;
}
