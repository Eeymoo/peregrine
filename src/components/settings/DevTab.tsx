import { useState } from "react";
import { useI18n } from "@/lib/i18n";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { updatePreferences, relaunchApp } from "@/lib/api";
import { TELEMETRY_DSN_AVAILABLE, testReport } from "@/lib/telemetry";
import type { AppConfig } from "@/types/config";

interface DevTabProps {
  config: AppConfig;
  setConfig: (cfg: AppConfig) => void;
}

/**
 * 「开发」Tab：仅在开发者模式已解锁（或开发构建）时显示。
 *
 * - DevTools switch：控制 `developer_mode`，切换后弹窗提示重启生效。
 *   开启后重开窗口右键可检查、Ctrl+Shift+I 可用；关闭后完全禁用。
 * - 测试上报：触发一条 Error 级测试事件（仅 DSN 可用时显示）。
 */
export function DevTab({ config, setConfig }: DevTabProps) {
  const { t } = useI18n();
  const [testReportState, setTestReportState] = useState<"idle" | "sending" | "done">("idle");

  return (
    <div className="space-y-6">
      {/* DevTools 开关：参考 GPU 加速 switch 交互 */}
      <div className="flex items-center justify-between gap-4">
        <div className="space-y-0.5">
          <Label className="text-sm font-medium">{t("settings.devTools")}</Label>
          <p className="text-xs text-muted-foreground">
            {t("settings.devToolsHint")}
          </p>
        </div>
        <Switch
          checked={config.settings.developer_mode ?? false}
          onCheckedChange={async (v) => {
            // 先弹窗确认重启，再写入配置（避免 DevTab 因 developer_mode=false 立即消失）。
            try {
              const { ask } = await import("@tauri-apps/plugin-dialog");
              const confirmed = await ask(t("settings.devToolsRestartDesc"), {
                title: t("settings.devToolsRestartTitle"),
                okLabel: t("settings.gpuRestartNow"),
                cancelLabel: t("settings.gpuRestartLater"),
                kind: "info",
              });
              const newConfig: AppConfig = {
                ...config,
                settings: { ...config.settings, developer_mode: v },
              };
              setConfig(newConfig);
              await updatePreferences({ developer_mode: v });
              if (confirmed) {
                await relaunchApp();
              }
            } catch (e) {
              console.error("[DevTools] dialog/relaunch failed:", e);
            }
          }}
        />
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
