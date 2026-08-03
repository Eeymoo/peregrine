//! Peregrine 可复用的核心库。
//!
//! 为 Tauri 入口提供 overlay 渲染、平台 API、几何形状等模块。

pub mod overlay_renderer;
pub mod platform;
pub mod shapes;
pub mod svg_renderer;

/// 物料运行时（layers + Rhai）静态渲染开关。
///
/// 当前为 `true`：overlay 与预览按 `Profile.layers` 做多图层静态渲染。
/// 翻回 `false` 则整体回退旧版 `Crosshair` 路径（全量软关闭，见 change
/// `disable-material-runtime`）。与 `MATERIAL_DYNAMIC_INPUT_ENABLED` 相互独立。
pub const MATERIAL_RUNTIME_ENABLED: bool = true;

/// 动态输入（时间 / 鼠标 / 键盘）与动态物料开关。
///
/// 当前为 `false`：不轮询动态输入，物料求值使用
/// `peregrine_material::DynamicContext::static_context()`（version=0，永久缓存），
/// 动态物料冻结渲染且在设置 UI 中不可选；overlay 重绘保持纯事件驱动。
/// 动态链路相关修复见挂起的 change `overlay-dynamic-text-fixes`，
/// 将来收尾后改回 `true` 即可恢复动态物料。
pub const MATERIAL_DYNAMIC_INPUT_ENABLED: bool = false;
