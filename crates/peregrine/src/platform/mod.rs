//! 平台相关辅助模块。
//!
//! 当前仅包含 Windows 平台的 Overlay 窗口实现，其它平台为空。

#[cfg(target_os = "windows")]
pub mod windows;
