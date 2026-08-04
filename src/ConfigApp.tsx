import { useEffect, useRef, useState } from "react";
import { useI18n } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Separator } from "@/components/ui/separator";
import { Preview } from "@/components/Preview";
import { StyleFields } from "@/components/StyleFields";
import { LayersEditor } from "@/components/LayersEditor";
import { ProfileManager } from "@/components/ProfileManager";
import { DeveloperPanel } from "@/components/DeveloperPanel";
import { AutoSwitchDialog } from "@/components/config/AutoSwitchDialog";
import { UpdateDialog } from "@/components/config/UpdateDialog";
import { UpdateProgress } from "@/components/config/UpdateProgress";
import { TargetWindowSelect } from "@/components/config/TargetWindowSelect";
import { useConfigAppState } from "@/hooks/useConfigAppState";
import { useConfigSave } from "@/hooks/useConfigSave";
import { useOverlayActions } from "@/hooks/useOverlayActions";
import { useUpdate } from "@/hooks/useUpdate";
import { updatePreferences, getCurrentWebviewWindow } from "@/lib/api";
import { MATERIAL_RUNTIME_ENABLED } from "@/lib/feature";
import {
  TELEMETRY_DSN_AVAILABLE,
  initTelemetry,
} from "@/lib/telemetry";
import type { AppConfig, CrosshairStyle } from "@/types/config";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const STYLES: CrosshairStyle[] = [
  "edge_rect",
  "cross",
  "large_cross",
  "corner_dots4",
  "corner_dots6",
  "corner_dots8",
  "ring",
  "custom_orb",
  "random_orb",
  "border_frame",
  // "custom_image", // 暂时隐藏，存在问题
  "edge_arrows",
  "grid",
];

export default function ConfigApp() {
  const { t } = useI18n();
  const {
    config,
    setConfig,
    windows,
    profiles,
    setProfiles,
    overlayActive,
    setOverlayActive,
    loading,
    version,
    layersMode,
    setLayersMode,
    refreshWindows,
    changeActiveProfile,
  } = useConfigAppState();
  const [showAutoSwitchDialog, setShowAutoSwitchDialog] = useState(false);
  const {
    profile,
    crosshair,
    hasLayers,
    isLegacyCompatible,
    updateCrosshair,
    updateProfileTargetWindow,
    colorCss,
  } = useConfigSave(config, setConfig);
  const { handleStartOverlay, handleStopOverlay, saveAutoSwitchPreference } =
    useOverlayActions(config, setOverlayActive, () =>
      setShowAutoSwitchDialog(true),
    );

  const { updateAvailable, updating, updateProgress, setUpdateAvailable, startUpdate } =
    useUpdate(config);

  // 点击版本号 3 次后启用"开发者"面板，写入 localStorage 持久化。
  const [devTabUnlocked, setDevTabUnlocked] = useState<boolean>(
    () => localStorage.getItem("peregrine:dev-tab") === "1",
  );
  const [versionClickCount, setVersionClickCount] = useState(0);
  const [justUnlocked, setJustUnlocked] = useState(false);

  // 点击版本号 5 次（连续，间隔 < 1.5s）解锁开发者面板；第 3 击起有剩余次数提示。
  // 解锁状态写入 localStorage，下次启动仍然有效；可在开发者面板里关闭。
  useEffect(() => {
    if (versionClickCount === 0) return;
    const timer = setTimeout(() => setVersionClickCount(0), 1500);
    if (versionClickCount >= 5) {
      setVersionClickCount(0);
      setDevTabUnlocked(true);
      localStorage.setItem("peregrine:dev-tab", "1");
      // 解锁成功提示：版本号旁短暂显示「已开启开发者模式」，3 秒后消失。
      setJustUnlocked(true);
      setTimeout(() => setJustUnlocked(false), 3000);
    }
    return () => clearTimeout(timer);
  }, [versionClickCount]);

  // 首次启动遥测授权弹窗（唯一授权提示）：
  // telemetry_enabled 字段缺失（null/undefined）时询问一次，默认允许；
  // 结果写入配置后不再出现任何形式的授权提示。
  // 无 DSN 构建（编译期禁用）直接跳过，不弹窗。
  const telemetryConsentAsked = useRef(false);
  useEffect(() => {
    if (!config || telemetryConsentAsked.current) return;
    if (!TELEMETRY_DSN_AVAILABLE) return;
    const consent = config.settings.telemetry_enabled;
    if (consent !== null && consent !== undefined) return;
    telemetryConsentAsked.current = true;
    (async () => {
      try {
        const { ask } = await import("@tauri-apps/plugin-dialog");
        const allow = await ask(t("telemetry.consentDesc"), {
          title: t("telemetry.consentTitle"),
          okLabel: t("telemetry.consentAllow"),
          cancelLabel: t("telemetry.consentDeny"),
          kind: "info",
        });
        await updatePreferences({ telemetry_enabled: allow });
        // 前端 SDK 即时生效；Rust 侧 SDK 下次启动时初始化。
        if (allow) initTelemetry(true);
      } catch (e) {
        console.error("[telemetry] consent dialog failed:", e);
      }
    })();
  }, [config, t]);

  /** 更新应用级偏好设置（仅更新指定字段，不覆盖整个配置）。 */
  const updateSettings = (patch: Partial<AppConfig["settings"]>) => {
    if (!config) return;
    setConfig({ ...config, settings: { ...config.settings, ...patch } });
    updatePreferences(patch).catch(console.error);
  };

  if (loading || !config) {
    return (
      <div className="h-screen flex items-center justify-center text-muted-foreground">
        {t("config.loading")}
      </div>
    );
  }

  // 异常情况（既没有 crosshair 又没有 layers）：显示错误并提供切换到图层编辑器。
  if (!crosshair && !hasLayers) {
    return (
      <div className="h-screen flex flex-col items-center justify-center text-muted-foreground gap-4">
        <span className="text-lg">{t("config.invalidFormat")}</span>
        {/* 异常兜底入口：软关闭不门控模式切换（D4 2026-08-03 二次修订） */}
        <button
          className="text-xs px-3 py-1 border rounded hover:bg-accent"
          onClick={() => setLayersMode(true)}
        >
          {t("config.switchToLayersFallback")}
        </button>
      </div>
    );
  }

  // 图层编辑器模式：显示全新 UI。
  // 模式切换不门控（D4 2026-08-03 二次修订）：软关闭只作用于渲染链路，
  // UI 可自由切换单/多图层；软关闭期间图层编辑不影响 overlay 渲染与预览。
  if (layersMode) {
    return (
      <div className="h-screen flex flex-col bg-background text-foreground">
        <LayersEditor
          config={config}
          overlayActive={overlayActive}
          windows={windows}
          profiles={profiles}
          onStartOverlay={handleStartOverlay}
          onStopOverlay={handleStopOverlay}
          onRefreshWindows={refreshWindows}
          onUpdateSettings={updateSettings}
          onConfigChange={setConfig}
          onSwitchSingleLayer={() => setLayersMode(false)}
          onActiveProfileChange={changeActiveProfile}
          onProfilesChange={setProfiles}
        />
      </div>
    );
  }

  // 到这里 crosshair 必为非 null（已从 layers[0] 反向生成）。
  const ch = crosshair!;

  // MATERIAL_RUNTIME_ENABLED 门控：物料运行时启用时使用真实兼容性判定——
  // 不兼容 profile 禁用单图层编辑（防止改坏多图层配置），引导至多图层模式；
  // 软关闭期间恒为 true（crosshair 为权威配置，允许按旧准星编辑驱动渲染）。
  const effectiveCompatible = MATERIAL_RUNTIME_ENABLED ? isLegacyCompatible : true;

  return (
    <div className="h-screen flex bg-background text-foreground overflow-hidden">
      {/* 左侧预览 */}
      <div className="flex-1 p-4 min-w-0 min-h-0 relative">
        <Preview previewKey={profile?.layers} />

        {/* 顶部工具栏：仅保留切换到多图层按钮 */}
        {/* 模式切换入口：软关闭不门控（D4 2026-08-03 二次修订） */}
        <div className="absolute top-4 left-4 right-4 flex items-center justify-end z-10">
          <button
            onClick={() => setLayersMode(true)}
            className="text-xs px-3 py-1.5 bg-primary text-primary-foreground rounded shadow hover:bg-primary/90"
            title={t("layers.switchToLayers")}
          >
            {t("layers.switchToLayers")}
          </button>
        </div>
      </div>

      {/* 右侧设置面板：顶部固定、中间滚动、底部固定 */}
      <div className="w-80 border-l bg-card p-4 flex flex-col gap-4 overflow-hidden h-screen">
        {/* 顶部固定区：Profile 管理 + 样式 + 公共配置 */}
        <div className="space-y-3 shrink-0">
          {/* Profile 管理 */}
          <ProfileManager
            activeProfile={config.active_profile}
            profiles={profiles}
            onActiveProfileChange={changeActiveProfile}
            onProfilesChange={setProfiles}
          />

          {/* 单图层不兼容提示：切换入口不门控（D4 2026-08-03 二次修订）；
              软关闭期间附加「功能暂停」说明，提示图层编辑暂不影响渲染 */}
          {!isLegacyCompatible && (
            <div className="p-2 rounded bg-yellow-500/10 border border-yellow-500/30 text-xs text-yellow-700 dark:text-yellow-400 space-y-1">
              <div>{t("profile.incompatible")}</div>
              {!MATERIAL_RUNTIME_ENABLED && <div>{t("profile.layersDisabled")}</div>}
              <button
                type="button"
                onClick={() => setLayersMode(true)}
                className="text-xs underline hover:text-yellow-800 dark:hover:text-yellow-300"
              >
                {t("config.switchToLayersFallback")}
              </button>
            </div>
          )}

          {/* 样式选择 */}
          <div className="space-y-2">
            <Label className="text-sm">{t("config.style")}</Label>
            <Select
              value={ch.style}
              onValueChange={(v) =>
                updateCrosshair({ style: v as CrosshairStyle }, { resetDefaults: true })
              }
              disabled={!effectiveCompatible}
            >
              <SelectTrigger className="h-8 text-sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {STYLES.map((s) => (
                  <SelectItem key={s} value={s} className="text-sm">
                    {t(`styles.${s}`)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* 公共配置 */}
          {/* 公共配置：禁用态与 StyleFields 一致（pointer-events-none + opacity-60 wrapper） */}
          <div className={effectiveCompatible ? "space-y-3" : "space-y-3 pointer-events-none opacity-60"}>
            <div className="space-y-2">
              <div className="flex justify-between">
                <Label className="text-sm">{t("config.opacity")}</Label>
                <span className="text-sm text-muted-foreground">
                  {ch.opacity.toFixed(2)}
                </span>
              </div>
              <Slider
                value={[ch.opacity]}
                min={0}
                max={1}
                step={0.01}
                onValueChange={([v]) => updateCrosshair({ opacity: v })}
              />
            </div>

            <div className="flex items-center gap-3">
              <Label className="shrink-0 text-sm">{t("config.color")}</Label>
              <input
                type="color"
                value={colorCss}
                onChange={(e) => {
                  const hex = e.target.value;
                  const r = parseInt(hex.slice(1, 3), 16) / 255;
                  const g = parseInt(hex.slice(3, 5), 16) / 255;
                  const b = parseInt(hex.slice(5, 7), 16) / 255;
                  updateCrosshair({ color: [r, g, b, 1] });
                }}
                className="h-8 w-14 rounded border bg-transparent cursor-pointer disabled:cursor-not-allowed"
              />
              {/* 快捷颜色色块 */}
              <div className="flex gap-1 flex-wrap">
                {(config.settings.quick_colors ?? []).map((qc, i) => {
                  const css = `rgb(${Math.round(qc[0] * 255)}, ${Math.round(
                    qc[1] * 255,
                  )}, ${Math.round(qc[2] * 255)})`;
                  const isActive =
                    ch.color[0] === qc[0] &&
                    ch.color[1] === qc[1] &&
                    ch.color[2] === qc[2];
                  return (
                    <button
                      key={i}
                      type="button"
                      title={css}
                      disabled={!effectiveCompatible}
                      onClick={() => updateCrosshair({ color: [...qc] })}
                      className="w-5 h-5 rounded-full border-2 transition-colors disabled:opacity-50"
                      style={{
                        backgroundColor: css,
                        borderColor: isActive
                          ? "hsl(var(--primary))"
                          : "hsl(var(--border))",
                      }}
                    />
                  );
                })}
              </div>
            </div>
          </div>
        </div>

        <Separator className="shrink-0" />

        {/* 中间样式配置：默认随窗口高度自适应，内容过多时才滚动 */}
        <div className="flex-1 min-h-0 overflow-y-auto pr-1">
          <StyleFields
            crosshair={ch}
            onChange={updateCrosshair}
            disabled={!effectiveCompatible}
          />
        </div>

        <Separator className="shrink-0" />

        {/* 底部固定区：覆盖模式 + 目标窗口 + 开始/停止覆盖 */}
        <div className="space-y-3 shrink-0">
          {/* 窗口模式勾选（默认全屏，勾选切换为窗口模式）。覆盖层激活时禁止切换，避免状态不一致。 */}
          <div className="flex items-center gap-2">
            <Checkbox
              id="window-mode"
              checked={!config.settings.fullscreen_overlay}
              disabled={overlayActive}
              title={overlayActive ? t("overlay.windowModeBlockedHint") : undefined}
              onCheckedChange={(v) => updateSettings({ fullscreen_overlay: !v })}
            />
            <Label
              htmlFor="window-mode"
              className={`text-sm ${overlayActive ? "text-muted-foreground cursor-not-allowed" : "cursor-pointer"}`}
            >
              {t("overlaySettings.windowMode")}
            </Label>
          </div>

          {/* 目标窗口（仅窗口模式时显示） */}
          {!config.settings.fullscreen_overlay && (
            <TargetWindowSelect
              value={profile?.target_window ?? ""}
              onChange={(v) => updateProfileTargetWindow(v === "__none__" ? "" : v)}
            />
          )}

          {/* 开始/停止覆盖 */}
          <div>
            {overlayActive ? (
              <Button
                variant="destructive"
                className="w-full h-8 text-sm"
                onClick={handleStopOverlay}
              >
                ■ {t("config.stopOverlay")}
              </Button>
            ) : (
              <Button
                className="w-full h-8 text-sm"
                onClick={handleStartOverlay}
                disabled={
                  !config.settings.fullscreen_overlay && !profile?.target_window
                }
              >
                ▶ {t("config.startOverlay")}
              </Button>
            )}
          </div>
        </div>

        {/* 开发者面板（仅 devTabUnlocked=true 时显示） */}
        {devTabUnlocked && (
          <div className="shrink-0 border-t pt-2 mt-2 max-h-48 overflow-y-auto">
            <DeveloperPanel
              config={config}
              version={version}
              onClose={() => {
                setDevTabUnlocked(false);
                localStorage.removeItem("peregrine:dev-tab");
              }}
            />
          </div>
        )}

        {/* 底部信息：连续点击 5 次解锁开发者面板 */}
        <div
          className="text-xs text-muted-foreground text-right cursor-pointer select-none shrink-0"
          onClick={() => {
            setVersionClickCount((n) => n + 1);
          }}
          title={devTabUnlocked ? t("developer.toggle") : t("developer.unlockHint")}
        >
          Peregrine v{version || "..."}
          {versionClickCount > 0 && versionClickCount < 5 && (
            <span className="ml-1 text-[10px] opacity-60">
              ({5 - versionClickCount} {t("developer.remaining")})
            </span>
          )}
          {justUnlocked && (
            <span className="ml-1 text-[10px] text-green-500">
              {t("developer.unlocked")}
            </span>
          )}
          {devTabUnlocked && !justUnlocked && (
            <span className="ml-1 text-[10px] text-yellow-500">
              {t("developer.tag")}
            </span>
          )}
        </div>
      </div>

      {/* 自动切换确认对话框 */}
      {showAutoSwitchDialog && (
        <AutoSwitchDialog
          onConfirm={(remember) => {
            setShowAutoSwitchDialog(false);
            if (remember) {
              saveAutoSwitchPreference("yes", profile?.target_window);
            } else if (profile?.target_window) {
              getCurrentWebviewWindow().destroy().catch(() => {});
            }
          }}
          onCancel={(remember) => {
            setShowAutoSwitchDialog(false);
            if (remember) {
              saveAutoSwitchPreference("no");
            }
          }}
          onCloseByEsc={handleStopOverlay}
        />
      )}

      {/* 发现新版本对话框 */}
      {updateAvailable && !updating && (
        <UpdateDialog
          version={updateAvailable.version}
          body={updateAvailable.body}
          onUpdate={() => {
            setUpdateAvailable(null);
            startUpdate();
          }}
          onLater={() => setUpdateAvailable(null)}
        />
      )}

      {/* 更新下载进度 */}
      {updating && <UpdateProgress progress={updateProgress} />}
    </div>
  );
}
