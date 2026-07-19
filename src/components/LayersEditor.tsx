import { useEffect, useState, useCallback } from "react";
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
import { useI18n } from "@/lib/i18n";

interface LayersEditorProps {
  config: AppConfig;
  overlayActive: boolean;
  windows: string[];
  onStartOverlay: () => void;
  onStopOverlay: () => void;
  onRefreshWindows: () => void;
  onUpdateSettings: (patch: Partial<AppConfig["settings"]>) => void;
  onConfigChange: (cfg: AppConfig) => void;
  onSwitchSingleLayer: () => void;
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
  onStartOverlay,
  onStopOverlay,
  onRefreshWindows,
  onUpdateSettings,
  onConfigChange,
  onSwitchSingleLayer,
}: LayersEditorProps) {
  const { t } = useI18n();
  const [layers, setLayers] = useState<Layer[]>([]);
  const [materials, setMaterials] = useState<MaterialInfo[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  // 加载图层与物料列表。
  const refresh = useCallback(async () => {
    try {
      const [l, m] = await Promise.all([listLayers(), listMaterials()]);
      setLayers(l);
      setMaterials(m);
      if (!selectedId && l.length > 0) {
        setSelectedId(l[0].id);
      }
      if (selectedId && !l.find((x) => x.id === selectedId)) {
        setSelectedId(l.length > 0 ? l[0].id : null);
      }
    } catch (err) {
      console.error("Failed to load layers:", err);
    }
  }, [selectedId]);

  useEffect(() => {
    refresh();
  }, [refresh, refreshKey]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("peregrine:layers-changed", () => {
      refresh();
      setRefreshKey((n) => n + 1);
    }).then((un) => {
      unlisten = un;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [refresh]);

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
  const quickColors = config.settings.quick_colors ?? [];

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
      {/* 顶部栏：标题 + 开始/停止覆盖 */}
      <div className="flex items-center justify-between px-4 py-2 border-b bg-card shrink-0">
        <div className="text-sm font-semibold">
          {t("app.title")} — {t("layers.editorTitle")}
        </div>
        <div>
          {overlayActive ? (
            <Button variant="destructive" size="sm" className="h-7 text-xs" onClick={onStopOverlay}>
              ■ {t("config.stopOverlay")}
            </Button>
          ) : (
            <Button
              size="sm"
              className="h-7 text-xs"
              onClick={onStartOverlay}
              disabled={!isFullscreen && !profile?.target_window}
            >
              ▶ {t("config.startOverlay")}
            </Button>
          )}
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
            ← {t("layers.backToLegacy")}
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
            {/* 快捷颜色 */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label className="text-sm font-medium">{t("quickColors.title")}</Label>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    const defaultColors: [number, number, number, number][] = [
                      [1, 1, 1, 1],
                      [0, 1, 0, 1],
                      [0.2, 0.5, 1, 1],
                      [1, 0, 0, 1],
                      [1, 0.5, 0, 1],
                    ];
                    onUpdateSettings({ quick_colors: defaultColors });
                  }}
                >
                  {t("quickColors.reset")}
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">{t("quickColors.hint")}</p>
              <div className="flex gap-3 pt-1">
                {quickColors.map((qc, i) => {
                  const css = `rgb(${Math.round(qc[0] * 255)}, ${Math.round(qc[1] * 255)}, ${Math.round(qc[2] * 255)})`;
                  const hex = `#${Math.round(qc[0] * 255).toString(16).padStart(2, "0")}${Math.round(qc[1] * 255).toString(16).padStart(2, "0")}${Math.round(qc[2] * 255).toString(16).padStart(2, "0")}`;
                  return (
                    <div key={i} className="flex flex-col items-center gap-1">
                      <input
                        type="color"
                        value={hex}
                        onChange={(e) => {
                          const h = e.target.value;
                          const r = parseInt(h.slice(1, 3), 16) / 255;
                          const g = parseInt(h.slice(3, 5), 16) / 255;
                          const b = parseInt(h.slice(5, 7), 16) / 255;
                          const newColors = [...quickColors];
                          newColors[i] = [r, g, b, 1];
                          onUpdateSettings({ quick_colors: newColors });
                        }}
                        className="w-8 h-8 rounded-full cursor-pointer border-2"
                        style={{ backgroundColor: css }}
                      />
                    </div>
                  );
                })}
              </div>
            </div>

            <Separator className="shrink-0" />

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
