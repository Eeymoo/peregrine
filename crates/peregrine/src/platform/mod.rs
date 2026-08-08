//! 平台相关辅助模块。
//!
//! 仅 Windows 提供完整的 Overlay 窗口实现（透明穿透置顶）；
//! 其它平台编译为占位模块，保证整个工程可跨平台编译。

#[cfg(windows)]
pub mod windows;

/// 获取当前动态输入上下文（鼠标位置、键盘状态、时间）。
///
/// Windows 平台通过 Win32 API 实时采集；其它平台返回安全默认值。
#[cfg(windows)]
pub fn poll_dynamic_context(screen_w: f32, screen_h: f32) -> peregrine_material::DynamicContext {
    windows::poll_dynamic_context(screen_w, screen_h)
}

/// 非 Windows 平台：返回默认上下文（鼠标居中，无按键，预览时间）。
#[cfg(not(windows))]
pub fn poll_dynamic_context(screen_w: f32, screen_h: f32) -> peregrine_material::DynamicContext {
    peregrine_material::DynamicContext::preview_snapshot(screen_w, screen_h)
}
