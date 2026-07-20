import { useState, useEffect, useRef } from "react";
import type { AppConfig } from "@/types/config";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { X, Terminal, RefreshCw, Trash2 } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import {
  getEntries,
  clearEntries,
  subscribeEntries,
  type LogEntry,
} from "@/lib/actionLog";

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
 * - DevTools 切换开关（打开/关闭 WebView 检查器）
 * - 开发者日志（实时显示所有 actions 与 console 日志）
 * - 查看当前配置 JSON（只读）
 * - 一键重置配置（清空 + 默认）
 */
export function DeveloperPanel({ config, version, onClose }: DeveloperPanelProps) {
  const { t } = useI18n();
  const [configExpanded, setConfigExpanded] = useState(false);
  const [devToolsOpen, setDevToolsOpen] = useState(false);
  const [entries, setEntries] = useState<readonly LogEntry[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const logEndRef = useRef<HTMLDivElement>(null);

  // 订阅日志更新。
  useEffect(() => {
    setEntries(getEntries());
    const unsub = subscribeEntries(() => {
      setEntries([...getEntries()]);
    });
    return unsub;
  }, []);

  // 自动滚动到底部。
  useEffect(() => {
    if (autoScroll && logEndRef.current) {
      logEndRef.current.scrollTop = logEndRef.current.scrollHeight;
    }
  }, [entries, autoScroll]);

  const toggleDevTools = async (open: boolean) => {
    try {
      const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const win = getCurrentWebviewWindow() as unknown as {
        openDevTools?: () => Promise<void>;
        closeDevTools?: () => Promise<void>;
        isDevToolsOpen?: () => Promise<boolean>;
      };
      if (open) {
        if (typeof win.openDevTools === "function") {
          await win.openDevTools();
          setDevToolsOpen(true);
        } else {
          alert(t("developer.devToolsDisabled"));
        }
      } else {
        if (typeof win.closeDevTools === "function") {
          await win.closeDevTools();
        }
        setDevToolsOpen(false);
      }
    } catch (e) {
      alert(`${t("developer.openDevToolsFailed")}: ${String(e)}`);
    }
  };

  const reloadPage = () => {
    window.location.reload();
  };

  const resetConfig = async () => {
    if (!confirm(t("developer.resetConfirm"))) return;
    try {
      const { saveConfig } = await import("@/lib/api");
      await saveConfig(config);
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

      {/* DevTools 开关 */}
      <div className="flex items-center justify-between p-2 rounded bg-muted/50">
        <span className="text-xs flex items-center gap-1">
          <Terminal className="w-3 h-3" />
          {t("developer.devTools")}
        </span>
        <Switch checked={devToolsOpen} onCheckedChange={toggleDevTools} />
      </div>

      <div className="flex flex-wrap gap-1.5">
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

      {/* 开发者日志 */}
      <div className="space-y-1">
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium">{t("developer.logTitle")}</span>
          <div className="flex items-center gap-2">
            <label className="text-[10px] flex items-center gap-1 cursor-pointer">
              <input
                type="checkbox"
                checked={autoScroll}
                onChange={(e) => setAutoScroll(e.target.checked)}
                className="w-3 h-3"
              />
              {t("developer.autoScroll")}
            </label>
            <button
              onClick={clearEntries}
              className="text-[10px] text-muted-foreground hover:text-foreground"
              title={t("developer.clearLog")}
            >
              {t("developer.clearLog")}
            </button>
          </div>
        </div>
        <div
          ref={logEndRef}
          className="text-[10px] font-mono bg-black/80 text-green-400 p-2 rounded max-h-48 overflow-y-auto space-y-0.5"
        >
          {entries.length === 0 ? (
            <div className="text-gray-500 italic">{t("developer.logEmpty")}</div>
          ) : (
            entries.map((e) => (
              <div
                key={e.id}
                className={`leading-tight ${
                  e.level === "error"
                    ? "text-red-400"
                    : e.level === "warn"
                      ? "text-yellow-400"
                      : e.category === "action"
                        ? "text-cyan-400"
                        : e.category === "system"
                          ? "text-purple-400"
                          : "text-green-400"
                }`}
              >
                <span className="text-gray-500">{e.ts}</span>{" "}
                <span className="text-gray-400">[{e.category}]</span>{" "}
                <span>{e.message}</span>
                {e.detail !== undefined && (
                  <span className="text-gray-500">
                    {" "}
                    {typeof e.detail === "string"
                      ? e.detail
                      : JSON.stringify(e.detail)}
                  </span>
                )}
              </div>
            ))
          )}
        </div>
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
