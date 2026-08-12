import { useState } from "react";
import { useI18n, LANGUAGE_OPTIONS, detectLocale, type Locale } from "@/lib/i18n";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { updatePreferences, relaunchApp } from "@/lib/api";
import {
  TELEMETRY_DSN_AVAILABLE,
  TELEMETRY_PENDING_RESTART_KEY,
  restartApp,
} from "@/lib/telemetry";
import type { AppConfig } from "@/types/config";

interface GeneralTabProps {
  config: AppConfig | null;
  locale: Locale;
  setConfig: (cfg: AppConfig) => void;
  setLocale: (locale: Locale) => void;
}

export function GeneralTab({ config, locale, setConfig, setLocale }: GeneralTabProps) {
  const { t } = useI18n();
  // 遥测开关「稍后重启」待生效标记（重启后由 main.tsx 清除）。
  const [telemetryPendingRestart, setTelemetryPendingRestart] = useState<boolean>(
    () => localStorage.getItem(TELEMETRY_PENDING_RESTART_KEY) === "1",
  );

  return (
    <div className="space-y-6">
      {/* 语言 */}
      <div className="flex items-center justify-between gap-4">
        <Label className="text-sm font-medium">{t("settings.language")}</Label>
        <Select
          value={locale}
          onValueChange={(v) => {
            const next = v as Locale;
            setLocale(next);
            // 非中文语言强制关闭中国大陆加速镜像。
            const resolved = next === "auto" ? detectLocale() : next;
            if (resolved !== "zh-CN" && config) {
              const newConfig: AppConfig = {
                ...config,
                settings: { ...config.settings, cn_mirror: false },
              };
              setConfig(newConfig);
              updatePreferences({ cn_mirror: false }).catch(() => {});
            }
          }}
        >
          <SelectTrigger className="w-40 h-8 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {LANGUAGE_OPTIONS.map((opt) => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.labelKey ? t(opt.labelKey) : opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {/* GPU 加速 */}
      <div className="flex items-center justify-between gap-4">
        <div className="space-y-0.5">
          <Label className="text-sm font-medium">{t("settings.gpuAcceleration")}</Label>
          <p className="text-xs text-muted-foreground">
            {t("settings.gpuAccelerationHint")}
          </p>
        </div>
        <Switch
          checked={config?.settings?.gpu_acceleration ?? false}
          onCheckedChange={async (v) => {
            if (!config) return;
            const newConfig: AppConfig = {
              ...config,
              settings: { ...config.settings, gpu_acceleration: v },
            };
            setConfig(newConfig);
            await updatePreferences({ gpu_acceleration: v });
            try {
              const { ask } = await import("@tauri-apps/plugin-dialog");
              const confirmed = await ask(t("settings.gpuRestartDesc"), {
                title: t("settings.gpuRestartTitle"),
                okLabel: t("settings.gpuRestartNow"),
                cancelLabel: t("settings.gpuRestartLater"),
                kind: "info",
              });
              if (confirmed) {
                await relaunchApp();
              }
            } catch (e) {
              console.error("[GPU] dialog/relaunch failed:", e);
            }
          }}
        />
      </div>

      {/* 遥测开关（匿名崩溃上报 + 启动统计）。
          编译期禁用构建（未注入 DSN）下整块隐藏。
          「测试上报」按钮已迁至「开发」Tab（仅开发者模式可见）。 */}
      {TELEMETRY_DSN_AVAILABLE && (
        <div className="flex items-center justify-between gap-4">
          <div className="space-y-0.5">
            <Label className="text-sm font-medium">{t("settings.telemetry")}</Label>
            <p className="text-xs text-muted-foreground">
              {t("settings.telemetryHint")}
            </p>
            {telemetryPendingRestart && (
              <p className="text-xs text-amber-600">
                {t("settings.telemetryPendingRestart")}
              </p>
            )}
          </div>
          <Switch
            checked={config?.settings?.telemetry_enabled ?? false}
            onCheckedChange={async (v) => {
              if (!config) return;
              const newConfig: AppConfig = {
                ...config,
                settings: { ...config.settings, telemetry_enabled: v },
              };
              setConfig(newConfig);
              await updatePreferences({ telemetry_enabled: v });
              try {
                const { ask } = await import("@tauri-apps/plugin-dialog");
                const confirmed = await ask(t("settings.telemetryRestartDesc"), {
                  title: t("settings.telemetryRestartTitle"),
                  okLabel: t("settings.telemetryRestartNow"),
                  cancelLabel: t("settings.telemetryRestartLater"),
                  kind: "info",
                });
                if (confirmed) {
                  await restartApp();
                } else {
                  // 稍后重启：保留「待重启生效」标记直至下次重启。
                  localStorage.setItem(TELEMETRY_PENDING_RESTART_KEY, "1");
                  setTelemetryPendingRestart(true);
                }
              } catch (e) {
                console.error("[telemetry] dialog/restart failed:", e);
              }
            }}
          />
        </div>
      )}
    </div>
  );
}

