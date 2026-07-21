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
import type { AppConfig } from "@/types/config";

interface GeneralTabProps {
  config: AppConfig | null;
  locale: Locale;
  setConfig: (cfg: AppConfig) => void;
  setLocale: (locale: Locale) => void;
}

export function GeneralTab({ config, locale, setConfig, setLocale }: GeneralTabProps) {
  const { t } = useI18n();

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
              updatePreferences({ cn_mirror: false }).catch(console.error);
            }
          }}
        >
          <SelectTrigger className="w-40 h-8 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {LANGUAGE_OPTIONS.map((opt) => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.label}
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
    </div>
  );
}
