import { useI18n } from "@/lib/i18n";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import appIcon from "../../../assets/icon.png";

interface AboutTabProps {
  version: string;
}

export function AboutTab({ version }: AboutTabProps) {
  const { t } = useI18n();

  return (
    <div className="space-y-6">
      {/* 头部 */}
      <div className="text-center space-y-2">
        <img
          src={appIcon}
          alt="Peregrine"
          className="w-16 h-16 mx-auto rounded-2xl"
        />
        <h2 className="text-xl font-bold">Peregrine</h2>
        <p className="text-sm text-muted-foreground leading-relaxed">
          {t("settings.about.description")}
        </p>
      </div>

      <Separator />

      {/* 信息列表 */}
      <div className="space-y-2">
        <div className="flex justify-between text-sm">
          <span className="text-muted-foreground">{t("settings.about.version")}</span>
          <span>{version || "..."}</span>
        </div>
        <div className="flex justify-between text-sm">
          <span className="text-muted-foreground">{t("settings.about.publisher")}</span>
          <span>Eeymoo</span>
        </div>
        <div className="flex justify-between text-sm">
          <span className="text-muted-foreground">{t("settings.about.license")}</span>
          <span>{t("license.mit")}</span>
        </div>
        <div className="flex justify-between items-center text-sm">
          <span className="text-muted-foreground">{t("settings.about.repository")}</span>
          <Button
            variant="link"
            size="xs"
            className="p-0 h-auto"
            onClick={() => {
              if (typeof window !== "undefined" && window.open) {
                window.open("https://github.com/Eeymoo/peregrine", "_blank");
              }
            }}
          >
            GitHub ↗
          </Button>
        </div>
      </div>

      <Button
        variant="outline"
        size="xs"
        className="w-full"
        onClick={() => {
          const info = `Peregrine v${version}\n${t("settings.about.publisher")}: Eeymoo\n${t("settings.about.license")}: MIT\n${t("settings.about.repository")}: https://github.com/Eeymoo/peregrine`;
          navigator.clipboard?.writeText(info).catch(() => {});
        }}
      >
        {t("settings.copyVersionInfo")}
      </Button>
    </div>
  );
}
