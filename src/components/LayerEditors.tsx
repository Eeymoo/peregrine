import { useEffect, useState } from "react";
import type { Layer, LayerStyle, Transform2D } from "@/types/config";
import { updateLayer } from "@/lib/api";
import { Slider } from "@/components/ui/slider";
import { useI18n } from "@/lib/i18n";

/**
 * 图层样式编辑器：颜色 + 不透明度 + 混合模式。
 */
export function LayerStyleEditor({
  layer,
  onChanged,
}: {
  layer: Layer;
  onChanged: () => void;
}) {
  const { t } = useI18n();
  const [style, setStyle] = useState<LayerStyle>(layer.style);

  useEffect(() => {
    setStyle(layer.style);
  }, [layer]);

  const update = async (patch: Partial<LayerStyle>) => {
    const newStyle = { ...style, ...patch };
    setStyle(newStyle);
    await updateLayer(layer.id, { style: newStyle });
    onChanged();
  };

  return (
    <div className="space-y-3">
      <div className="space-y-1">
        <label className="text-xs font-medium">{t("layers.color")}</label>
        <div className="flex gap-2 items-center">
          <input
            type="color"
            value={rgbaToHex(style.color)}
            onChange={(e) => update({ color: hexToRgba(e.target.value, style.color[3]) })}
            className="w-10 h-8 border rounded"
          />
          <span className="text-xs text-muted-foreground font-mono">
            {rgbaToHex(style.color).toUpperCase()}
          </span>
        </div>
      </div>

      <div className="space-y-1">
        <label className="text-xs font-medium">{t("layers.alpha")}</label>
        <input
          type="number"
          min={0}
          max={1}
          step={0.01}
          value={style.color[3]}
          onChange={(e) => update({ color: [...style.color.slice(0, 3), parseFloat(e.target.value)] as [number, number, number, number] })}
          className="w-full px-2 py-1 text-sm border rounded bg-background"
        />
      </div>

      <div className="space-y-1">
        <label className="text-xs font-medium">{t("layers.opacity")}</label>
        <Slider
          min={0}
          max={1}
          step={0.01}
          value={[style.opacity]}
          onValueChange={(v) => update({ opacity: v[0] })}
        />
        <div className="text-xs text-muted-foreground text-right">
          {(style.opacity * 100).toFixed(0)}%
        </div>
      </div>
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
  layer: Layer;
  onChanged: () => void;
}) {
  const { t } = useI18n();
  const [transform, setTransform] = useState<Transform2D>(layer.transform);

  useEffect(() => {
    setTransform(layer.transform);
  }, [layer]);

  const update = async (patch: Partial<Transform2D>) => {
    const newTransform = { ...transform, ...patch };
    setTransform(newTransform);
    await updateLayer(layer.id, { transform: newTransform });
    onChanged();
  };

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

      <div className="space-y-1">
        <label className="text-xs font-medium">{t("layers.scale")}</label>
        <Slider
          min={0.1}
          max={5}
          step={0.01}
          value={[transform.scale]}
          onValueChange={(v) => update({ scale: v[0] })}
        />
        <div className="text-xs text-muted-foreground text-right">
          {transform.scale.toFixed(2)}x
        </div>
      </div>

      <div className="space-y-1">
        <label className="text-xs font-medium">{t("layers.rotation")}</label>
        <Slider
          min={-180}
          max={180}
          step={1}
          value={[transform.rotation_deg]}
          onValueChange={(v) => update({ rotation_deg: v[0] })}
        />
        <div className="text-xs text-muted-foreground text-right">
          {transform.rotation_deg.toFixed(0)}°
        </div>
      </div>
    </div>
  );
}

function rgbaToHex(color: [number, number, number, number]): string {
  const r = Math.round(color[0] * 255);
  const g = Math.round(color[1] * 255);
  const b = Math.round(color[2] * 255);
  return `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}`;
}

function hexToRgba(hex: string, alpha: number): [number, number, number, number] {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  return [r, g, b, alpha];
}
