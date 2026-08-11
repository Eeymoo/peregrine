import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useI18n } from "@/lib/i18n";
import { useConfig } from "@/hooks/useAppState";
import { useAppVersion } from "@/hooks/useAppState";
import { useInitMirror } from "@/hooks/useSettingsSync";
import { useSettingsSync } from "@/hooks/useSettingsSync";
import { useUpdate } from "@/hooks/useUpdate";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent } from "@/components/ui/card";
import { GeneralTab } from "@/components/settings/GeneralTab";
import { OverlayTab } from "@/components/settings/OverlayTab";
import { HotkeysTab } from "@/components/settings/HotkeysTab";
import { UpdateTab, type UpdateState } from "@/components/settings/UpdateTab";
import { AboutTab } from "@/components/settings/AboutTab";
import { DevTab } from "@/components/settings/DevTab";
import { updatePreferences } from "@/lib/api";
import type { AppConfig } from "@/types/config";

export default function SettingsApp() {
  const { t, locale, setLocale, resolvedLocale } = useI18n();
  const { config, setConfig } = useConfig();
  const version = useAppVersion();
  const [autoSwitch, setAutoSwitchState] = useState<string>("ask");
  const [updateState, setUpdateState] = useState<UpdateState>({ status: "idle" });
  // 开发者模式：解锁后或开发构建（import.meta.env.DEV）下显示「开发」Tab。
  const devUnlocked = !!config?.settings?.developer_mode || import.meta.env.DEV;

  useEffect(() => {
    getCurrentWebviewWindow().setTitle(`${t("app.title")} ${t("settings.title")}`).catch(() => {});
  }, [t]);

  useEffect(() => {
    if (config) {
      setAutoSwitchState(config.settings?.auto_switch_on_overlay ?? "ask");
    }
  }, [config?.settings?.auto_switch_on_overlay]);

  useInitMirror();
  useSettingsSync(setConfig);
  useUpdate(config);

  const handleAutoSwitchChange = (value: string) => {
    setAutoSwitchState(value);
    if (!config) return;
    const newConfig: AppConfig = {
      ...config,
      settings: { ...config.settings, auto_switch_on_overlay: value },
    };
    setConfig(newConfig);
    updatePreferences({ auto_switch_on_overlay: value }).catch(() => {});
  };

  // AboutTab 解锁成功后立即更新本地 config state（updatePreferences 已写盘 +
  // 广播，useSettingsSync 会同步，这里立即刷新避免等待广播延迟）。
  const handleDeveloperModeChange = (unlocked: boolean) => {
    if (!config) return;
    setConfig({
      ...config,
      settings: { ...config.settings, developer_mode: unlocked },
    });
  };

  if (!config) {
    return (
      <div className="h-screen flex items-center justify-center text-muted-foreground">
        {t("config.loading")}
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-background text-foreground">
      <Tabs defaultValue="general" className="flex flex-col h-full">
        <div className="px-6 pt-5">
          <TabsList className={`grid w-full ${devUnlocked ? "grid-cols-6" : "grid-cols-5"}`}>
            <TabsTrigger value="general">{t("settings.sectionGeneral")}</TabsTrigger>
            <TabsTrigger value="overlay">{t("settings.sectionOverlay")}</TabsTrigger>
            <TabsTrigger value="hotkeys">{t("settings.sectionHotkeys")}</TabsTrigger>
            <TabsTrigger value="update">{t("settings.sectionUpdate")}</TabsTrigger>
            <TabsTrigger value="about">{t("settings.sectionAbout")}</TabsTrigger>
            {devUnlocked && <TabsTrigger value="dev">{t("settings.sectionDev")}</TabsTrigger>}
          </TabsList>
        </div>

        <TabsContent value="general" className="flex-1 overflow-y-auto m-0 p-6">
          <Card>
            <CardContent className="space-y-6 pt-6">
              <GeneralTab
                config={config}
                locale={locale}
                setConfig={setConfig}
                setLocale={setLocale}
              />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="overlay" className="flex-1 overflow-y-auto m-0 p-6">
          <Card>
            <CardContent className="space-y-6 pt-6">
              <OverlayTab
                config={config}
                autoSwitch={autoSwitch}
                onAutoSwitchChange={handleAutoSwitchChange}
                setConfig={setConfig}
              />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="hotkeys" className="flex-1 overflow-y-auto m-0 p-6">
          <Card>
            <CardContent className="space-y-4 pt-6">
              <HotkeysTab
                bindings={config.settings.hotkey_bindings}
                onChange={(newBindings) => {
                  const newConfig: AppConfig = {
                    ...config,
                    settings: { ...config.settings, hotkey_bindings: newBindings },
                  };
                  setConfig(newConfig);
                  updatePreferences({ hotkey_bindings: newBindings }).catch(() => {});
                }}
              />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="update" className="flex-1 overflow-y-auto m-0 p-6">
          <Card>
            <CardContent className="space-y-6 pt-6">
              <UpdateTab
                config={config}
                resolvedLocale={resolvedLocale}
                updateState={updateState}
                setUpdateState={setUpdateState}
                setConfig={setConfig}
              />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="about" className="flex-1 overflow-y-auto m-0 p-6">
          <Card>
            <CardContent className="space-y-6 pt-6">
              <AboutTab
                version={version}
                developerMode={!!config.settings.developer_mode}
                onDeveloperModeChange={handleDeveloperModeChange}
              />
            </CardContent>
          </Card>
        </TabsContent>

        {devUnlocked && (
          <TabsContent value="dev" className="flex-1 overflow-y-auto m-0 p-6">
            <Card>
              <CardContent className="space-y-6 pt-6">
                <DevTab config={config} setConfig={setConfig} />
              </CardContent>
            </Card>
          </TabsContent>
        )}
      </Tabs>
    </div>
  );
}
