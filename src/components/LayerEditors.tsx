import { useEffect, useState, useCallback } from "react";
import type { Layer, LayerStyle, Transform2D } from "@/types/config";
import { updateLayer } from "@/lib/api";
import { logAction } from "@/lib/actionLog";
import { useI18n } from "@/lib/i18n";
import { SliderField } from "@/components/fields/SliderField";
import { ColorField, type Rgba } from "@/components/fields/ColorField";

/**
 * 图层样式编辑器：颜色 + 不透明度 + 混合模式。
 */
export function LayerStyleEditor({
  layer,
  quickColors,
  onChanged,
}: {
  /** 当前编辑的图层对象，包含样式信息 */
  layer: Layer;
  /** 快捷颜色列表，用于快速设置图层颜色 */
  quickColors?: [number, number, number, number][];
  /** 样式更新后的回调函数，用于触发重新渲染 */
  onChanged: () => void;
}) {
  const { t } = useI18n();
  const [style, setStyle] = useState<LayerStyle>(layer.style);

  useEffect(() => {
    setStyle(layer.style);
  }, [layer]);

  const update = useCallback(async (patch: Partial<LayerStyle>) => {
    const newStyle = { ...style, ...patch };
    logAction("update-layer-style", { id: layer.id, patch });
    setStyle(newStyle);
    await updateLayer(layer.id, { style: newStyle });
    onChanged();
  }, [layer.id, onChanged, style]);

  return (
    <div className="space-y-3">
      <ColorField
        label={t("layers.color")}
        value={style.color}
        disabled={layer.locked}
        quickColors={quickColors as Rgba[] | undefined}
        onChange={(color) => update({ color })}
      />

      <SliderField
        label={t("layers.opacity")}
        value={style.opacity}
        min={0}
        max={1}
        step={0.01}
        unit="%"
        disabled={layer.locked}
        onChange={(opacity) => update({ opacity })}
      />
    </div>
  );
}

/**
 * 图层变换编辑器：位移 / 缩放 / 旋转。
 */
export function LayerTransformEditor({
  layer,
  onChanged,
}: {
  /** 当前编辑的图层对象，包含变换信息 */
  layer: Layer;
  /** 变换更新后的回调函数，用于触发重新渲染 */
  onChanged: () => void;
}) {
  const { t } = useI18n();
  const [transform, setTransform] = useState<Transform2D>(layer.transform);

  useEffect(() => {
    setTransform(layer.transform);
  }, [layer]);

  const update = useCallback(async (patch: Partial<Transform2D>) => {
    const newTransform = { ...transform, ...patch };
    logAction("update-layer-transform", { id: layer.id, patch });
    setTransform(newTransform);
    await updateLayer(layer.id, { transform: newTransform });
    onChanged();
  }, [layer.id, onChanged, transform]);

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1">
          <label className="text-xs font-medium">{t("layers.offsetX")}</label>
          <input
            type="number"
            value={transform.offset_x}
            onChange={(e) => update({ offset_x: parseFloat(e.target.value) })}
            className="w-full px-2 py-1 text-sm border rounded bg-background"
          />
        </div>
        <div className="space-y-1">
          <label className="text-xs font-medium">{t("layers.offsetY")}</label>
          <input
            type="number"
            value={transform.offset_y}
            onChange={(e) => update({ offset_y: parseFloat(e.target.value) })}
            className="w-full px-2 py-1 text-sm border rounded bg-background"
          />
        </div>
      </div>

      <SliderField
        label={t("layers.scale")}
        value={transform.scale}
        min={0.1}
        max={5}
        step={0.01}
        unit="x"
        disabled={layer.locked}
        onChange={(scale) => update({ scale })}
      />

      <SliderField
        label={t("layers.rotation")}
        value={transform.rotation_deg}
        min={-180}
        max={180}
        step={1}
        unit="°"
        disabled={layer.locked}
        onChange={(rotation_deg) => update({ rotation_deg })}
      />
    </div>
  );
}
