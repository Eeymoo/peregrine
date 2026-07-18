import { useEffect, useState, useCallback } from "react";
import type { Layer, MaterialInfo } from "@/types/config";
import { listLayers, listMaterials } from "@/lib/api";
import { Preview } from "@/components/Preview";
import {
  LayerPanel,
  MaterialParamControls,
} from "@/components/LayerPanel";
import { LayerStyleEditor, LayerTransformEditor } from "@/components/LayerEditors";
import { useI18n } from "@/lib/i18n";

/**
 * 图层编辑器：完整的四层架构编辑 UI。
 *
 * 布局：左侧 Preview | 中间图层面板 | 右侧参数 / 样式 / 变换
 *
 * 通过 Tauri commands 直接操作 backend，所有变化即时持久化。
 */
export function LayersEditor() {
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
      // 如果没有选中图层，默认选第一个。
      if (!selectedId && l.length > 0) {
        setSelectedId(l[0].id);
      }
      // 如果选中的图层已被删除，清空选择。
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

  // 触发 Preview 重新计算（图层数据变化时调用）。
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

  return (
    <div className="h-screen flex bg-background text-foreground overflow-hidden">
      {/* 左侧：预览 */}
      <div className="flex-1 p-4 min-w-0">
        <Preview previewKey={`${selectedId}-${refreshKey}`} />
      </div>

      {/* 中间：图层列表 */}
      <div className="w-72">
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

      {/* 右侧：参数 + 样式 + 变换 */}
      <div className="w-80 border-l bg-card p-4 overflow-y-auto">
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
            {/* 物料信息 */}
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

            {/* 图层名称 */}
            <div className="space-y-1">
              <label className="text-xs font-medium">{t("layers.name")}</label>
              <input
                type="text"
                value={selectedLayer.name}
                disabled={selectedLayer.locked}
                onChange={(e) => {
                  // 即时更新本地，触发后端持久化。
                  updateLayerName(selectedLayer.id, e.target.value, () => {
                    refresh();
                    triggerPreviewRefresh();
                  });
                }}
                className="w-full px-2 py-1 text-sm border rounded bg-background"
              />
            </div>

            {/* 物料参数 */}
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

            {/* 图层样式 */}
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

            {/* 图层变换 */}
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
    </div>
  );
}

/** 异步更新图层名称（带简单节流）。 */
async function updateLayerName(layerId: string, name: string, onChanged: () => void) {
  const { updateLayer } = await import("@/lib/api");
  await updateLayer(layerId, { name });
  onChanged();
}
