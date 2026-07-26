/**
 * 物料运行时（layers + Rhai）软关闭开关。
 *
 * 当前为 `false`：设置面板隐藏图层编辑器入口，仅显示旧版准星设置 UI。
 * 与 Rust 侧 `peregrine::MATERIAL_RUNTIME_ENABLED` 保持一致；
 * 将来修复后两处同时改回 `true` 即可整体恢复。
 */
export const MATERIAL_RUNTIME_ENABLED = false;
