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

/// 动态输入（时间 / 鼠标 / 键盘）与动态物料开关（编译期总闸）。
///
/// 当前为 `true`（change `restore-dynamic-material` 起恢复）：动态链路默认活跃，
/// overlay 轮询动态输入、动态物料按帧率档位持续重绘、选择器展示动态物料。
/// 运行时层另有用户开关 `settings.material.dynamic_enabled`（默认 true），
/// 与本常量构成**与门**：任一层关闭即用户侧软关闭（求值走
/// `DynamicContext::static_context()`、动态判定恒 false、选择器隐藏动态物料）。
/// 翻回 `false` 可整体回退到 `material-static-rendering` 时代的软关闭行为
/// （运行时开关与 FPS 设置退化为无消费 UI 字段，无害）。
pub const MATERIAL_DYNAMIC_INPUT_ENABLED: bool = true;
