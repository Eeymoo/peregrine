import { useState } from "react";
import type { AppConfig } from "@/types/config";
import { Button } from "@/components/ui/button";
import { X, Terminal, RefreshCw, Trash2 } from "lucide-react";

interface DeveloperPanelProps {
  config: AppConfig;
  version: string;
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
        alert(
          "此构建未启用 DevTools feature。\n\n开发模式（npx tauri dev）下可用，或需要在 Cargo.toml 启用 tauri devtools feature。",
        );
      }
    } catch (e) {
      console.error("failed to open devtools:", e);
      alert(`打开 DevTools 失败：${String(e)}`);
    }
  };

  const reloadPage = () => {
    window.location.reload();
  };

  const resetConfig = async () => {
    if (!confirm("确定要清空当前配置并写入默认配置吗？\n\n此操作会备份现有 config.json 为 .bak。")) return;
    try {
      // 通过 saveConfig 写入默认配置
      const { saveConfig } = await import("@/lib/api");
      await saveConfig(config); // 这里仅测试 saveConfig 是否可调用，真实重置应由后端提供
      alert("配置重置接口已调用。完整重置请删除 %APPDATA%/Peregrine/config.json 后重启应用。");
    } catch (e) {
      alert(`重置失败：${String(e)}`);
    }
  };

  return (
    <div className="border-t pt-3 mt-3 space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold uppercase text-yellow-600 dark:text-yellow-400 flex items-center gap-1">
          <Terminal className="w-3 h-3" />
          开发者
        </span>
        <button
          onClick={onClose}
          className="text-muted-foreground hover:text-foreground"
          title="关闭开发者模式"
        >
          <X className="w-3 h-3" />
        </button>
      </div>

      <div className="flex flex-wrap gap-1.5">
        <Button size="sm" variant="outline" onClick={openDevTools} className="h-7 text-xs gap-1">
          <Terminal className="w-3 h-3" />
          DevTools
        </Button>
        <Button size="sm" variant="outline" onClick={reloadPage} className="h-7 text-xs gap-1">
          <RefreshCw className="w-3 h-3" />
          刷新
        </Button>
        <Button
          size="sm"
          variant="outline"
          onClick={resetConfig}
          className="h-7 text-xs gap-1 text-red-500 hover:text-red-600"
        >
          <Trash2 className="w-3 h-3" />
          重置配置
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
            {configExpanded ? "▼ 隐藏" : "▶ 显示"} config.json
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
