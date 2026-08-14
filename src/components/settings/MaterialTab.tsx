import { useI18n } from "@/lib/i18n";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { updatePreferences } from "@/lib/api";
import { MATERIAL_RUNTIME_ENABLED, dynamicInputEnabled } from "@/lib/feature";
import type { AppConfig } from "@/types/config";

interface MaterialTabProps {
  config: AppConfig;
  setConfig: (cfg: AppConfig) => void;
}

/** 帧率档位值（与后端校验枚举集一致：30 / 60 / 120；"system" = None 跟随系统）。 */
const FPS_OPTIONS = [
  { value: "system", fps: null },
  { value: "30", fps: 30 },
  { value: "60", fps: 60 },
  { value: "120", fps: 120 },
] as const;

/**
 * 「物料」Tab：动态物料运行时开关 + 动画帧率选择器。
 *
 * - 开关走 `settings.material.dynamic_enabled`（与编译期总闸构成与门），
 *   变更经 update_preferences 保存 → overlay UpdateConfig 路径热生效；
 * - 帧率走 `settings.material.fps`（None = 跟随系统主屏刷新率，回退 60），
 *   语义为「动画最高帧率节拍（cap）」，纯静态 profile 不受影响。
 *
 * 整个 Tab 在 `MATERIAL_RUNTIME_ENABLED = false`（编译期软关闭）时由
 * SettingsApp 隐藏，此处无需重复判断。
 */
export function MaterialTab({ config, setConfig }: MaterialTabProps) {
  const { t } = useI18n();
  const material = config.settings?.material;
  const dynamicEnabled = material?.dynamic_enabled ?? true;
  const fps = material?.fps ?? null;

  /** 更新 settings.material 子对象并持久化（update_preferences 走后端 patch）。 */
  const updateMaterial = (patch: Partial<NonNullable<AppConfig["settings"]["material"]>>) => {
    const next = { ...(material ?? { dynamic_enabled: true }), ...patch };
    const newConfig: AppConfig = {
      ...config,
      settings: { ...config.settings, material: next },
    };
    setConfig(newConfig);
    updatePreferences({ material: next }).catch(() => {
      // updatePreferences 失败由 invoke 包装 toast；settings-changed 事件会回滚本地态。
    });
  };

  // 编译期总闸关闭时（软关闭构建），运行时开关无消费方——禁用控件并提示。
  const runtimeDisabled = !MATERIAL_RUNTIME_ENABLED;

  return (
    <div className="space-y-6">
      {/* 动态物料开关 */}
      <div className="flex items-center justify-between gap-4">
        <div className="space-y-0.5">
          <Label className="text-sm font-medium">{t("material.dynamicEnabled")}</Label>
          <p className="text-xs text-muted-foreground">
            {t("material.dynamicEnabledHint")}
          </p>
        </div>
        <Switch
          checked={dynamicEnabled}
          disabled={runtimeDisabled}
          onCheckedChange={(v) => updateMaterial({ dynamic_enabled: v })}
        />
      </div>

      {/* 动画帧率选择器（单选组：系统 / 30 / 60 / 120） */}
      <div className="space-y-2">
        <div className="space-y-0.5">
          <Label className="text-sm font-medium">{t("material.fps")}</Label>
          <p className="text-xs text-muted-foreground">
            {t("material.fpsHint")}
          </p>
        </div>
        <RadioGroup
          value={fps === null ? "system" : String(fps)}
          onValueChange={(v) => {
            const matched = FPS_OPTIONS.find((o) => o.value === v);
            updateMaterial({ fps: matched ? (matched.fps as 30 | 60 | 120 | null) : null });
          }}
          className="flex flex-row gap-6"
          disabled={runtimeDisabled}
        >
          {FPS_OPTIONS.map((opt) => (
            <div key={opt.value} className="flex items-center gap-2">
              <RadioGroupItem value={opt.value} id={`fps-${opt.value}`} />
              <Label
                htmlFor={`fps-${opt.value}`}
                className="text-sm font-normal cursor-pointer"
              >
                {opt.value === "system" ? t("material.fpsSystem") : opt.value}
              </Label>
            </div>
          ))}
        </RadioGroup>
      </div>

      {/* 当前动态链路状态说明（合取判定结果） */}
      <p className="text-xs text-muted-foreground">
        {dynamicInputEnabled(dynamicEnabled)
          ? t("material.statusActive")
          : t("material.statusFrozen")}
      </p>
    </div>
  );
}
