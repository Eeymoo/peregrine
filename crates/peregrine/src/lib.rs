//! Peregrine 可复用的核心库。
//!
//! 为 Tauri 入口提供 overlay 渲染、平台 API、几何形状等模块。

pub mod overlay_renderer;
pub mod platform;
pub mod shapes;
pub mod svg_renderer;

/// 物料运行时（layers + Rhai）软关闭开关。
///
/// 当前为 `false`：overlay 渲染强制走旧版 `Crosshair` 路径，忽略 `Profile.layers`，
/// 物料求值、动态输入轮询与动态重绘调度均不进入运行链路。
/// 物料相关代码与测试完整保留；将来修复后改回 `true` 即可整体恢复。
pub const MATERIAL_RUNTIME_ENABLED: bool = false;
