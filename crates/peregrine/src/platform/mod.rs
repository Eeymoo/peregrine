//! 平台相关辅助模块。
//!
//! 仅 Windows 提供完整的 Overlay 窗口实现（透明穿透置顶）；
//! 其它平台编译为占位模块，保证整个工程可跨平台编译。

#[cfg(windows)]
pub mod windows;
