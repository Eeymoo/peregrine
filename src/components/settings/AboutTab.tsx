import { useEffect, useState } from "react";
import { useI18n } from "@/lib/i18n";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { updatePreferences } from "@/lib/api";
import appIcon from "../../../assets/icon.png";

interface AboutTabProps {
  version: string;
  /** 当前是否已解锁开发者模式（来自配置文件，非 import.meta.env.DEV）。 */
  developerMode: boolean;
  /** 解锁状态变更回调：通知父组件刷新 config。 */
  onDeveloperModeChange: (unlocked: boolean) => void;
}

/**
 * 关于 Tab。
 *
 * 版本号可点击：连续点击 5 次（间隔 < 1.5s 超时清零）解锁开发者模式，
 * 持久化到 `AppSettings.developer_mode`，重启后保持。第 3 击起显示剩余次数提示。
 */
export function AboutTab({ version, developerMode, onDeveloperModeChange }: AboutTabProps) {
  const { t } = useI18n();
  const [clickCount, setClickCount] = useState(0);
  const [justUnlocked, setJustUnlocked] = useState(false);

  // 连续点击计数：间隔超过 1.5s 自动清零；满 5 次解锁并持久化。
  useEffect(() => {
    if (clickCount === 0) return;
    const timer = setTimeout(() => setClickCount(0), 1500);
    if (clickCount >= 5) {
      setClickCount(0);
      setJustUnlocked(true);
      setTimeout(() => setJustUnlocked(false), 3000);
      updatePreferences({ developer_mode: true }).catch(console.error);
      onDeveloperModeChange(true);
    }
    return () => clearTimeout(timer);
  }, [clickCount, onDeveloperModeChange]);

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
          <button
            type="button"
            className="cursor-pointer select-none hover:text-foreground"
            onClick={() => setClickCount((n) => n + 1)}
            title={
              developerMode || import.meta.env.DEV
                ? t("settings.devUnlocked")
                : t("settings.devUnlockHint")
            }
          >
            {version || "..."}
            {clickCount >= 3 && clickCount < 5 && (
              <span className="ml-1 text-[10px] opacity-60">
                ({5 - clickCount} {t("settings.devRemaining")})
              </span>
            )}
            {justUnlocked && (
              <span className="ml-1 text-[10px] text-green-500">
                {t("settings.devUnlocked")}
              </span>
            )}
            {(developerMode || import.meta.env.DEV) && !justUnlocked && (
              <span className="ml-1 text-[10px] text-yellow-500">DEV</span>
            )}
          </button>
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
