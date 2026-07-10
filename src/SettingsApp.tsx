import { useEffect } from "react";
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

export default function SettingsApp() {
  const { t, locale, setLocale } = useI18n();

  useEffect(() => {
    getCurrentWebviewWindow().setTitle(`${t("app.title")} ${t("settings.title")}`).catch(() => {});
  }, [t]);

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

      <div className="space-y-3">
        <h2 className="text-lg font-medium">{t("settings.about.title")}</h2>
        <p className="text-sm text-muted-foreground">
          {t("settings.about.description")}
        </p>
        <ul className="text-sm space-y-1 text-muted-foreground">
          <li>{t("settings.about.version")}：0.1.1</li>
          <li>{t("settings.about.license")}：{t("license.polyform")}</li>
          <li>{t("settings.about.repository")}：https://github.com/eeymoo/peregrine</li>
        </ul>
      </div>

      <div className="mt-auto text-xs text-muted-foreground text-right">
        Peregrine v0.1.1
      </div>
    </div>
  );
}
