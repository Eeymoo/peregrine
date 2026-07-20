import { useEffect, useState, useCallback, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { Layer, MaterialInfo, AppConfig } from "@/types/config";
import { listLayers, listMaterials, saveConfig } from "@/lib/api";
import { listen } from "@tauri-apps/api/event";
import { Preview } from "@/components/Preview";
import { LayerPanel, MaterialParamControls } from "@/components/LayerPanel";
import { LayerStyleEditor, LayerTransformEditor } from "@/components/LayerEditors";
import { ProfileManager } from "@/components/ProfileManager";
import { useI18n } from "@/lib/i18n";

interface LayersEditorProps {
  /** 完整的应用配置对象，包含所有配置文件数据 */
  config: AppConfig;
  /** 覆盖层是否处于激活状态 */
  overlayActive: boolean;
  /** 窗口列表，用于目标窗口选择 */
  windows: string[];
  /** 可用配置文件列表（可选），用于profile管理 */
  profiles?: string[];
  /** 启动覆盖层的回调函数 */
  onStartOverlay: () => void;
  /** 停止覆盖层的回调函数 */
  onStopOverlay: () => void;
  /** 刷新窗口列表的回调函数 */
  onRefreshWindows: () => void;
  /** 更新设置的回调函数，接收部分设置对象 */
  onUpdateSettings: (patch: Partial<AppConfig["settings"]>) => void;
  /** 配置文件变更的回调函数，接收新的完整配置 */
  onConfigChange: (cfg: AppConfig) => void;
  /** 切换到单图层模式的回调函数 */
  onSwitchSingleLayer: () => void;
  /** 当前激活配置文件变更的回调函数（可选） */
  onActiveProfileChange?: (name: string) => void;
  /** 配置文件列表变更的回调函数（可选） */
  onProfilesChange?: (profiles: string[]) => void;
}

/**
 * 图层编辑器：完整的四层架构编辑 UI。
 *
 * 布局：左侧 Preview | 中间图层面板 | 右侧参数 / 样式 / 变换 + 通用控制面板
 *
 * 通过 Tauri commands 直接操作 backend，所有变化即时持久化。
 * 右侧底部保留通用控制：窗口模式、目标窗口、快捷颜色、开始/停止覆盖。
 */
export function LayersEditor({
  config,
  overlayActive,
  windows,
  profiles,
  onStartOverlay,
  onStopOverlay,
  onRefreshWindows,
  onUpdateSettings,
  onConfigChange,
  onSwitchSingleLayer,
  onActiveProfileChange,
  onProfilesChange,
}: LayersEditorProps) {
  const { t } = useI18n();
  const [layers, setLayers] = useState<Layer[]>([]);
  const [materials, setMaterials] = useState<MaterialInfo[]>([]);
  const [selectedId, setSelectedIdRaw] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  // 用 ref 保存最新 selectedId，避免 refresh 闭包捕获过期值。
  // 这样后端 emit layers-changed 触发 refresh 时，不会误判 selectedId 失效而回退到 l[0]。
  const selectedIdRef = useRef<string | null>(null);

  /** 记录一条操作日志到控制台，便于排查"切换到图层0"等问题。 */
  const logAction = useCallback((action: string, detail?: unknown) => {
    const ts = new Date().toISOString().slice(11, 23);
    console.log(`[action ${ts}] ${action}`, detail ?? "");
  }, []);

  /** 包装的 setSelectedId，自动同步 ref 并记录日志。 */
  const setSelectedId = useCallback((id: string | null, reason?: string) => {
    const prev = selectedIdRef.current;
    if (prev !== id) {
      logAction("select-layer", { from: prev, to: id, reason });
    }
    selectedIdRef.current = id;
    setSelectedIdRaw(id);
  }, [logAction]);

  useEffect(() => {
    selectedIdRef.current = selectedId;
  }, [selectedId]);

  // 加载图层与物料列表。不依赖 selectedId，避免闭包陷阱。
  const refresh = useCallback(async () => {
    try {
      const [l, m] = await Promise.all([listLayers(), listMaterials()]);
      setLayers(l);
      setMaterials(m);
      // 用 ref 读取最新 selectedId，避免回退到 l[0]。
      const current = selectedIdRef.current;
      if (!current && l.length > 0) {
        setSelectedId(l[0].id, "refresh:no-prev-selection");
      } else if (current && !l.find((x) => x.id === current)) {
        // 当前选中的图层确实不存在了（被删除），才回退到第一个。
        setSelectedId(l.length > 0 ? l[0].id : null, `refresh:prev-${current}-not-found`);
      }
    } catch (err) {
      console.error("Failed to load layers:", err);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh, refreshKey]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("peregrine:layers-changed", () => {
      logAction("layers-changed event");
      refresh();
      setRefreshKey((n) => n + 1);
    }).then((un) => {
      unlisten = un;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [refresh, logAction]);

  const triggerPreviewRefresh = () => setRefreshKey((n) => n + 1);

  const selectedLayer = layers.find((l) => l.id === selectedId);
  const selectedMaterial = selectedLayer
    ? materials.find((m) =>
        m.id ===
        (selectedLayer.material.kind === "builtin"
          ? selectedLayer.material.id
          : selectedLayer.material.name),
      )
    : null;

  const profile = config.profiles[config.active_profile];
  const isFullscreen = config.settings.fullscreen_overlay ?? true;

  const updateTargetWindow = (targetWindow: string) => {
    if (!profile) return;
    const newConfig: AppConfig = {
      ...config,
      profiles: {
        ...config.profiles,
        [config.active_profile]: {
          ...profile,
          target_window: targetWindow,
        },
      },
    };
    onConfigChange(newConfig);
    saveConfig(newConfig).catch(console.error);
  };

  return (
    <div className="h-full flex flex-col bg-background text-foreground">
      {/* 顶部栏：标题 + Profile 管理 */}
      <div className="flex items-center justify-between px-4 py-2 border-b bg-card shrink-0 gap-2">
        <div className="text-sm font-semibold">
          {t("app.title")} — {t("layers.editorTitle")}
        </div>
        <div className="flex items-center gap-2">
          <ProfileManager
            activeProfile={config.active_profile}
            profiles={profiles}
            onActiveProfileChange={onActiveProfileChange}
            onProfilesChange={onProfilesChange}
          />
        </div>
      </div>

      {/* 主体：三栏布局 */}
      <div className="flex-1 flex overflow-hidden min-h-0">
        {/* 左侧：预览 */}
        <div className="flex-1 p-4 min-w-0 min-h-0 relative">
          <Preview previewKey={`${selectedId}-${refreshKey}`} />
          {/* 切换到单图层按钮：与单图层模式的切换按钮位置一致 */}
          <button
            onClick={onSwitchSingleLayer}
            className="absolute top-6 right-6 text-xs px-3 py-1.5 bg-primary text-primary-foreground rounded shadow hover:bg-primary/90 z-10"
          >
            {t("layers.backToLegacy")}
          </button>
        </div>

        {/* 中间：图层列表 */}
        <div className="w-72 h-full flex flex-col border-l">
          <LayerPanel
            layers={layers}
            selectedLayerId={selectedId}
            onSelectLayer={setSelectedId}
            onChanged={() => {
              refresh();
              triggerPreviewRefresh();
            }}
          />
        </div>

        {/* 右侧：参数 + 样式 + 变换 + 通用控制 */}
        <div className="w-80 border-l bg-card p-4 flex flex-col gap-4 overflow-hidden h-full">
          {/* 上方：图层编辑参数（可滚动） */}
          <div className="flex-1 min-h-0 overflow-y-auto pr-1">
            {!selectedLayer ? (
              <div className="text-center text-muted-foreground text-sm mt-8">
                {t("layers.selectPrompt")}
              </div>
            ) : !selectedMaterial ? (
              <div className="text-center text-destructive text-sm mt-8">
                {t("layers.materialNotFound")}
              </div>
            ) : (
              <div className="space-y-6">
                <div className="space-y-1">
                  <h3 className="font-semibold text-sm">{selectedLayer.name}</h3>
                  <div className="text-xs text-muted-foreground">
                    {selectedMaterial.display_name} ({selectedMaterial.id})
                  </div>
                  {selectedMaterial.is_dynamic && (
                    <div className="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
                      ⚡ {t("layers.dynamicHint")}
                    </div>
                  )}
                </div>

                <div className="space-y-1">
                  <label className="text-xs font-medium">{t("layers.name")}</label>
                  <input
                    type="text"
                    value={selectedLayer.name}
                    disabled={selectedLayer.locked}
                    onChange={(e) => {
                      updateLayerName(selectedLayer.id, e.target.value, () => {
                        refresh();
                        triggerPreviewRefresh();
                      });
                    }}
                    onBlur={(e) => {
                      updateLayerName(selectedLayer.id, e.target.value, () => {
                        refresh();
                        triggerPreviewRefresh();
                      });
                    }}
                    className="w-full px-2 py-1 text-sm border rounded bg-background"
                  />
                </div>

                <div className="space-y-2">
                  <h4 className="text-xs font-semibold uppercase text-muted-foreground">
                    {t("layers.params")}
                  </h4>
                  <MaterialParamControls
                    schema={selectedMaterial.schema}
                    params={selectedLayer.params as Record<string, unknown>}
                    layerId={selectedLayer.id}
                    onChanged={() => triggerPreviewRefresh()}
                    locked={selectedLayer.locked}
                  />
                </div>

                <div className="space-y-2 pt-4 border-t">
                  <h4 className="text-xs font-semibold uppercase text-muted-foreground">
                    {t("layers.styleSection")}
                  </h4>
              <LayerStyleEditor
                layer={selectedLayer}
                quickColors={config.settings.quick_colors ?? []}
                onChanged={() => {
                  refresh();
                  triggerPreviewRefresh();
                }}
              />
                </div>

                <div className="space-y-2 pt-4 border-t">
                  <h4 className="text-xs font-semibold uppercase text-muted-foreground">
                    {t("layers.transformSection")}
                  </h4>
                  <LayerTransformEditor
                    layer={selectedLayer}
                    onChanged={() => {
                      refresh();
                      triggerPreviewRefresh();
                    }}
                  />
                </div>
              </div>
            )}
          </div>

          <Separator className="shrink-0" />

            {/* 下方：通用控制面板 */}
          <div className="space-y-3 shrink-0">
            {/* 窗口模式勾选 */}
            <div className="flex items-center gap-2">
              <Checkbox
                id="window-mode-layers"
                checked={!isFullscreen}
                disabled={overlayActive}
                title={overlayActive ? t("overlay.windowModeBlockedHint") : undefined}
                onCheckedChange={(v) => onUpdateSettings({ fullscreen_overlay: !v })}
              />
              <Label
                htmlFor="window-mode-layers"
                className={`text-sm ${overlayActive ? "text-muted-foreground cursor-not-allowed" : "cursor-pointer"}`}
              >
                {t("overlaySettings.windowMode")}
              </Label>
            </div>

            {/* 目标窗口（仅窗口模式时显示） */}
            {!isFullscreen && (
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <Label className="text-sm">{t("config.targetWindow")}</Label>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={onRefreshWindows}
                    className="h-8 text-sm px-2"
                  >
                    {t("config.refresh")}
                  </Button>
                </div>
                <Select
                  value={profile?.target_window || "__none__"}
                  onValueChange={(v) => updateTargetWindow(v === "__none__" ? "" : v)}
                >
                  <SelectTrigger className="h-8 text-sm">
                    <SelectValue placeholder={t("config.none")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__none__" className="text-sm">
                      {t("config.none")}
                    </SelectItem>
                    {windows.map((w) => (
                      <SelectItem key={w} value={w} className="text-sm">
                        {w.length > 30 ? w.slice(0, 30) + "…" : w}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}

            {/* 开始/停止覆盖 */}
            <div>
              {overlayActive ? (
                <Button variant="destructive" className="w-full h-8 text-sm" onClick={onStopOverlay}>
                  ■ {t("config.stopOverlay")}
                </Button>
              ) : (
                <Button
                  className="w-full h-8 text-sm"
                  onClick={onStartOverlay}
                  disabled={!isFullscreen && !profile?.target_window}
                >
                  ▶ {t("config.startOverlay")}
                </Button>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

async function updateLayerName(layerId: string, name: string, onChanged: () => void) {
  const { updateLayer } = await import("@/lib/api");
  await updateLayer(layerId, { name });
  onChanged();
}
