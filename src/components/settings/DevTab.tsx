import { useState } from "react";
import { useI18n } from "@/lib/i18n";
import { Label } from "@/components/ui/label";
import { TELEMETRY_DSN_AVAILABLE, testReport } from "@/lib/telemetry";

/**
 * 「开发」Tab：仅在开发者模式已解锁（或开发构建）时显示。
 *
 * 仅包含两个区块：
 * - 「开启 DevTools」：调用前端 `getCurrentWebviewWindow().openDevTools()`。
 *   未解锁构建（DevTools feature 关闭）会失败，提示用户重开窗口。
 * - 「测试上报」：触发一条 Error 级测试事件（仅 DSN 可用时显示）。
 */
export function DevTab() {
  const { t } = useI18n();
  const [testReportState, setTestReportState] = useState<"idle" | "sending" | "done">("idle");

  const openDevTools = async () => {
    try {
      const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const win = getCurrentWebviewWindow() as unknown as {
        openDevTools?: () => Promise<void>;
      };
      if (typeof win.openDevTools === "function") {
        await win.openDevTools();
      } else {
        alert(t("settings.devToolsDisabled"));
      }
    } catch (e) {
      alert(`${t("settings.devOpenDevToolsFailed")}: ${String(e)}`);
    }
  };

  return (
    <div className="space-y-6">
      {/* 开启 DevTools */}
      <div className="flex items-center justify-between gap-4">
        <div className="space-y-0.5">
          <Label className="text-sm font-medium">{t("settings.devOpenDevTools")}</Label>
          <p className="text-xs text-muted-foreground">{t("settings.devOpenDevToolsHint")}</p>
        </div>
        <button
          type="button"
          className="px-3 py-1.5 rounded bg-muted text-muted-foreground text-xs font-medium"
          onClick={openDevTools}
        >
          {t("settings.devOpenDevTools")}
        </button>
      </div>

      {/* 测试上报：仅在编译期注入了遥测 DSN 的构建中显示。 */}
      {TELEMETRY_DSN_AVAILABLE && (
        <div className="flex items-center justify-between gap-4">
          <div className="space-y-0.5">
            <Label className="text-sm font-medium">{t("settings.telemetryTestReport")}</Label>
            <p className="text-xs text-muted-foreground">
              {t("settings.telemetryTestReportHint")}
            </p>
          </div>
          <button
            type="button"
            disabled={testReportState === "sending"}
            className="px-3 py-1.5 rounded bg-muted text-muted-foreground text-xs font-medium disabled:opacity-50"
            onClick={async () => {
              setTestReportState("sending");
              try {
                await testReport();
                setTestReportState("done");
                setTimeout(() => setTestReportState("idle"), 3000);
              } catch (e) {
                alert(`${t("settings.telemetryTestReportFailed")}: ${String(e)}`);
                setTestReportState("idle");
              }
            }}
          >
            {testReportState === "done"
              ? t("settings.telemetryTestReportDone")
              : t("settings.telemetryTestReport")}
          </button>
        </div>
      )}
    </div>
  );
}
