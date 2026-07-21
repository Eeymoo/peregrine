import { useState } from "react";
import { CheckCircle2, Loader2, Download, RefreshCw } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Card } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { MarkdownReleaseNotes } from "@/components/MarkdownReleaseNotes";
import { updatePreferences, checkForUpdate, downloadAndInstallUpdate } from "@/lib/api";
import type { AppConfig } from "@/types/config";

export interface UpdateState {
  status: "idle" | "checking" | "available" | "upToDate" | "updating" | "failed";
  version?: string;
  body?: string;
  progress?: number;
}

interface UpdateTabProps {
  config: AppConfig | null;
  resolvedLocale: string;
  updateState: UpdateState;
  setUpdateState: (state: UpdateState) => void;
  setConfig: (cfg: AppConfig) => void;
}

const MIRROR_PRESETS = [
  "https://gh-proxy.org",
  "https://v4.gh-proxy.org",
  "https://v6.gh-proxy.org",
  "https://cdn.gh-proxy.org",
];

export function UpdateTab({
  config,
  resolvedLocale,
  updateState,
  setUpdateState,
  setConfig,
}: UpdateTabProps) {
  const { t } = useI18n();
  const [customUrl, setCustomUrl] = useState(config?.settings?.mirror_url ?? "");

  const updateSetting = <K extends keyof AppConfig["settings"]>(
    key: K,
    value: AppConfig["settings"][K]
  ) => {
    if (!config) return;
    const newConfig: AppConfig = {
      ...config,
      settings: { ...config.settings, [key]: value },
    };
    setConfig(newConfig);
    updatePreferences({ [key]: value } as Partial<AppConfig["settings"]>).catch(console.error);
  };

  const handleCheckUpdate = async () => {
    setUpdateState({ status: "checking" });
    try {
      const channel = config?.settings?.update_channel ?? "stable";
      const cnMirror = config?.settings?.cn_mirror ?? false;
      const mirrorUrl = config?.settings?.mirror_url ?? "https://v4.gh-proxy.org";
      const result = await checkForUpdate(channel, cnMirror, mirrorUrl);
      if (result.available) {
        setUpdateState({ status: "available", version: result.version, body: result.body });
      } else {
        setUpdateState({ status: "upToDate" });
      }
    } catch (e) {
      console.error("[Update] check failed:", e);
      setUpdateState({ status: "failed" });
    }
  };

  const handleDownload = async () => {
    setUpdateState({ status: "updating", progress: 0 });
    try {
      const channel = config?.settings?.update_channel ?? "stable";
      const cnMirror = config?.settings?.cn_mirror ?? false;
      const mirrorUrl = config?.settings?.mirror_url ?? "https://v4.gh-proxy.org";
      await downloadAndInstallUpdate(channel, cnMirror, mirrorUrl, (downloaded, total) => {
        if (total > 0) {
          const pct = Math.min(100, Math.round((downloaded / total) * 100));
          setUpdateState({ status: "updating", progress: pct });
        }
      });
    } catch (e) {
      console.error("[Update] download failed:", e);
      setUpdateState({ status: "failed" });
    }
  };

  const mirrorUrl = config?.settings?.mirror_url ?? "https://v4.gh-proxy.org";
  const mirrorSelectValue = MIRROR_PRESETS.includes(mirrorUrl) ? mirrorUrl : "__custom__";
  const showCustomInput =
    config?.settings?.cn_mirror && !MIRROR_PRESETS.includes(mirrorUrl);

  return (
    <div className="space-y-6">
      {/* 中国大陆加速（仅中文显示） */}
      {resolvedLocale === "zh-CN" && (
        <>
          <div className="flex items-center justify-between gap-4">
            <div className="space-y-0.5">
              <Label className="text-sm font-medium">{t("settings.cnMirror")}</Label>
              <p className="text-xs text-muted-foreground">{t("settings.cnMirrorHint")}</p>
            </div>
            <Switch
              checked={config?.settings?.cn_mirror ?? false}
              onCheckedChange={(v) => updateSetting("cn_mirror", v)}
            />
          </div>

          {/* 加速站选择 */}
          {config?.settings?.cn_mirror && (
            <div className="flex items-center justify-between gap-4">
              <Label className="text-sm font-medium">{t("settings.mirrorSite")}</Label>
              <Select
                value={mirrorSelectValue}
                onValueChange={(v) => {
                  if (!config) return;
                  if (v === "__custom__") {
                    updateSetting("mirror_url", "");
                    setCustomUrl("");
                    return;
                  }
                  updateSetting("mirror_url", v);
                }}
              >
                <SelectTrigger className="w-48 h-8 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="https://gh-proxy.org">gh-proxy.org</SelectItem>
                  <SelectItem value="https://v4.gh-proxy.org">v4.gh-proxy.org（推荐）</SelectItem>
                  <SelectItem value="https://v6.gh-proxy.org">v6.gh-proxy.org</SelectItem>
                  <SelectItem value="https://cdn.gh-proxy.org">cdn.gh-proxy.org</SelectItem>
                  <SelectItem value="__custom__">{t("settings.mirrorCustom")}</SelectItem>
                </SelectContent>
              </Select>
            </div>
          )}

          {/* 自定义镜像地址输入 */}
          {showCustomInput && (
            <div className="flex items-center justify-between gap-4">
              <Label className="text-sm font-medium">{t("settings.mirrorCustomUrl")}</Label>
              <input
                type="text"
                className="flex h-8 w-48 rounded-md border border-input bg-background px-2 text-xs ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                value={customUrl}
                placeholder="https://your-mirror.example.com"
                onChange={(e) => {
                  setCustomUrl(e.target.value);
                  if (!config) return;
                  const newConfig: AppConfig = {
                    ...config,
                    settings: { ...config.settings, mirror_url: e.target.value },
                  };
                  setConfig(newConfig);
                }}
                onBlur={(e) => {
                  const val = e.target.value.trim();
                  if (val && config) {
                    updateSetting("mirror_url", val);
                  }
                }}
              />
            </div>
          )}
        </>
      )}

      {/* 更新通道 */}
      <div className="flex items-center justify-between gap-4">
        <Label className="text-sm font-medium">{t("settings.updateChannel")}</Label>
        <Select
          value={config?.settings?.update_channel ?? "stable"}
          onValueChange={(v) => updateSetting("update_channel", v)}
        >
          <SelectTrigger className="w-40 h-8 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="stable">{t("settings.updateChannelStable")}</SelectItem>
            <SelectItem value="prerelease">{t("settings.updateChannelPrerelease")}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* 检查更新按钮 */}
      <Button
        className="w-full"
        size="xs"
        disabled={updateState.status === "checking" || updateState.status === "updating"}
        onClick={handleCheckUpdate}
      >
        {updateState.status === "checking" ? (
          <><Loader2 className="h-3.5 w-3.5 animate-spin" /> {t("settings.checking") || "..."}</>
        ) : (
          <><RefreshCw className="h-3.5 w-3.5" /> {t("settings.checkUpdate")}</>
        )}
      </Button>

      {/* 结果区域 */}
      {updateState.status === "failed" && (
        <Card className="bg-muted/50 p-4">
          <p className="text-sm text-red-500">{t("settings.updateFailed")}</p>
        </Card>
      )}

      {updateState.status === "upToDate" && (
        <Card className="bg-muted/50 p-4">
          <p className="text-sm text-green-600 flex items-center gap-1.5">
            <CheckCircle2 className="h-4 w-4" />
            {t("settings.updateUpToDate")}
          </p>
        </Card>
      )}

      {updateState.status === "updating" && (
        <Card className="bg-muted/50 p-4 space-y-2">
          <p className="text-sm text-blue-500">{t("settings.updating")}</p>
          <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full bg-blue-500 rounded-full transition-all"
              style={{ width: updateState.progress !== undefined ? `${updateState.progress}%` : "30%" }}
            />
          </div>
          {updateState.progress !== undefined && (
            <p className="text-xs text-muted-foreground text-right">
              {Math.round(updateState.progress)}%
            </p>
          )}
        </Card>
      )}

      {updateState.status === "available" && (
        <Card className="bg-muted/50 p-4 space-y-3">
          <p className="text-sm font-medium">
            {t("settings.updateAvailable")}：v{updateState.version}
          </p>
          {updateState.body && (
            <MarkdownReleaseNotes body={updateState.body} />
          )}
          <div className="flex gap-2">
            <Button size="xs" onClick={handleDownload}>
              <Download className="h-3.5 w-3.5" />
              {t("settings.updateNow")}
            </Button>
            <Button
              variant="outline"
              size="xs"
              onClick={() => setUpdateState({ status: "idle" })}
            >
              {t("settings.updateLater")}
            </Button>
          </div>
        </Card>
      )}
    </div>
  );
}
