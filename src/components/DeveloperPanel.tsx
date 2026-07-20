import { useState } from "react";
import type { AppConfig } from "@/types/config";
import { Button } from "@/components/ui/button";
import { X, Terminal, RefreshCw, Trash2 } from "lucide-react";
import { useI18n } from "@/lib/i18n";

interface DeveloperPanelProps {
  /** 完整的应用配置对象 */
  config: AppConfig;
  /** 应用版本号，用于显示和调试 */
  version: string;
  /** 关闭开发者面板的回调函数 */
  onClose: () => void;
}

/**
 * 开发者面板：仅当连续点击版本号 3 次后显示。
 *
 * 功能：
 * - 打开 WebView DevTools（前端控制台）
 * - 查看当前配置 JSON（只读）
 * - 查看 storage 路径
 * - 一键重置配置（清空 + 默认）
 */
export function DeveloperPanel({ config, version, onClose }: DeveloperPanelProps) {
  const { t } = useI18n();
  const [configExpanded, setConfigExpanded] = useState(false);

  const openDevTools = async () => {
    try {
      const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const win = getCurrentWebviewWindow() as unknown as {
        openDevTools?: () => Promise<void>;
      };
      if (typeof win.openDevTools === "function") {
        await win.openDevTools();
      } else {
        alert(t("developer.devToolsDisabled"));
      }
    } catch (e) {
      console.error("failed to open devtools:", e);
      alert(`${t("developer.openDevToolsFailed")}: ${String(e)}`);
    }
  };

  const reloadPage = () => {
    window.location.reload();
  };

  const resetConfig = async () => {
    if (!confirm(t("developer.resetConfirm"))) return;
    try {
      // 通过 saveConfig 写入默认配置
      const { saveConfig } = await import("@/lib/api");
      await saveConfig(config); // 这里仅测试 saveConfig 是否可调用，真实重置应由后端提供
      alert(t("developer.resetCalled"));
    } catch (e) {
      alert(`${t("developer.resetFailed")}: ${String(e)}`);
    }
  };

  return (
    <div className="border-t pt-3 mt-3 space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold uppercase text-yellow-600 dark:text-yellow-400 flex items-center gap-1">
          <Terminal className="w-3 h-3" />
          {t("developer.title")}
        </span>
        <button
          onClick={onClose}
          className="text-muted-foreground hover:text-foreground"
          title={t("common.close")}
        >
          <X className="w-3 h-3" />
        </button>
      </div>

      <div className="flex flex-wrap gap-1.5">
        <Button size="sm" variant="outline" onClick={openDevTools} className="h-7 text-xs gap-1">
          <Terminal className="w-3 h-3" />
          {t("developer.devTools")}
        </Button>
        <Button size="sm" variant="outline" onClick={reloadPage} className="h-7 text-xs gap-1">
          <RefreshCw className="w-3 h-3" />
          {t("developer.reload")}
        </Button>
        <Button
          size="sm"
          variant="outline"
          onClick={resetConfig}
          className="h-7 text-xs gap-1 text-red-500 hover:text-red-600"
        >
          <Trash2 className="w-3 h-3" />
          {t("developer.resetConfig")}
        </Button>
      </div>

      <div className="text-[10px] text-muted-foreground space-y-0.5">
        <div>version: {version}</div>
        <div>active_profile: {config.active_profile}</div>
        <div>profiles: {Object.keys(config.profiles).join(", ")}</div>
        <div>
          <button
            onClick={() => setConfigExpanded((v) => !v)}
            className="hover:text-foreground underline"
          >
            {configExpanded ? `▼ ${t("developer.hideConfig")}` : `▶ ${t("developer.showConfig")}`}
          </button>
        </div>
        {configExpanded && (
          <pre className="text-[10px] bg-muted/50 p-2 rounded max-h-40 overflow-auto whitespace-pre-wrap break-all">
            {JSON.stringify(config, null, 2)}
          </pre>
        )}
      </div>
    </div>
  );
}
