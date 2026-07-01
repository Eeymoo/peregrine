//! Windows 平台 Overlay 覆盖层辅助函数。
//!
//! 通过 `windows-rs` 调用 Win32 API，将 `winit` 创建的普通窗口改造为：
//! - 透明（`WS_EX_LAYERED` + `SetLayeredWindowAttributes`，纯黑色作为透明色）
//! - 置顶（`WS_EX_TOPMOST`）
//! - 鼠标穿透（`WS_EX_TRANSPARENT`）
//! - 不获取焦点（`WS_EX_NOACTIVATE`）
//! - 不在任务栏显示按钮（`WS_EX_TOOLWINDOW`）
//!
//! 同时提供顶层窗口枚举、目标游戏窗口查找、矩形计算、以及 16ms 轮询跟随逻辑。
//! 本模块所有公开函数均返回 [`Result`] 或不 panic。

use std::time::Duration;
use tokio::sync::oneshot;
use windows::Win32::Foundation::{
    BOOL, COLORREF, GetLastError, HWND, LPARAM, POINT, RECT, SetLastError, WIN32_ERROR,
};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GWL_EXSTYLE, GWL_STYLE, GetClientRect, GetWindowLongPtrW, GetWindowRect,
    GetWindowTextLengthW, GetWindowTextW, HWND_NOTOPMOST, HWND_TOPMOST, IsIconic, IsWindowVisible,
    LWA_COLORKEY, SW_HIDE, SW_SHOWNA, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOOWNERZORDER, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    WINDOW_LONG_PTR_INDEX, WS_CAPTION, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_SYSMENU, WS_THICKFRAME,
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

/// Overlay 平台错误类型。
#[derive(Debug, thiserror::Error)]
pub enum OverlayError {
    /// 无法从 `winit` 窗口获取原生句柄。
    #[error("无法获取窗口句柄")]
    NoWindowHandle,
    /// 获取到的原生句柄不是 Win32 `HWND`。
    #[error("平台句柄不是 Win32 HWND")]
    NotWin32Hwnd,
    /// Win32 API 调用失败。
    #[error("Win32 API 调用失败: {0}")]
    Win32(#[from] windows::core::Error),
    /// 找不到标题匹配的目标窗口。
    #[error("目标窗口未找到: {0}")]
    TargetNotFound(String),
    /// 跟随任务被外部取消。
    #[error("跟随任务被取消")]
    Cancelled,
}

/// 本模块统一返回类型。
pub type Result<T> = std::result::Result<T, OverlayError>;

/// 从 `winit` 窗口获取对应的 Win32 `HWND`。
///
/// # 错误
///
/// 当窗口未提供原生句柄，或不是 Win32 句柄时返回错误。
pub fn hwnd_from_window(window: &Window) -> Result<HWND> {
    let handle = window
        .window_handle()
        .map_err(|_| OverlayError::NoWindowHandle)?;
    match handle.as_raw() {
        RawWindowHandle::Win32(h) => Ok(HWND(h.hwnd.get() as *mut std::ffi::c_void)),
        _ => Err(OverlayError::NotWin32Hwnd),
    }
}

/// 安全地调用 `SetWindowLongPtrW` 并检查 `GetLastError`。
///
/// `SetWindowLongPtrW` 失败时返回 0，但不会直接返回 `Result`，
/// 因此需要手动清空并检查 last error。
fn set_window_long(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> Result<()> {
    unsafe {
        SetLastError(WIN32_ERROR(0));
        if SetWindowLongPtrW(hwnd, index, value) == 0 {
            GetLastError().ok()?;
        }
    }
    Ok(())
}

/// 将 `winit` 窗口改造为透明置顶穿透的 Overlay 窗口。
///
/// 具体行为：
/// - 扩展样式增加 `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST |
///   WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`。
/// - 移除 `WS_CAPTION | WS_THICKFRAME | WS_SYSMENU` 等边框样式。
/// - 使用 `SetLayeredWindowAttributes` 将纯黑色（`RGB 0,0,0`）设为透明色。
///
/// # 注意
///
/// 调用方需要保证渲染时把准心外区域清为纯黑，颜色键透明才会生效。
pub fn setup_overlay_window(window: &Window) -> Result<()> {
    let hwnd = hwnd_from_window(window)?;
    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let new_ex_style = ex_style
            | WS_EX_LAYERED.0
            | WS_EX_TRANSPARENT.0
            | WS_EX_TOPMOST.0
            | WS_EX_NOACTIVATE.0
            | WS_EX_TOOLWINDOW.0;
        set_window_long(hwnd, GWL_EXSTYLE, new_ex_style as isize)?;

        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let new_style = style & !(WS_CAPTION.0 | WS_THICKFRAME.0 | WS_SYSMENU.0);
        set_window_long(hwnd, GWL_STYLE, new_style as isize)?;

        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )?;

        SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_COLORKEY)?;
    }
    Ok(())
}

/// 将 Overlay 窗口恢复为普通窗口样式，用于进入设置界面时撤销覆盖层效果。
///
/// 具体行为：
/// - 移除 `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST |
///   WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`。
/// - 恢复 `WS_CAPTION | WS_THICKFRAME | WS_SYSMENU` 边框样式。
/// - 取消置顶。
pub fn restore_normal_window(window: &Window) -> Result<()> {
    let hwnd = hwnd_from_window(window)?;
    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let new_ex_style = ex_style
            & !(WS_EX_LAYERED.0
                | WS_EX_TRANSPARENT.0
                | WS_EX_TOPMOST.0
                | WS_EX_NOACTIVATE.0
                | WS_EX_TOOLWINDOW.0);
        set_window_long(hwnd, GWL_EXSTYLE, new_ex_style as isize)?;

        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let new_style = style | WS_CAPTION.0 | WS_THICKFRAME.0 | WS_SYSMENU.0;
        set_window_long(hwnd, GWL_STYLE, new_style as isize)?;

        SetWindowPos(
            hwnd,
            HWND_NOTOPMOST,
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )?;
    }
    Ok(())
}

/// 根据窗口标题查找目标游戏窗口。
///
/// 使用与「选择窗口」按钮相同的枚举逻辑，保证选中后能再次定位到同一窗口。
///
/// # 错误
///
/// 找不到匹配窗口时返回 [`OverlayError::TargetNotFound`]。
pub fn find_target_window(title: &str) -> Result<HWND> {
    let entries = list_window_entries();
    entries
        .into_iter()
        .find(|e| e.title == title)
        .map(|e| e.hwnd)
        .ok_or_else(|| OverlayError::TargetNotFound(title.to_string()))
}

/// 一个可见顶层窗口的句柄与标题。
#[derive(Debug, Clone)]
pub struct WindowEntry {
    /// 窗口句柄。
    pub hwnd: HWND,
    /// 窗口标题。
    pub title: String,
}

/// 枚举当前可见且有标题的顶层窗口（排除自身 Peregrine 与空标题）。
///
/// 用于「选择窗口」按钮循环切换目标窗口，以及按标题查找 HWND。
unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let state = lparam.0 as *mut EnumWindowsState;
    if state.is_null() {
        return BOOL(1);
    }

    // 跳过不可见窗口。
    let visible = unsafe { IsWindowVisible(hwnd).as_bool() };
    if !visible {
        return BOOL(1);
    }

    // 跳过自身窗口。
    let len = unsafe { GetWindowTextLengthW(hwnd) as usize };
    if len == 0 {
        return BOOL(1);
    }
    let mut buf = vec![0u16; len + 1];
    let got = unsafe { GetWindowTextW(hwnd, &mut buf) as usize };
    if got == 0 {
        return BOOL(1);
    }
    let title = String::from_utf16_lossy(&buf[..got.min(len)]);

    unsafe {
        if title == (*state).self_title {
            return BOOL(1);
        }
        (*state).entries.push(WindowEntry { hwnd, title });
    }
    BOOL(1)
}

#[derive(Debug, Default)]
struct EnumWindowsState {
    /// 自身窗口标题，用于排除。
    self_title: String,
    /// 收集到的顶层窗口。
    entries: Vec<WindowEntry>,
}

/// 枚举当前可见的顶层窗口（排除 Peregrine 自身与空标题）。
///
/// 返回按 `EnumWindows` 遍历顺序的窗口列表。
pub fn list_window_entries() -> Vec<WindowEntry> {
    let mut state = EnumWindowsState {
        self_title: "Peregrine".to_string(),
        entries: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(Some(enum_window_proc), LPARAM(&mut state as *mut _ as isize));
    }
    let titles: Vec<String> = state.entries.iter().map(|e| e.title.clone()).collect();
    tracing::info!(count = state.entries.len(), ?titles, "enumerated windows");
    state.entries
}

/// 返回 `current` 标题在窗口列表中的下一个窗口信息（循环）。
///
/// 逻辑：
/// - 列表为空：返回 `None`。
/// - 能找到 `current`：返回它的下一个；若已是最后一个则回到第一个。
/// - 找不到 `current`：返回列表第一个，避免目标窗口标题变化后按钮彻底失效。
pub fn next_window_entry(current: &str) -> Option<WindowEntry> {
    let entries = list_window_entries();
    tracing::info!(current, count = entries.len(), "select next window");
    if entries.is_empty() {
        return None;
    }
    let result = entries
        .iter()
        .position(|e| e.title == current)
        .map(|idx| entries[(idx + 1) % entries.len()].clone())
        .or_else(|| entries.first().cloned());
    if let Some(ref e) = result {
        tracing::info!(next = %e.title, "selected next window");
    }
    result
}

/// 返回 `current` 标题在窗口列表中的下一个窗口标题（循环）。
///
/// 等价于 [`next_window_entry`] 只取标题，供 UI 层使用。
pub fn next_window_title(current: &str) -> Option<String> {
    next_window_entry(current).map(|e| e.title)
}
#[inline]
fn rect_eq(a: &RECT, b: &RECT) -> bool {
    a.left == b.left && a.top == b.top && a.right == b.right && a.bottom == b.bottom
}

/// 计算目标游戏窗口在屏幕上的覆盖矩形。
///
/// - 若目标窗口没有标题栏（`WS_CAPTION`），认为是无边框窗口化，
///   返回 `GetWindowRect` 全矩形。
/// - 否则为普通窗口化，返回 `GetClientRect` 客户区，
///   并通过 `ClientToScreen` 转换为屏幕坐标。
///
/// 这样可以保证窗口化模式下，Overlay 准心对准游戏画面中心，
/// 而不是包含标题栏的整个窗口。
pub fn get_target_rect(target: HWND) -> Result<RECT> {
    unsafe {
        let style = GetWindowLongPtrW(target, GWL_STYLE) as u32;
        if style & WS_CAPTION.0 == 0 {
            // 无边框窗口化：使用整个窗口矩形。
            let mut rect = RECT::default();
            GetWindowRect(target, &mut rect)?;
            Ok(rect)
        } else {
            // 窗口化：使用客户区并转换为屏幕坐标。
            let mut rect = RECT::default();
            GetClientRect(target, &mut rect)?;

            let mut top_left = POINT {
                x: rect.left,
                y: rect.top,
            };
            ClientToScreen(target, &mut top_left).ok()?;

            let mut bottom_right = POINT {
                x: rect.right,
                y: rect.bottom,
            };
            ClientToScreen(target, &mut bottom_right).ok()?;

            Ok(RECT {
                left: top_left.x,
                top: top_left.y,
                right: bottom_right.x,
                bottom: bottom_right.y,
            })
        }
    }
}

/// 可以安全跨线程传递的 `HWND` 包装。
///
/// `HWND` 内部是原始指针，默认未实现 `Send`。窗口句柄本身没有所有权，
/// 跨线程只读使用是安全的，因此用此包装放入 tokio 后台任务。
#[derive(Clone, Copy)]
pub struct SendHwnd(pub HWND);

unsafe impl Send for SendHwnd {}

/// 以 16ms 为周期轮询目标窗口，同步 Overlay 窗口的位置与大小。
///
/// 行为：
/// - 目标窗口最小化时隐藏 Overlay（`SW_HIDE`）。
/// - 目标窗口恢复时重新显示 Overlay（`SW_SHOWNA`，不激活）。
/// - 目标窗口矩形发生变化时，调用 `SetWindowPos` 将 Overlay 移动到对应屏幕位置
///   并调整为相同大小。
///
/// 通过 `stop_rx` 可以优雅地终止轮询循环。
///
/// # 错误
///
/// Win32 API 调用失败，或接收端被丢弃时返回错误。
pub async fn follow_target_window(
    overlay: SendHwnd,
    target: SendHwnd,
    mut stop_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let mut last_rect = RECT::default();
    let mut visible = true;
    let mut interval = tokio::time::interval(Duration::from_millis(16));

    loop {
        tokio::select! {
            _ = &mut stop_rx => return Err(OverlayError::Cancelled),
            _ = interval.tick() => {
                unsafe {
                    if IsIconic(target.0).as_bool() {
                        if visible {
                            ShowWindow(overlay.0, SW_HIDE).ok()?;
                            visible = false;
                        }
                        continue;
                    }

                    let rect = get_target_rect(target.0)?;
                    if !rect_eq(&rect, &last_rect) {
                        let width = rect.right - rect.left;
                        let height = rect.bottom - rect.top;
                        tracing::debug!(
                            left = rect.left,
                            top = rect.top,
                            width,
                            height,
                            "follow target window"
                        );
                        SetWindowPos(
                            overlay.0,
                            HWND_TOPMOST,
                            rect.left,
                            rect.top,
                            width,
                            height,
                            SWP_NOACTIVATE | SWP_NOOWNERZORDER,
                        )?;
                        last_rect = rect;
                    }

                    if !visible {
                        ShowWindow(overlay.0, SW_SHOWNA).ok()?;
                        visible = true;
                    }
                }
            }
        }
    }
}
