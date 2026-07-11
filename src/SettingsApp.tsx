import { useEffect, useState, useCallback } from "react";
import { useI18n, LANGUAGE_OPTIONS, type Locale } from "@/lib/i18n";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Separator } from "@/components/ui/separator";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { getConfig, updatePreferences } from "@/lib/api";
import type { AppConfig } from "@/types/config";

export default function SettingsApp() {
  const { t, locale, setLocale } = useI18n();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [autoSwitch, setAutoSwitchState] = useState<string>("ask");

  useEffect(() => {
    getCurrentWebviewWindow().setTitle(`${t("app.title")} ${t("settings.title")}`).catch(() => {});
  }, [t]);

  // 初始化时读取配置。
  useEffect(() => {
    getConfig()
      .then((cfg) => {
        setConfig(cfg);
        setAutoSwitchState(cfg.settings?.auto_switch_on_overlay ?? "ask");
      })
      .catch(console.error);
  }, []);

  /** 监听后端 settings 变更（来自配置窗口的对话框记住选择），同步本窗口状态。 */
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<{ auto_switch_on_overlay?: string }>(
          "peregrine:settings-changed",
          (event) => {
            const { auto_switch_on_overlay } = event.payload;
            if (auto_switch_on_overlay !== undefined) {
              setAutoSwitchState(auto_switch_on_overlay);
            }
          },
        );
      } catch { /* 非 Tauri 环境忽略 */ }
    })();
    return () => unlisten?.();
  }, []);

  const handleAutoSwitchChange = useCallback((value: string) => {
    setAutoSwitchState(value);
    if (!config) return;
    const newConfig: AppConfig = {
      ...config,
      settings: { ...config.settings, auto_switch_on_overlay: value },
    };
    setConfig(newConfig);
    updatePreferences({ auto_switch_on_overlay: value }).catch(console.error);
  }, [config]);

  return (
    <div className="h-screen flex flex-col bg-background text-foreground p-6">
      <h1 className="text-xl font-semibold">{t("settings.title")}</h1>
      <p className="text-sm text-muted-foreground mt-1">
        {t("settings.description")}
      </p>

      <Separator className="my-4" />

      {/* 语言设置 */}
      <div className="space-y-2">
        <Label className="text-sm">{t("settings.language")}</Label>
        <Select value={locale} onValueChange={(v) => setLocale(v as Locale)}>
          <SelectTrigger className="h-8 text-sm">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {LANGUAGE_OPTIONS.map((opt) => (
              <SelectItem key={opt.value} value={opt.value} className="text-sm">
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <Separator className="my-4" />

      {/* 开始覆盖时自动切换设置 */}
      <div className="space-y-2">
        <Label className="text-sm">{t("settings.autoSwitch")}</Label>
        <p className="text-xs text-muted-foreground">{t("settings.autoSwitchHint")}</p>
        <Select value={autoSwitch} onValueChange={handleAutoSwitchChange}>
          <SelectTrigger className="h-8 text-sm">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="ask" className="text-sm">{t("settings.autoSwitchAsk")}</SelectItem>
            <SelectItem value="yes" className="text-sm">{t("overlay.switchYes")}</SelectItem>
            <SelectItem value="no" className="text-sm">{t("overlay.switchNo")}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <Separator className="my-4" />

      <div className="space-y-3">
        <h2 className="text-lg font-medium">{t("settings.about.title")}</h2>
        <p className="text-sm text-muted-foreground">
          {t("settings.about.description")}
        </p>
        <ul className="text-sm space-y-1 text-muted-foreground">
          <li>{t("settings.about.version")}：0.1.1</li>
          <li>{t("settings.about.license")}：{t("license.polyform")}</li>
          <li>{t("settings.about.repository")}：https://github.com/Eeymoo/peregrine</li>
        </ul>
      </div>

      <div className="mt-auto text-xs text-muted-foreground text-right">
        Peregrine v0.1.1
      </div>
    </div>
  );
}
