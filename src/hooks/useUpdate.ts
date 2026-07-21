import { useEffect, useState } from "react";
import { checkForUpdate, downloadAndInstallUpdate } from "@/lib/api";

export interface UpdateInfo {
  version: string;
  body?: string;
}

export interface UseUpdateOptions {
  autoCheckDelayMs?: number;
  onStart?: () => void;
  onProgress?: (progress: number) => void;
  onError?: (error: unknown) => void;
}

export function useUpdate(
  config: { settings?: { update_channel?: string; cn_mirror?: boolean; mirror_url?: string } } | null,
  options: UseUpdateOptions = {}
) {
  const { autoCheckDelayMs = 3000, onStart, onProgress, onError } = options;
  const [updateAvailable, setUpdateAvailable] = useState<UpdateInfo | null>(null);
  const [updating, setUpdating] = useState(false);
  const [updateProgress, setUpdateProgress] = useState(0);

  useEffect(() => {
    const autoCheck = async () => {
      try {
        await new Promise((r) => setTimeout(r, autoCheckDelayMs));
        if (!config) return;
        const channel = config.settings?.update_channel ?? "stable";
        const cnMirror = config.settings?.cn_mirror ?? false;
        const mirrorUrl = config.settings?.mirror_url ?? "https://v4.gh-proxy.org";
        const result = await checkForUpdate(channel, cnMirror, mirrorUrl);
        if (result.available) {
          setUpdateAvailable({ version: result.version || "", body: result.body });
        }
      } catch {
        /* 静默失败 */
      }
    };
    autoCheck();
  }, []);

  const startUpdate = async () => {
    if (!config) return;
    setUpdating(true);
    setUpdateProgress(0);
    onStart?.();
    try {
      const channel = config.settings?.update_channel ?? "stable";
      const cnMirror = config.settings?.cn_mirror ?? false;
      const mirrorUrl = config.settings?.mirror_url ?? "https://v4.gh-proxy.org";
      await downloadAndInstallUpdate(channel, cnMirror, mirrorUrl, (downloaded, total) => {
        if (total > 0) {
          const pct = Math.min(100, Math.round((downloaded / total) * 100));
          setUpdateProgress(pct);
          onProgress?.(pct);
        }
      });
    } catch (e) {
      console.error("[Update] download failed:", e);
      setUpdating(false);
      setUpdateProgress(0);
      onError?.(e);
    }
  };

  return {
    updateAvailable,
    setUpdateAvailable,
    updating,
    updateProgress,
    setUpdateProgress,
    startUpdate,
  };
}
