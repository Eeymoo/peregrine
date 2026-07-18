import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useI18n } from "@/lib/i18n";
import { pickImagePath } from "@/lib/api";
import type { Crosshair, Anchor, RingStyle, BorderFrameStyle, GridAlignment } from "@/types/config";

const ANCHORS: Anchor[] = ["top", "bottom", "left", "right", "center"];
const RING_STYLES: RingStyle[] = ["solid", "dashed", "double"];
const BORDER_STYLES: BorderFrameStyle[] = ["solid", "gap"];
const GRID_ALIGNMENTS: GridAlignment[] = ["center", "edge"];

interface StyleFieldsProps {
  crosshair: Crosshair;
  onChange: (patch: Partial<Crosshair>) => void;
}

/** 根据当前准心样式分发到对应小组件。 */
export function StyleFields({ crosshair, onChange }: StyleFieldsProps) {
  switch (crosshair.style) {
    case "edge_rect":
      return <EdgeRectFields crosshair={crosshair} onChange={onChange} />;
    case "cross":
      return <CrossFields crosshair={crosshair} onChange={onChange} />;
    case "large_cross":
      return <LargeCrossFields crosshair={crosshair} onChange={onChange} />;
    case "corner_dots4":
    case "corner_dots6":
    case "corner_dots8":
      return <CornerDotsFields crosshair={crosshair} onChange={onChange} />;
    case "ring":
      return <RingFields crosshair={crosshair} onChange={onChange} />;
    case "custom_orb":
      return <CustomOrbFields crosshair={crosshair} onChange={onChange} />;
    case "random_orb":
      return <RandomOrbFields crosshair={crosshair} onChange={onChange} />;
    case "border_frame":
      return <BorderFrameFields crosshair={crosshair} onChange={onChange} />;
    case "custom_image":
      return <CustomImageFields crosshair={crosshair} onChange={onChange} />;
    case "edge_arrows":
      return <EdgeArrowsFields crosshair={crosshair} onChange={onChange} />;
    case "grid":
      return <GridFields crosshair={crosshair} onChange={onChange} />;
    default:
      return null;
  }
}

/** 贴边矩形。 */
function EdgeRectFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <SliderField label={t("fields.width")} value={crosshair.size} min={10} max={400} onChange={(v) => onChange({ size: v })} />
      <SliderField label={t("fields.height")} value={crosshair.secondary_size} min={10} max={300} onChange={(v) => onChange({ secondary_size: v })} />
      <SliderField label={t("fields.cornerRadius")} value={crosshair.corner_radius} min={0} max={60} onChange={(v) => onChange({ corner_radius: v })} />
      <div className="space-y-2">
        <Label className="text-sm">{t("fields.anchor")}</Label>
        <Select value={crosshair.anchor} onValueChange={(v) => onChange({ anchor: v as Anchor })}>
          <SelectTrigger className="h-8 text-sm"><SelectValue /></SelectTrigger>
          <SelectContent>
            {ANCHORS.map((a) => <SelectItem key={a} value={a}>{t(`anchors.${a}`)}</SelectItem>)}
          </SelectContent>
        </Select>
      </div>
      <SliderField label={t("fields.margin")} value={crosshair.margin} min={0} max={200} onChange={(v) => onChange({ margin: v })} />
    </div>
  );
}

/** 准星。 */
function CrossFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <SliderField label={t("fields.armLength")} value={crosshair.size} min={5} max={200} onChange={(v) => onChange({ size: v })} />
      <SliderField label={t("fields.lineWidth")} value={crosshair.thickness} min={1} max={20} onChange={(v) => onChange({ thickness: v })} />
      <SliderField label={t("fields.centerGap")} value={crosshair.gap} min={0} max={50} onChange={(v) => onChange({ gap: v })} />
    </div>
  );
}

/** 大准星。 */
function LargeCrossFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <SliderField label={t("fields.lineWidth")} value={crosshair.thickness} min={1} max={30} onChange={(v) => onChange({ thickness: v })} />
    </div>
  );
}

/** 定位球（4/6/8）。 */
function CornerDotsFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <SliderField label={t("fields.offset")} value={crosshair.offset} min={0} max={200} onChange={(v) => onChange({ offset: v })} />
      <SliderField
        label={crosshair.radius > 0 ? t("fields.radius") : t("fields.radiusAuto")}
        value={crosshair.radius}
        min={0}
        max={80}
        onChange={(v) => onChange({ radius: v })}
      />
      <SliderField label={t("fields.lineWidthAuto")} value={crosshair.thickness} min={1} max={20} onChange={(v) => onChange({ thickness: v })} />
    </div>
  );
}

/** 中心环。 */
function RingFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <SliderField label={t("fields.ringRadiusPct")} value={crosshair.ring_radius_pct} min={0.03} max={0.08} step={0.001} onChange={(v) => onChange({ ring_radius_pct: v })} />
      <SliderField label={t("fields.lineWidth")} value={crosshair.thickness} min={1} max={3} onChange={(v) => onChange({ thickness: v })} />
      <div className="space-y-2">
        <Label className="text-sm">{t("fields.ringStyle")}</Label>
        <Select value={crosshair.ring_style} onValueChange={(v) => onChange({ ring_style: v as RingStyle })}>
          <SelectTrigger className="h-8 text-sm"><SelectValue /></SelectTrigger>
          <SelectContent>
            {RING_STYLES.map((s) => <SelectItem key={s} value={s} className="text-sm">{t(`ringStyles.${s}`)}</SelectItem>)}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}

/** 自定义定位球。 */
function CustomOrbFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <div className="grid grid-cols-2 gap-3">
        <SliderField label={t("fields.radius")} value={crosshair.radius} min={4} max={12} onChange={(v) => onChange({ radius: v })} />
        <SliderField label={t("fields.offset")} value={crosshair.offset} min={10} max={200} onChange={(v) => onChange({ offset: v })} />
        <SliderField label={t("fields.countTop")} value={crosshair.custom_orb_top_count} min={1} max={10} step={1} onChange={(v) => onChange({ custom_orb_top_count: v })} />
        <SliderField label={t("fields.countBottom")} value={crosshair.custom_orb_bottom_count} min={1} max={10} step={1} onChange={(v) => onChange({ custom_orb_bottom_count: v })} />
        <SliderField label={t("fields.countLeft")} value={crosshair.custom_orb_left_count} min={1} max={10} step={1} onChange={(v) => onChange({ custom_orb_left_count: v })} />
        <SliderField label={t("fields.countRight")} value={crosshair.custom_orb_right_count} min={1} max={10} step={1} onChange={(v) => onChange({ custom_orb_right_count: v })} />
      </div>
      <OrbPositionCheck crosshair={crosshair} onChange={onChange} />
    </div>
  );
}

/** 随机球。 */
function RandomOrbFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <SliderField label={t("fields.perEdgeCount")} value={crosshair.random_orb_count} min={1} max={10} step={1} onChange={(v) => onChange({ random_orb_count: v })} />
      <SliderField label={t("fields.offset")} value={crosshair.random_orb_offset} min={0} max={300} onChange={(v) => onChange({ random_orb_offset: v })} />
      <SliderField label={t("fields.positionJitter")} value={crosshair.random_orb_jitter} min={0} max={200} onChange={(v) => onChange({ random_orb_jitter: v })} />
      <SliderField label={t("fields.radiusMin")} value={crosshair.random_radius_min} min={4} max={12} onChange={(v) => onChange({ random_radius_min: v })} />
      <SliderField label={t("fields.radiusMax")} value={crosshair.random_radius_max} min={4} max={12} onChange={(v) => onChange({ random_radius_max: v })} />
    </div>
  );
}

/** 边框。 */
function BorderFrameFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <SliderField label={t("fields.barHeight")} value={crosshair.thickness} min={1} max={20} onChange={(v) => onChange({ thickness: v })} />
      <SliderField label={t("fields.offset")} value={crosshair.offset} min={0} max={100} onChange={(v) => onChange({ offset: v })} />
      <div className="space-y-2">
        <Label className="text-sm">{t("fields.borderStyle")}</Label>
        <Select value={crosshair.border_frame_style} onValueChange={(v) => onChange({ border_frame_style: v as BorderFrameStyle })}>
          <SelectTrigger className="h-8 text-sm"><SelectValue /></SelectTrigger>
          <SelectContent>
            {BORDER_STYLES.map((s) => <SelectItem key={s} value={s} className="text-sm">{t(`borderStyles.${s}`)}</SelectItem>)}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}

/** 箭头。 */
function EdgeArrowsFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <SliderField label={t("fields.arrowSize")} value={crosshair.size} min={4} max={60} step={1} onChange={(v) => onChange({ size: v })} />
      <SliderField label={t("fields.arrowWidth")} value={crosshair.arrow_width} min={0} max={72} step={1} onChange={(v) => onChange({ arrow_width: v })} />
      <div className="flex items-center gap-3">
        <Checkbox id="tail_per_edge" checked={crosshair.arrow_tail_per_edge} onCheckedChange={(v) => onChange({ arrow_tail_per_edge: !!v })} />
        <Label htmlFor="tail_per_edge" className="text-sm">{t("fields.tailPerEdge")}</Label>
      </div>
      {crosshair.arrow_tail_per_edge ? (
        <div className="grid grid-cols-2 gap-3">
          <SliderField label={t("fields.tailTop")} value={crosshair.arrow_tail_top} min={0} max={500} step={1} onChange={(v) => onChange({ arrow_tail_top: v })} />
          <SliderField label={t("fields.tailBottom")} value={crosshair.arrow_tail_bottom} min={0} max={500} step={1} onChange={(v) => onChange({ arrow_tail_bottom: v })} />
          <SliderField label={t("fields.tailLeft")} value={crosshair.arrow_tail_left} min={0} max={500} step={1} onChange={(v) => onChange({ arrow_tail_left: v })} />
          <SliderField label={t("fields.tailRight")} value={crosshair.arrow_tail_right} min={0} max={500} step={1} onChange={(v) => onChange({ arrow_tail_right: v })} />
        </div>
      ) : (
        <SliderField label={t("fields.tailLength")} value={crosshair.arrow_distance} min={0} max={500} step={1} onChange={(v) => onChange({ arrow_distance: v })} />
      )}
      <OrbPositionCheck crosshair={crosshair} onChange={onChange} />
    </div>
  );
}

/** 网格。 */
function GridFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-2">
      <SliderField label={t("fields.gridSize")} value={crosshair.grid_size ?? 80} min={10} max={500} step={5} onChange={(v) => onChange({ grid_size: v })} />
      <SliderField label={t("fields.lineWidth")} value={crosshair.thickness} min={1} max={20} onChange={(v) => onChange({ thickness: v })} />
      <div className="space-y-2">
        <Label className="text-sm">{t("fields.gridAlignment")}</Label>
        <Select value={crosshair.grid_alignment ?? "center"} onValueChange={(v) => onChange({ grid_alignment: v as GridAlignment })}>
          <SelectTrigger className="h-8 text-sm"><SelectValue /></SelectTrigger>
          <SelectContent>
            {GRID_ALIGNMENTS.map((a) => <SelectItem key={a} value={a} className="text-sm">{t(`gridAlignments.${a}`)}</SelectItem>)}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}

/** 自定义图片。 */
function CustomImageFields({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  const handlePick = async () => {
    const path = await pickImagePath();
    if (path) onChange({ image_path: path });
  };

  return (
    <div className="space-y-2">
      <div className="space-y-2">
        <Label className="text-sm">{t("fields.file")}</Label>
        <div className="flex gap-2">
          <input
            type="text"
            value={crosshair.image_path}
            onChange={(e) => onChange({ image_path: e.target.value })}
            placeholder={t("fields.imagePathPlaceholder")}
            className="flex-1 h-8 rounded-md border border-input bg-background px-2 text-sm"
          />
          <Button variant="secondary" onClick={handlePick} className="h-8 text-sm px-2">{t("fields.browse")}</Button>
        </div>
      </div>
      <SliderField label={t("fields.imageScale")} value={crosshair.image_scale} min={0.1} max={5} step={0.1} onChange={(v) => onChange({ image_scale: v })} />
      <SliderField label={t("fields.imageOffsetX")} value={crosshair.image_offset_x} min={-500} max={500} onChange={(v) => onChange({ image_offset_x: v })} />
      <SliderField label={t("fields.imageOffsetY")} value={crosshair.image_offset_y} min={-500} max={500} onChange={(v) => onChange({ image_offset_y: v })} />
    </div>
  );
}

/** 通用滑块字段。 */
function SliderField({
  label,
  value,
  min,
  max,
  step = 1,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (v: number) => void;
}) {
  return (
    <div className="space-y-2">
      <div className="flex justify-between">
        <Label className="text-sm">{label}</Label>
        <span className="text-sm text-muted-foreground">{Number(value).toFixed(step < 1 ? 2 : 0)}</span>
      </div>
      <Slider
        value={[value]}
        min={min}
        max={max}
        step={step}
        onValueChange={([v]) => onChange(v)}
      />
    </div>
  );
}

/** 边缘位置复选框（上/下/左/右）。 */
function OrbPositionCheck({ crosshair, onChange }: StyleFieldsProps) {
  const { t } = useI18n();
  const pos = crosshair.orb_positions;
  const set = (flag: number, checked: boolean) => {
    let next = pos;
    if (checked) next |= flag;
    else next &= ~flag;
    onChange({ orb_positions: next });
  };
  const items: { flag: number; key: string; label: string }[] = [
    { flag: 0b0001, key: "top", label: t("fields.top") },
    { flag: 0b0010, key: "bottom", label: t("fields.bottom") },
    { flag: 0b0100, key: "left", label: t("fields.left") },
    { flag: 0b1000, key: "right", label: t("fields.right") },
  ];
  return (
    <div className="space-y-2">
      <Label className="text-sm">{t("fields.enabled")}</Label>
      <div className="flex flex-wrap gap-4">
        {items.map(({ flag, key, label }) => (
          <div key={key} className="flex items-center gap-1">
            <Checkbox
              id={`orb-${key}`}
              checked={!!(pos & flag)}
              onCheckedChange={(v) => set(flag, !!v)}
            />
            <Label htmlFor={`orb-${key}`} className="text-sm">{label}</Label>
          </div>
        ))}
      </div>
    </div>
  );
}
