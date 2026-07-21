import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type {
  AppConfig,
  BuiltShape,
  Layer,
  LayerPatch,
  MaterialInfo,
  Profile,
} from "@/types/config";

export { getCurrentWebviewWindow };

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
 * cn_mirror: 是否使用中国大陆加速镜像。
 * mirror_url: 镜像站地址（如 "https://v4.gh-proxy.org"）。
 */
export async function checkForUpdate(
  channel: string = "stable",
  cnMirror: boolean = false,
  mirrorUrl: string = ""
): Promise<{ available: boolean; version?: string; body?: string }> {
  return invoke<{ available: boolean; version?: string; body?: string }>(
    "check_update",
    { channel, cnMirror, mirrorUrl }
  );
}

/** 下载并安装更新，完成后自动重启。
 * channel 决定更新通道，cnMirror/mirrorUrl 决定加速镜像。
 * onProgress: 可选回调，接收已下载字节数和总字节数。
 */
export async function downloadAndInstallUpdate(
  channel: string,
  cnMirror: boolean,
  mirrorUrl: string,
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
    channel,
    cnMirror,
    mirrorUrl,
    onEvent: channel_,
  });
}

/** 设置当前 active profile 的准心颜色（快捷键颜色切换共用此逻辑）。 */
export async function setCrosshairColor(
  color: [number, number, number, number]
): Promise<void> {
  return invoke("set_crosshair_color", { color });
}

// ===== 四层架构：图层 / 物料 API =====

/** 计算当前激活 Profile 的图元列表（供前端预览绘制）。 */
export async function buildShapes(
  screenW: number,
  screenH: number,
): Promise<BuiltShape[]> {
  return invoke<BuiltShape[]>("build_shapes_ipc", { screenW, screenH });
}

/** 列出全部已注册物料（内置 + 用户）。 */
export async function listMaterials(): Promise<MaterialInfo[]> {
  return invoke<MaterialInfo[]>("list_materials");
}

/** 在当前激活 Profile 末尾添加图层。 */
export async function addLayer(materialId: string, name: string): Promise<Layer> {
  return invoke<Layer>("add_layer", { materialId, name });
}

/** 删除指定 id 的图层。 */
export async function removeLayer(layerId: string): Promise<void> {
  return invoke("remove_layer", { layerId });
}

/** 调整图层顺序。 */
export async function moveLayer(layerId: string, newIndex: number): Promise<void> {
  return invoke("move_layer", { layerId, newIndex });
}

/** 复制图层（生成新 id）。 */
export async function duplicateLayer(layerId: string): Promise<Layer> {
  return invoke<Layer>("duplicate_layer", { layerId });
}

/** 批量更新图层字段。 */
export async function updateLayer(
  layerId: string,
  patch: LayerPatch,
): Promise<void> {
  return invoke("update_layer", { layerId, patch });
}

/** 列出当前激活 Profile 的全部图层。 */
export async function listLayers(): Promise<Layer[]> {
  return invoke<Layer[]>("list_layers");
}

// ===== Profile 管理 API =====

export async function listProfiles(): Promise<string[]> {
  return invoke<string[]>("list_profiles");
}

export async function createProfile(name: string): Promise<Profile> {
  return invoke<Profile>("create_profile", { name });
}

export async function renameProfile(oldName: string, newName: string): Promise<void> {
  return invoke("rename_profile", { oldName, newName });
}

export async function deleteProfile(name: string): Promise<void> {
  return invoke("delete_profile", { name });
}

export async function setActiveProfile(name: string): Promise<void> {
  return invoke("set_active_profile", { name });
}

export async function getProfile(name: string): Promise<Profile> {
  return invoke<Profile>("get_profile", { name });
}

export async function isProfileLegacyCompatible(profile: Profile): Promise<boolean> {
  return invoke<boolean>("is_profile_legacy_compatible", { profile });
}

export async function getActiveProfileName(): Promise<string> {
  return invoke<string>("get_active_profile_name");
}

export async function copyProfile(baseName: string): Promise<string> {
  return invoke<string>("copy_profile", { baseName });
}

