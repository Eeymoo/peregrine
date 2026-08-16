import { useEffect, useMemo, useState, useCallback } from "react";
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
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Trash2, Copy, ChevronUp, ChevronDown, Plus, Eye, EyeOff, Lock, Unlock } from "lucide-react";
import { logAction } from "@/lib/actionLog";
import { useI18n } from "@/lib/i18n";
import { localizeMaterial } from "@/lib/material-i18n";
import { dynamicInputEnabled } from "@/lib/feature";
import { SliderField } from "@/components/fields/SliderField";
import { NumberField } from "@/components/fields/NumberField";
import { TextField } from "@/components/fields/TextField";
import { ColorField, type Rgba } from "@/components/fields/ColorField";
import { ToggleField } from "@/components/fields/ToggleField";
import { SelectField, type SelectOption } from "@/components/fields/SelectField";
import { ImagePathField } from "@/components/fields/ImagePathField";

interface LayerPanelProps {
  /** 图层数组，包含所有显示的图层信息 */
  layers: Layer[];
  /** 当前选中的图层ID，为null表示未选中任何图层 */
  selectedLayerId: string | null;
  /** 选择图层的回调函数，接收被选中的图层ID */
  onSelectLayer: (id: string) => void;
  /** 图层数据变化后的回调函数，用于刷新图层数据 */
  onChanged: () => void;
  /** overlay 是否处于活动状态（渲染不变量保护：禁用使可见图层归零的删除/隐藏） */
  overlayActive: boolean;
  /** 是否存在 legacy crosshair（仅 layers 为空时作为渲染兜底、豁免保护） */
  hasCrosshair: boolean;
  /** 动态物料运行时开关（settings.material.dynamic_enabled），用于物料选择器合取过滤 */
  dynamicMaterialEnabled?: boolean;
}

/**
 * 图层管理面板：显示图层列表，支持增删/排序/可见性/复制。
 */
export function LayerPanel({
  layers,
  selectedLayerId,
  onSelectLayer,
  onChanged,
  overlayActive,
  hasCrosshair,
  dynamicMaterialEnabled,
}: LayerPanelProps) {
  const { t, has, resolvedLocale } = useI18n();
  const [materials, setMaterials] = useState<MaterialInfo[]>([]);
  const [showAddDialog, setShowAddDialog] = useState(false);

  // 内置物料展示文案按当前界面语言覆盖（用户物料原样返回，见 material-i18n.ts）。
  // t/has 行为仅随 resolvedLocale 变化，以其为依赖即可。
  const localizedMaterials = useMemo(
    () => materials.map((m) => localizeMaterial(m, { t, has })),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [materials, resolvedLocale],
  );

  // 加载物料列表（仅一次）。
  // 动态输入合取门控：编译期总闸 AND 运行时用户开关（design D2）。
  // 任一关闭时过滤 is_dynamic 物料（配置保留，渲染冻结）；
  // 运行时开关变化会以 dynamicMaterialEnabled 为依赖重拉列表，热生效。
  // custom_image 暂隐藏：图片渲染链路当前不可用（不渲染图片），待后续修复后恢复。
  useEffect(() => {
    listMaterials()
      .then((list) =>
        setMaterials(
          (dynamicInputEnabled(dynamicMaterialEnabled)
            ? list
            : list.filter((m) => !m.is_dynamic)
          ).filter((m) => m.id !== "builtin.custom_image"),
        ),
      )
      .catch(() => {
        // listMaterials 失败由 invoke 包装 toast 提示；这里不阻塞 UI。
      });
  }, [dynamicMaterialEnabled]);

  const handleAdd = (materialId: string, name: string) => {
    logAction("add-layer", { materialId, name });
    addLayer(materialId, name)
      .then(() => {
        setShowAddDialog(false);
        onChanged();
      })
      .catch(() => {
        // invoke 包装已 toast；吞掉 rejection 防止 unhandled rejection 上报（PGR-3003）。
      });
  };

  const handleDelete = (id: string) => {
    logAction("remove-layer", { id });
    removeLayer(id)
      .then(() => onChanged())
      .catch(() => {
        // invoke 包装已 toast；吞掉 rejection 防止 unhandled rejection 上报（PGR-3003）。
      });
  };

  const handleDuplicate = (id: string) => {
    logAction("duplicate-layer", { id });
    duplicateLayer(id)
      .then(() => onChanged())
      .catch(() => {
        // invoke 包装已 toast；吞掉 rejection 防止 unhandled rejection 上报（PGR-3003）。
      });
  };

  const handleMove = (id: string, direction: "up" | "down") => {
    const idx = layers.findIndex((l) => l.id === id);
    if (idx < 0) return;
    // 列表反序显示（顶层图层在最上，Photoshop 习惯），
    // 因此视觉上"上移" = 数组索引 +1（更晚渲染、更靠顶层）。
    const newIdx = direction === "up" ? Math.min(layers.length - 1, idx + 1) : Math.max(0, idx - 1);
    if (newIdx === idx) return;
    logAction("move-layer", { id, from: idx, to: newIdx });
    moveLayer(id, newIdx)
      .then(() => onChanged())
      .catch(() => {
        // invoke 包装已 toast；吞掉 rejection 防止 unhandled rejection 上报（PGR-3003）。
      });
  };

  const handleToggleVisible = (layer: Layer) => {
    logAction("toggle-visible", { id: layer.id, visible: !layer.visible });
    updateLayer(layer.id, { visible: !layer.visible })
      .then(() => onChanged())
      .catch(() => {
        // invoke 包装已 toast；吞掉 rejection 防止 unhandled rejection 上报（PGR-3003）。
      });
  };

  const handleToggleLock = (layer: Layer) => {
    logAction("toggle-lock", { id: layer.id, locked: !layer.locked });
    updateLayer(layer.id, { locked: !layer.locked })
      .then(() => onChanged())
      .catch(() => {
        // invoke 包装已 toast；吞掉 rejection 防止 unhandled rejection 上报（PGR-3003）。
      });
  };

  return (
    <div className="flex flex-col h-full bg-card border-l relative">
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
        {layers.length === 0 ? (
          <div className="p-4 text-center text-sm text-muted-foreground">
            {t("layers.empty")}
          </div>
        ) : (
          (() => {
            // 渲染不变量：overlay 活动中，使可见图层归零的删除/隐藏需禁用（后端 Err 为最终防线）。
            // 豁免条件与渲染路径一致：crosshair 仅在 layers 为空时才被渲染（legacy 兜底）；
            // layers 非空时 crosshair 不参与渲染（双写配置也不能豁免），
            // 即删除/隐藏最后一个可见图层在 layers 非空时一律禁用。
            const visibleCount = layers.filter((l) => l.visible).length;
            const legacyFallback = layers.length === 0 && hasCrosshair;
            const protectLastVisible = overlayActive && !legacyFallback;
            const reversedLayers = [...layers].reverse();
            return reversedLayers.map((layer) => {
              const isLastVisible = protectLastVisible && layer.visible && visibleCount <= 1;
            const materialId =
              layer.material.kind === "builtin" ? layer.material.id : layer.material.name;
            const material = localizedMaterials.find((m) => m.id === materialId);
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
                  disabled={isLastVisible}
                  className="text-muted-foreground hover:text-foreground disabled:opacity-30 disabled:cursor-not-allowed"
                  title={
                    isLastVisible
                      ? t("layers.lastVisibleLocked")
                      : layer.visible
                        ? t("layers.hide")
                        : t("layers.show")
                  }
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
                  disabled={isLastVisible}
                  className="text-muted-foreground hover:text-red-500 disabled:opacity-30 disabled:cursor-not-allowed"
                  title={isLastVisible ? t("layers.lastVisibleLocked") : t("layers.delete")}
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            );
          });
          })()
        )}
      </div>

      {/* 添加图层对话框 */}
      {showAddDialog && (
        <AddLayerDialog
          materials={localizedMaterials}
          onAdd={handleAdd}
          onClose={() => setShowAddDialog(false)}
        />
      )}
    </div>
  );
}

/** 添加图层对话框：列出所有可用物料供选择。
 * 
 * @param materials - 可用物料信息列表
 * @param onAdd - 添加图层回调函数，接收物料ID和图层名称
 * @param onClose - 关闭对话框回调函数
 */
function AddLayerDialog({
  materials,
  onAdd,
  onClose,
}: {
  /** 可用物料信息列表，包含所有内置和自定义物料 */
  materials: MaterialInfo[];
  /** 添加图层回调函数，接收物料ID和图层名称 */
  onAdd: (materialId: string, name: string) => void;
  /** 关闭对话框回调函数 */
  onClose: () => void;
}) {
  const { t } = useI18n();
  const [selected, setSelected] = useState<string | null>(null);
  const [name, setName] = useState("");

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-[100]" onClick={onClose}>
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
  /** 物料参数schema定义，包含参数类型、范围、默认值等信息 */
  schema: MaterialSchemaEntry[];
  /** 当前图层的参数值对象 */
  params: Record<string, unknown>;
  /** 图层ID，用于更新参数时的标识 */
  layerId: string;
  /** 参数更新后的回调函数，接收新的参数对象 */
  onChanged: (newParams: Record<string, unknown>) => void;
  /** 是否锁定编辑状态，锁定时禁用所有输入控件 */
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

  const updateParam = useCallback(async (key: string, value: unknown) => {
    logAction("update-layer-param", { layerId, key, value });
    const newParams = { ...params, [key]: value };
    onChanged(newParams);
    await invoke("update_layer", { layerId, patch: { params: newParams } });
  }, [layerId, onChanged, params]);

  return (
    <div className="space-y-3">
      {schema.map((entry) => {
        const value = params[entry.key];
        return (
          <div key={entry.key}>
            {renderWidget(entry, value, (v) => updateParam(entry.key, v), locked)}
          </div>
        );
      })}
    </div>
  );
}

/** 根据物料 schema 的 widget 类型分发到对应的共享字段组件。
 *
 * 每个字段组件内部负责渲染自己的 label（统一两行布局规范）。
 * slider 与 number 必须区分对待：slider 渲染为可拖拽 SliderField，
 * number 渲染为纯 NumberField（无滑块）。
 */
function renderWidget(
  entry: MaterialSchemaEntry,
  value: unknown,
  onChange: (v: unknown) => void,
  locked: boolean,
) {
  const { t } = useI18n();
  const disabled = locked;
  switch (entry.widget) {
    case "slider":
      return (
        <SliderField
          label={entry.label}
          value={typeof value === "number" ? value : 0}
          min={entry.min}
          max={entry.max}
          step={entry.step}
          disabled={disabled}
          onChange={(v) => onChange(v)}
        />
      );
    case "number":
      // 位掩码字段（custom_orb.orb_positions / edge_arrows.positions_mask）：
      // 渲染为 4 个 checkbox，提升多图层模式下的可操作性。
      if (entry.bitmask) {
        return (
          <BitmaskField
            label={entry.label}
            value={typeof value === "number" ? value : 0}
            disabled={disabled}
            onChange={(v) => onChange(v)}
          />
        );
      }
      return (
        <NumberField
          label={entry.label}
          value={typeof value === "number" ? value : 0}
          min={entry.min}
          max={entry.max}
          step={entry.step}
          disabled={disabled}
          onChange={(v) => onChange(v)}
        />
      );
    case "text":
      return (
        <TextField
          label={entry.label}
          value={String(value ?? "")}
          disabled={disabled}
          onChange={(v) => onChange(v)}
        />
      );
    case "color":
      return (
        <ColorField
          label={entry.label}
          value={(value as Rgba) ?? [1, 1, 1, 1]}
          disabled={disabled}
          onChange={(v) => onChange(v)}
        />
      );
    case "toggle":
      return (
        <ToggleField
          label={entry.label}
          value={!!value}
          disabled={disabled}
          onChange={(v) => onChange(v)}
        />
      );
    case "select": {
      // schema entry 的 options 转换为 SelectField 所需的 {value, label}[]。
      const options: SelectOption[] = (entry.options ?? []).map((opt) => ({
        value: String(opt.value),
        label: opt.label,
      }));
      // 任务 9.7：物料 schema 标记 coming_soon 的 select 控件禁用，
      // 并在 label 后追加「开发中」后缀提示（random_orb.mode 等）。
      const comingSoon = entry.coming_soon === true;
      // 原始 schema option value 可能是数字（corner_dots.count）或字符串（mode）。
      // SelectField 只接受 string，但回传给 onChange 时需按原始类型还原，
      // 否则 Rhai 侧 `params.count >= 6` 会因类型不匹配而失败（"6" vs 6）。
      const originalOptions = entry.options ?? [];
      return (
        <SelectField
          label={comingSoon ? `${entry.label}${t("layers.comingSoonSuffix")}` : entry.label}
          value={String(value ?? "")}
          options={options}
          disabled={disabled || comingSoon}
          onChange={(v) => {
            const matched = originalOptions.find((opt) => String(opt.value) === v);
            onChange(matched ? matched.value : v);
          }}
        />
      );
    }
    case "image_path":
      return (
        <ImagePathField
          label={entry.label}
          value={String(value ?? "")}
          disabled={disabled}
          onChange={(v) => onChange(v)}
        />
      );
    default:
      return (
        <div className="text-xs text-muted-foreground italic">
          {t("common.unknown")} widget: {entry.widget}
        </div>
      );
  }
}

/** 位掩码字段：把 4 位整数（1=top, 2=bottom, 4=left, 8=right）渲染为 4 个 checkbox。
 *
 * 用于 custom_orb.orb_positions、edge_arrows.positions_mask 等位掩码字段，
 * 替代裸数字输入，避免用户手算位运算。
 */
function BitmaskField({
  label,
  value,
  disabled,
  onChange,
}: {
  label: string;
  value: number;
  disabled?: boolean;
  onChange: (v: number) => void;
}) {
  const { t } = useI18n();
  const items: { flag: number; key: string; label: string }[] = [
    { flag: 0b0001, key: "top", label: t("fields.top") },
    { flag: 0b0010, key: "bottom", label: t("fields.bottom") },
    { flag: 0b0100, key: "left", label: t("fields.left") },
    { flag: 0b1000, key: "right", label: t("fields.right") },
  ];
  const set = (flag: number, checked: boolean) => {
    onChange(checked ? value | flag : value & ~flag);
  };
  return (
    <div className="space-y-2">
      <Label className="text-sm">{label}</Label>
      <div className="flex flex-wrap gap-4">
        {items.map(({ flag, key, label: itemLabel }) => (
          <div key={key} className="flex items-center gap-1">
            <Checkbox
              id={`bitmask-${key}`}
              checked={(value & flag) !== 0}
              disabled={disabled}
              onCheckedChange={(v) => set(flag, !!v)}
            />
            <Label htmlFor={`bitmask-${key}`} className="text-sm">
              {itemLabel}
            </Label>
          </div>
        ))}
      </div>
    </div>
  );
}
