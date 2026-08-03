/**
 * 物料运行时（layers + Rhai）静态渲染开关。
 *
 * 当前为 `true`：设置面板可自由切换单/多图层模式，图层编辑生效。
 * 与 Rust 侧 `peregrine::MATERIAL_RUNTIME_ENABLED` 保持一致；
 * 翻回 `false` 则回退旧版准星 UI（全量软关闭）。
 */
export const MATERIAL_RUNTIME_ENABLED = true;

/**
 * 动态输入（时间 / 鼠标 / 键盘）与动态物料开关。
 *
 * 当前为 `false`：物料选择器隐藏 `is_dynamic` 物料，动态输入相关设置项不可见。
 * 与 Rust 侧 `peregrine::MATERIAL_DYNAMIC_INPUT_ENABLED` 保持一致；
 * 动态链路修复收尾后两处同时改回 `true` 即可恢复动态物料。
 */
export const MATERIAL_DYNAMIC_INPUT_ENABLED = false;
