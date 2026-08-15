/**
 * 物料运行时（layers + Rhai）静态渲染开关。
 *
 * 当前为 `true`：设置面板可自由切换单/多图层模式，图层编辑生效。
 * 与 Rust 侧 `peregrine::MATERIAL_RUNTIME_ENABLED` 保持一致；
 * 翻回 `false` 则回退旧版准星 UI（全量软关闭）。
 */
export const MATERIAL_RUNTIME_ENABLED = true;

/**
 * 动态输入（时间 / 鼠标 / 键盘）与动态物料开关（编译期总闸）。
 *
 * 当前为 `true`（restore-dynamic-material 起恢复）：动态链路默认活跃。
 * 与 Rust 侧 `peregrine::MATERIAL_DYNAMIC_INPUT_ENABLED` 保持一致；
 * 运行时层另有用户开关 `settings.material.dynamic_enabled`（默认 true），
 * 与本常量构成与门：任一层关闭即隐藏 `is_dynamic` 物料、冻结动态输入。
 * 两处同时翻回 `false` 可整体回退软关闭行为。
 */
export const MATERIAL_DYNAMIC_INPUT_ENABLED = true;

/**
 * 动态链路合取判定：编译期总闸 AND 运行时用户开关。
 *
 * 运行时开关读取自 `settings.material.dynamic_enabled`（默认 true），
 * 与 Rust 侧四个门控点的判定表达式保持一致（见 design D2）。
 */
export function dynamicInputEnabled(runtimeSetting: boolean | undefined | null): boolean {
  return MATERIAL_DYNAMIC_INPUT_ENABLED && (runtimeSetting ?? true);
}
