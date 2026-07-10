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
import { pickImagePath } from "@/lib/api";
import type { Crosshair, Anchor, RingStyle, BorderFrameStyle } from "@/types/config";

const ANCHORS: { value: Anchor; label: string }[] = [
  { value: "top", label: "顶部" },
  { value: "bottom", label: "底部" },
  { value: "left", label: "左侧" },
  { value: "right", label: "右侧" },
  { value: "center", label: "居中" },
];

const RING_STYLES: { value: RingStyle; label: string }[] = [
  { value: "solid", label: "实线" },
  { value: "dashed", label: "虚线" },
  { value: "double", label: "双环" },
];

const BORDER_STYLES: { value: BorderFrameStyle; label: string }[] = [
  { value: "solid", label: "完整" },
  { value: "gap", label: "四边缺口" },
];

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
    default:
      return null;
  }
}

/** 贴边矩形。 */
function EdgeRectFields({ crosshair, onChange }: StyleFieldsProps) {
  return (
    <div className="space-y-2">
      <SliderField label="宽度" value={crosshair.size} min={10} max={400} onChange={(v) => onChange({ size: v })} />
      <SliderField label="高度" value={crosshair.secondary_size} min={10} max={300} onChange={(v) => onChange({ secondary_size: v })} />
      <SliderField label="圆角" value={crosshair.corner_radius} min={0} max={60} onChange={(v) => onChange({ corner_radius: v })} />
      <div className="space-y-2">
        <Label className="text-sm">贴边</Label>
        <Select value={crosshair.anchor} onValueChange={(v) => onChange({ anchor: v as Anchor })}>
          <SelectTrigger className="h-8 text-sm"><SelectValue /></SelectTrigger>
          <SelectContent>
            {ANCHORS.map((a) => <SelectItem key={a.value} value={a.value}>{a.label}</SelectItem>)}
          </SelectContent>
        </Select>
      </div>
      <SliderField label="边距" value={crosshair.margin} min={0} max={200} onChange={(v) => onChange({ margin: v })} />
    </div>
  );
}

/** 准星。 */
function CrossFields({ crosshair, onChange }: StyleFieldsProps) {
  return (
    <div className="space-y-2">
      <SliderField label="臂长" value={crosshair.size} min={5} max={200} onChange={(v) => onChange({ size: v })} />
      <SliderField label="线宽" value={crosshair.thickness} min={1} max={20} onChange={(v) => onChange({ thickness: v })} />
      <SliderField label="中心间隙" value={crosshair.gap} min={0} max={50} onChange={(v) => onChange({ gap: v })} />
    </div>
  );
}

/** 大准星。 */
function LargeCrossFields({ crosshair, onChange }: StyleFieldsProps) {
  return (
    <div className="space-y-2">
      <SliderField label="线宽" value={crosshair.thickness} min={1} max={30} onChange={(v) => onChange({ thickness: v })} />
    </div>
  );
}

/** 定位球（4/6/8）。 */
function CornerDotsFields({ crosshair, onChange }: StyleFieldsProps) {
  return (
    <div className="space-y-2">
      <SliderField label="距边缘距离" value={crosshair.offset} min={0} max={200} onChange={(v) => onChange({ offset: v })} />
      <SliderField label={crosshair.radius > 0 ? "半径" : "半径（0=自动）"} value={crosshair.radius} min={0} max={80} onChange={(v) => onChange({ radius: v })} />
      <SliderField label="线宽（自动半径时生效）" value={crosshair.thickness} min={1} max={20} onChange={(v) => onChange({ thickness: v })} />
    </div>
  );
}

/** 中心环。 */
function RingFields({ crosshair, onChange }: StyleFieldsProps) {
  return (
    <div className="space-y-2">
      <SliderField label="半径占屏高比例" value={crosshair.ring_radius_pct} min={0.03} max={0.08} step={0.001} onChange={(v) => onChange({ ring_radius_pct: v })} />
      <SliderField label="线宽" value={crosshair.thickness} min={1} max={3} onChange={(v) => onChange({ thickness: v })} />
      <div className="space-y-2">
        <Label className="text-sm">线型</Label>
        <Select value={crosshair.ring_style} onValueChange={(v) => onChange({ ring_style: v as RingStyle })}>
          <SelectTrigger className="h-8 text-sm"><SelectValue /></SelectTrigger>
          <SelectContent>
            {RING_STYLES.map((s) => <SelectItem key={s.value} value={s.value} className="text-sm">{s.label}</SelectItem>)}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}

/** 自定义定位球。 */
function CustomOrbFields({ crosshair, onChange }: StyleFieldsProps) {
  return (
    <div className="space-y-2">
      <div className="grid grid-cols-2 gap-3">
        <SliderField label="半径" value={crosshair.radius} min={4} max={12} onChange={(v) => onChange({ radius: v })} />
        <SliderField label="距边缘距离" value={crosshair.offset} min={10} max={200} onChange={(v) => onChange({ offset: v })} />
        <SliderField label="上边缘数量" value={crosshair.custom_orb_top_count} min={1} max={10} step={1} onChange={(v) => onChange({ custom_orb_top_count: v })} />
        <SliderField label="下边缘数量" value={crosshair.custom_orb_bottom_count} min={1} max={10} step={1} onChange={(v) => onChange({ custom_orb_bottom_count: v })} />
        <SliderField label="左边缘数量" value={crosshair.custom_orb_left_count} min={1} max={10} step={1} onChange={(v) => onChange({ custom_orb_left_count: v })} />
        <SliderField label="右边缘数量" value={crosshair.custom_orb_right_count} min={1} max={10} step={1} onChange={(v) => onChange({ custom_orb_right_count: v })} />
      </div>
      <OrbPositionCheck crosshair={crosshair} onChange={onChange} />
    </div>
  );
}

/** 随机球。 */
function RandomOrbFields({ crosshair, onChange }: StyleFieldsProps) {
  return (
    <div className="space-y-2">
      <SliderField label="每边数量" value={crosshair.random_orb_count} min={1} max={10} step={1} onChange={(v) => onChange({ random_orb_count: v })} />
      <SliderField label="距边缘距离" value={crosshair.random_orb_offset} min={0} max={300} onChange={(v) => onChange({ random_orb_offset: v })} />
      <SliderField label="位置扰动" value={crosshair.random_orb_jitter} min={0} max={200} onChange={(v) => onChange({ random_orb_jitter: v })} />
      <SliderField label="最小半径" value={crosshair.random_radius_min} min={4} max={12} onChange={(v) => onChange({ random_radius_min: v })} />
      <SliderField label="最大半径" value={crosshair.random_radius_max} min={4} max={12} onChange={(v) => onChange({ random_radius_max: v })} />
    </div>
  );
}

/** 边框。 */
function BorderFrameFields({ crosshair, onChange }: StyleFieldsProps) {
  return (
    <div className="space-y-2">
      <SliderField label="矩形条高度" value={crosshair.thickness} min={1} max={20} onChange={(v) => onChange({ thickness: v })} />
      <SliderField label="距边缘距离" value={crosshair.offset} min={0} max={100} onChange={(v) => onChange({ offset: v })} />
      <div className="space-y-2">
        <Label className="text-sm">样式</Label>
        <Select value={crosshair.border_frame_style} onValueChange={(v) => onChange({ border_frame_style: v as BorderFrameStyle })}>
          <SelectTrigger className="h-8 text-sm"><SelectValue /></SelectTrigger>
          <SelectContent>
            {BORDER_STYLES.map((s) => <SelectItem key={s.value} value={s.value} className="text-sm">{s.label}</SelectItem>)}
          </SelectContent>
        </Select>
      </div>
      <div className="flex items-center gap-3">
        <Checkbox id="border_gap" checked={crosshair.border_gap} onCheckedChange={(v) => onChange({ border_gap: !!v })} />
        <Label htmlFor="border_gap" className="text-sm">四边中间留 20% 缺口</Label>
      </div>
    </div>
  );
}

/** 箭头。 */
function EdgeArrowsFields({ crosshair, onChange }: StyleFieldsProps) {
  return (
    <div className="space-y-2">
      <SliderField label="箭头大小" value={crosshair.size} min={4} max={60} step={1} onChange={(v) => onChange({ size: v })} />
      <SliderField label="宽度(0=等箭头)" value={crosshair.arrow_width} min={0} max={72} step={1} onChange={(v) => onChange({ arrow_width: v })} />
      <div className="flex items-center gap-3">
        <Checkbox id="tail_per_edge" checked={crosshair.arrow_tail_per_edge} onCheckedChange={(v) => onChange({ arrow_tail_per_edge: !!v })} />
        <Label htmlFor="tail_per_edge" className="text-sm">分别设置尾巴长度</Label>
      </div>
      {crosshair.arrow_tail_per_edge ? (
        <div className="grid grid-cols-2 gap-3">
          <SliderField label="上尾巴" value={crosshair.arrow_tail_top} min={0} max={500} step={1} onChange={(v) => onChange({ arrow_tail_top: v })} />
          <SliderField label="下尾巴" value={crosshair.arrow_tail_bottom} min={0} max={500} step={1} onChange={(v) => onChange({ arrow_tail_bottom: v })} />
          <SliderField label="左尾巴" value={crosshair.arrow_tail_left} min={0} max={500} step={1} onChange={(v) => onChange({ arrow_tail_left: v })} />
          <SliderField label="右尾巴" value={crosshair.arrow_tail_right} min={0} max={500} step={1} onChange={(v) => onChange({ arrow_tail_right: v })} />
        </div>
      ) : (
        <SliderField label="尾巴长度" value={crosshair.arrow_distance} min={0} max={500} step={1} onChange={(v) => onChange({ arrow_distance: v })} />
      )}
      <OrbPositionCheck crosshair={crosshair} onChange={onChange} />
    </div>
  );
}

/** 自定义图片。 */
function CustomImageFields({ crosshair, onChange }: StyleFieldsProps) {
  const handlePick = async () => {
    const path = await pickImagePath();
    if (path) onChange({ image_path: path });
  };

  return (
    <div className="space-y-2">
      <div className="space-y-2">
        <Label className="text-sm">文件</Label>
        <div className="flex gap-2">
          <input
            type="text"
            value={crosshair.image_path}
            onChange={(e) => onChange({ image_path: e.target.value })}
            placeholder="PNG 文件路径"
            className="flex-1 h-8 rounded-md border border-input bg-background px-2 text-sm"
          />
          <Button variant="secondary" onClick={handlePick} className="h-8 text-sm px-2">浏览…</Button>
        </div>
      </div>
      <SliderField label="缩放比例" value={crosshair.image_scale} min={0.1} max={5} step={0.1} onChange={(v) => onChange({ image_scale: v })} />
      <SliderField label="水平偏移" value={crosshair.image_offset_x} min={-500} max={500} onChange={(v) => onChange({ image_offset_x: v })} />
      <SliderField label="垂直偏移" value={crosshair.image_offset_y} min={-500} max={500} onChange={(v) => onChange({ image_offset_y: v })} />
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
  const pos = crosshair.orb_positions;
  const set = (flag: number, checked: boolean) => {
    let next = pos;
    if (checked) next |= flag;
    else next &= ~flag;
    onChange({ orb_positions: next });
  };
  return (
    <div className="space-y-2">
      <Label className="text-sm">启用</Label>
      <div className="flex gap-4">
        {[
          { flag: 0b0001, label: "上" },
          { flag: 0b0010, label: "下" },
          { flag: 0b0100, label: "左" },
          { flag: 0b1000, label: "右" },
        ].map(({ flag, label }) => (
          <div key={label} className="flex items-center gap-1">
            <Checkbox
              id={`orb-${label}`}
              checked={!!(pos & flag)}
              onCheckedChange={(v) => set(flag, !!v)}
            />
            <Label htmlFor={`orb-${label}`} className="text-sm">{label}</Label>
          </div>
        ))}
      </div>
    </div>
  );
}
