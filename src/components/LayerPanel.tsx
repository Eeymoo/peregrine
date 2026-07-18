import { useEffect, useState } from "react";
import type { Layer, MaterialInfo, MaterialSchemaEntry } from "@/types/config";
import { invoke } from "@tauri-apps/api/core";
import {
  addLayer,
  duplicateLayer,
  listMaterials,
  moveLayer,
  removeLayer,
  updateLayer,
} from "@/lib/api";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import { Trash2, Copy, ChevronUp, ChevronDown, Plus, Eye, EyeOff, Lock, Unlock } from "lucide-react";
import { useI18n } from "@/lib/i18n";

interface LayerPanelProps {
  layers: Layer[];
  selectedLayerId: string | null;
  onSelectLayer: (id: string) => void;
  onChanged: () => void;
}

/**
 * 图层管理面板：显示图层列表，支持增删/排序/可见性/复制。
 */
export function LayerPanel({
  layers,
  selectedLayerId,
  onSelectLayer,
  onChanged,
}: LayerPanelProps) {
  const { t } = useI18n();
  const [materials, setMaterials] = useState<MaterialInfo[]>([]);
  const [showAddDialog, setShowAddDialog] = useState(false);

  // 加载物料列表（仅一次）。
  useEffect(() => {
    listMaterials().then(setMaterials).catch(console.error);
  }, []);

  const handleAdd = async (materialId: string, name: string) => {
    await addLayer(materialId, name);
    setShowAddDialog(false);
    onChanged();
  };

  const handleDelete = async (id: string) => {
    await removeLayer(id);
    onChanged();
  };

  const handleDuplicate = async (id: string) => {
    await duplicateLayer(id);
    onChanged();
  };

  const handleMove = async (id: string, direction: "up" | "down") => {
    const idx = layers.findIndex((l) => l.id === id);
    if (idx < 0) return;
    const newIdx = direction === "up" ? Math.max(0, idx - 1) : Math.min(layers.length - 1, idx + 1);
    if (newIdx === idx) return;
    await moveLayer(id, newIdx);
    onChanged();
  };

  const handleToggleVisible = async (layer: Layer) => {
    await updateLayer(layer.id, { visible: !layer.visible });
    onChanged();
  };

  const handleToggleLock = async (layer: Layer) => {
    await updateLayer(layer.id, { locked: !layer.locked });
    onChanged();
  };

  // 渲染顺序：最顶层图层显示在最上面（与 Photoshop 习惯一致）。
  const reversedLayers = [...layers].reverse();

  return (
    <div className="flex flex-col h-full bg-card border-l">
      <div className="flex items-center justify-between p-3 border-b">
        <h3 className="font-semibold text-sm">{t("layers.title")}</h3>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setShowAddDialog(true)}
          title={t("layers.add")}
        >
          <Plus className="w-4 h-4" />
        </Button>
      </div>

      {/* 图层列表 */}
      <div className="flex-1 overflow-y-auto">
        {reversedLayers.length === 0 ? (
          <div className="p-4 text-center text-sm text-muted-foreground">
            {t("layers.empty")}
          </div>
        ) : (
          reversedLayers.map((layer) => {
            const materialId =
              layer.material.kind === "builtin" ? layer.material.id : layer.material.name;
            const material = materials.find((m) => m.id === materialId);
            return (
              <div
                key={layer.id}
                onClick={() => onSelectLayer(layer.id)}
                className={`flex items-center gap-2 px-3 py-2 cursor-pointer border-b text-sm hover:bg-accent/50 ${
                  selectedLayerId === layer.id ? "bg-accent" : ""
                } ${!layer.visible ? "opacity-50" : ""}`}
              >
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleToggleVisible(layer);
                  }}
                  className="text-muted-foreground hover:text-foreground"
                  title={layer.visible ? t("layers.hide") : t("layers.show")}
                >
                  {layer.visible ? <Eye className="w-3.5 h-3.5" /> : <EyeOff className="w-3.5 h-3.5" />}
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleToggleLock(layer);
                  }}
                  className="text-muted-foreground hover:text-foreground"
                  title={layer.locked ? t("layers.unlock") : t("layers.lock")}
                >
                  {layer.locked ? <Lock className="w-3.5 h-3.5" /> : <Unlock className="w-3.5 h-3.5" />}
                </button>
                <div className="flex-1 min-w-0">
                  <div className="truncate">{layer.name}</div>
                  <div className="text-xs text-muted-foreground truncate">
                    {material?.display_name ?? "—"}
                  </div>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleMove(layer.id, "up");
                  }}
                  className="text-muted-foreground hover:text-foreground"
                  title={t("layers.moveUp")}
                >
                  <ChevronUp className="w-3.5 h-3.5" />
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleMove(layer.id, "down");
                  }}
                  className="text-muted-foreground hover:text-foreground"
                  title={t("layers.moveDown")}
                >
                  <ChevronDown className="w-3.5 h-3.5" />
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDuplicate(layer.id);
                  }}
                  className="text-muted-foreground hover:text-foreground"
                  title={t("layers.duplicate")}
                >
                  <Copy className="w-3.5 h-3.5" />
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDelete(layer.id);
                  }}
                  className="text-muted-foreground hover:text-red-500"
                  title={t("layers.delete")}
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            );
          })
        )}
      </div>

      {/* 添加图层对话框 */}
      {showAddDialog && (
        <AddLayerDialog
          materials={materials}
          onAdd={handleAdd}
          onClose={() => setShowAddDialog(false)}
        />
      )}
    </div>
  );
}

/** 添加图层对话框：列出所有可用物料供选择。 */
function AddLayerDialog({
  materials,
  onAdd,
  onClose,
}: {
  materials: MaterialInfo[];
  onAdd: (materialId: string, name: string) => void;
  onClose: () => void;
}) {
  const { t } = useI18n();
  const [selected, setSelected] = useState<string | null>(null);
  const [name, setName] = useState("");

  return (
    <div className="absolute inset-0 bg-black/50 flex items-center justify-center z-50" onClick={onClose}>
      <div
        className="bg-background border rounded-lg shadow-lg max-w-md w-full mx-4 max-h-[80vh] flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="p-4 border-b">
          <h3 className="font-semibold">{t("layers.addTitle")}</h3>
        </div>
        <div className="flex-1 overflow-y-auto p-4 space-y-2">
          {materials.map((m) => (
            <label
              key={m.id}
              className={`flex items-center gap-3 p-2 rounded cursor-pointer hover:bg-accent ${
                selected === m.id ? "bg-accent" : ""
              }`}
            >
              <input
                type="radio"
                checked={selected === m.id}
                onChange={() => {
                  setSelected(m.id);
                  setName(m.display_name);
                }}
              />
              <div className="flex-1">
                <div className="text-sm">{m.display_name}</div>
                <div className="text-xs text-muted-foreground">{m.id}</div>
              </div>
              {m.is_dynamic && (
                <span className="text-xs bg-yellow-500/20 text-yellow-700 dark:text-yellow-400 px-2 py-0.5 rounded">
                  {t("layers.dynamic")}
                </span>
              )}
              {m.builtin && (
                <span className="text-xs bg-blue-500/20 text-blue-700 dark:text-blue-400 px-2 py-0.5 rounded">
                  {t("layers.builtin")}
                </span>
              )}
            </label>
          ))}
        </div>
        <div className="p-4 border-t flex items-center gap-2">
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder={t("layers.layerName")}
            className="flex-1 px-3 py-2 text-sm border rounded bg-background"
          />
          <Button
            disabled={!selected || !name.trim()}
            onClick={() => selected && onAdd(selected, name.trim())}
          >
            {t("common.add")}
          </Button>
          <Button variant="ghost" onClick={onClose}>
            {t("common.cancel")}
          </Button>
        </div>
      </div>
    </div>
  );
}

/**
 * 根据物料 schema 动态生成参数控件。
 */
export function MaterialParamControls({
  schema,
  params,
  layerId,
  onChanged,
  locked,
}: {
  schema: MaterialSchemaEntry[];
  params: Record<string, unknown>;
  layerId: string;
  onChanged: (newParams: Record<string, unknown>) => void;
  locked: boolean;
}) {
  const { t } = useI18n();

  if (schema.length === 0) {
    return (
      <div className="text-xs text-muted-foreground italic p-2">
        {t("layers.noParams")}
      </div>
    );
  }

  const updateParam = async (key: string, value: unknown) => {
    const newParams = { ...params, [key]: value };
    onChanged(newParams);
    await invoke("update_layer", { layerId, patch: { params: newParams } });
  };

  return (
    <div className="space-y-3">
      {schema.map((entry) => {
        const value = params[entry.key];
        return (
          <div key={entry.key} className="space-y-1">
            <label className="text-xs font-medium">{entry.label}</label>
            {renderWidget(entry, value, (v) => updateParam(entry.key, v), locked)}
          </div>
        );
      })}
    </div>
  );
}

function renderWidget(
  entry: MaterialSchemaEntry,
  value: unknown,
  onChange: (v: unknown) => void,
  locked: boolean,
) {
  const disabled = locked;
  switch (entry.widget) {
    case "slider":
    case "number":
      return (
        <input
          type="number"
          value={typeof value === "number" ? value : 0}
          min={entry.min}
          max={entry.max}
          step={entry.step}
          disabled={disabled}
          onChange={(e) => onChange(parseFloat(e.target.value))}
          className="w-full px-2 py-1 text-sm border rounded bg-background"
        />
      );
    case "toggle":
      return <Switch checked={!!value} onCheckedChange={onChange} disabled={disabled} />;
    case "select":
      return (
        <select
          value={String(value ?? "")}
          disabled={disabled}
          onChange={(e) => {
            const opt = entry.options?.find((o) => String(o.value) === e.target.value);
            onChange(opt?.value ?? e.target.value);
          }}
          className="w-full px-2 py-1 text-sm border rounded bg-background"
        >
          {entry.options?.map((opt) => (
            <option key={String(opt.value)} value={String(opt.value)}>
              {opt.label}
            </option>
          ))}
        </select>
      );
    case "text":
      return (
        <input
          type="text"
          value={String(value ?? "")}
          disabled={disabled}
          onChange={(e) => onChange(e.target.value)}
          className="w-full px-2 py-1 text-sm border rounded bg-background"
        />
      );
    case "color":
      return (
        <input
          type="color"
          value={rgbToHex(value as [number, number, number, number])}
          disabled={disabled}
          onChange={(e) => onChange(hexToRgba(e.target.value))}
          className="w-full h-8 border rounded bg-background"
        />
      );
    case "image_path":
      return (
        <div className="flex gap-2">
          <input
            type="text"
            value={String(value ?? "")}
            disabled={disabled}
            onChange={(e) => onChange(e.target.value)}
            className="flex-1 px-2 py-1 text-sm border rounded bg-background"
          />
          <Button
            size="sm"
            variant="outline"
            disabled={disabled}
            onClick={async () => {
              const { invoke } = await import("@tauri-apps/api/core");
              const path = await invoke<string | null>("pick_image_path");
              if (path) onChange(path);
            }}
          >
            浏览
          </Button>
        </div>
      );
    default:
      return (
        <div className="text-xs text-muted-foreground italic">
          未支持的 widget 类型: {entry.widget}
        </div>
      );
  }
}

function rgbToHex(color: [number, number, number, number] | undefined): string {
  if (!color) return "#ffffff";
  const r = Math.round(color[0] * 255);
  const g = Math.round(color[1] * 255);
  const b = Math.round(color[2] * 255);
  return `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}`;
}

function hexToRgba(hex: string): [number, number, number, number] {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  return [r, g, b, 1];
}
