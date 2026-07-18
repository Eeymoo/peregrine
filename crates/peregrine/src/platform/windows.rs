//! Windows 平台 Overlay 覆盖层辅助函数。
//!
//! 透明、穿透、置顶由 winit 的窗口属性设置：
//! - `with_transparent(true)` → DWM 透明（softbuffer 处理 per-pixel alpha）
//! - `set_cursor_hittest(false)` → `WS_EX_TRANSPARENT | WS_EX_LAYERED`（鼠标穿透）
//! - `WindowLevel::AlwaysOnTop` → `WS_EX_TOPMOST`（置顶）
//!
//! 本模块仅补充 winit 不直接暴露的样式（`WS_EX_NOACTIVATE`、`WS_EX_TOOLWINDOW`），
//! 以及窗口枚举、目标窗口查找、矩形计算、跟随逻辑。

use std::time::Duration;
use tokio::sync::oneshot;
use windows::Win32::Foundation::{
    BOOL, GetLastError, HWND, LPARAM, POINT, RECT, SetLastError, WIN32_ERROR,
};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GWL_EXSTYLE, GWL_STYLE, GetClientRect, GetForegroundWindow, GetWindowLongPtrW,
    GetWindowRect, GetWindowTextLengthW, GetWindowTextW, HWND_NOTOPMOST, HWND_TOPMOST, IsIconic,
    IsWindow, IsWindowVisible, SW_HIDE, SW_SHOWNA, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOOWNERZORDER, SWP_NOSIZE, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    WINDOW_LONG_PTR_INDEX, WS_CAPTION, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_SYSMENU, WS_THICKFRAME,
};

/// 在 32 位 Windows 上 `SetWindowLongPtrW` 实际是 `SetWindowLongW`，参数为 `i32`；
/// 64 位上参数为 `isize`。本函数根据目标平台做统一转换，避免类型不匹配。
unsafe fn set_window_long_ptr(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    #[cfg(target_pointer_width = "64")]
    {
        unsafe { SetWindowLongPtrW(hwnd, index, value) }
    }
    #[cfg(target_pointer_width = "32")]
    {
        unsafe { SetWindowLongPtrW(hwnd, index, value as i32) as isize }
    }
}

/// 同 [`set_window_long_ptr`]，用于读取窗口样式。
unsafe fn get_window_long_ptr(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    #[cfg(target_pointer_width = "64")]
    {
        unsafe { GetWindowLongPtrW(hwnd, index) }
    }
    #[cfg(target_pointer_width = "32")]
    {
        unsafe { GetWindowLongPtrW(hwnd, index) as isize }
    }
}
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
        if set_window_long_ptr(hwnd, index, value) == 0 {
            GetLastError().ok()?;
        }
    }
    Ok(())
}

/// 将 Overlay 窗口补充设置 `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`。
///
/// 透明、穿透、置顶已由 winit 的窗口属性设置完成
///（`with_transparent`、`set_cursor_hittest`、`WindowLevel::AlwaysOnTop`）。
/// softbuffer 内部处理 per-pixel alpha 透明。
/// 本函数仅补充 winit 不直接暴露的样式。
pub fn setup_overlay_window(window: &Window) -> Result<()> {
    let hwnd = hwnd_from_window(window)?;
    unsafe {
        let ex_style = get_window_long_ptr(hwnd, GWL_EXSTYLE) as u32;
        let new_ex_style = ex_style | WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0;
        set_window_long(hwnd, GWL_EXSTYLE, new_ex_style as isize)?;

        let style = get_window_long_ptr(hwnd, GWL_STYLE) as u32;
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
    }
    Ok(())
}

/// 将 Overlay 窗口恢复为普通窗口样式。
///
/// 双窗口架构下 Overlay 窗口独立创建和销毁，不再需要恢复样式，
/// 保留此函数供后续可能的使用场景。
#[allow(dead_code)]
pub fn restore_normal_window(window: &Window) -> Result<()> {
    let hwnd = hwnd_from_window(window)?;
    unsafe {
        let ex_style = get_window_long_ptr(hwnd, GWL_EXSTYLE) as u32;
        let new_ex_style = ex_style
            & !(WS_EX_LAYERED.0
                | WS_EX_TRANSPARENT.0
                | WS_EX_TOPMOST.0
                | WS_EX_NOACTIVATE.0
                | WS_EX_TOOLWINDOW.0);
        set_window_long(hwnd, GWL_EXSTYLE, new_ex_style as isize)?;

        let style = get_window_long_ptr(hwnd, GWL_STYLE) as u32;
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
/// 匹配规则（按优先级）：
/// 1. 标题完全相等。
/// 2. 窗口标题**包含**给定的标题字符串。
/// 这样可以兼容游戏窗口标题中带动态后缀的情况（如 "GameName - Chapter 2"）。
/// 若有多个匹配项，返回第一个。
///
/// # 错误
///
/// 找不到匹配窗口时返回 [`OverlayError::TargetNotFound`]。
pub fn find_target_window(title: &str) -> Result<HWND> {
    let entries = list_window_entries();
    // 优先精确匹配。
    if let Some(e) = entries.iter().find(|e| e.title == title) {
        return Ok(e.hwnd);
    }
    // 其次模糊匹配：窗口标题包含给定字符串。
    entries
        .into_iter()
        .find(|e| e.title.contains(title))
        .map(|e| e.hwnd)
        .ok_or_else(|| OverlayError::TargetNotFound(title.to_string()))
}

/// 根据窗口标题查找目标游戏窗口的宽高比（width / height）。
///
/// 找不到窗口时返回 None。
pub fn target_window_aspect(title: &str) -> Option<f32> {
    let hwnd = find_target_window(title).ok()?;
    let rect = get_target_rect(hwnd).ok()?;
    let w = (rect.right - rect.left) as f32;
    let h = (rect.bottom - rect.top) as f32;
    if h > 0.0 { Some(w / h) } else { None }
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
        let _ = EnumWindows(
            Some(enum_window_proc),
            LPARAM(&mut state as *mut _ as isize),
        );
    }
    let titles: Vec<String> = state.entries.iter().map(|e| e.title.clone()).collect();
    tracing::info!(count = state.entries.len(), ?titles, "enumerated windows");
    state.entries
}

/// 枚举当前可见的顶层窗口标题列表（排除 Peregrine 自身与空标题）。
///
/// 供 UI 层的下拉选择控件使用，用户可以直接从列表中挑选目标窗口。
pub fn list_window_titles() -> Vec<String> {
    list_window_entries().into_iter().map(|e| e.title).collect()
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
        let style = get_window_long_ptr(target, GWL_STYLE) as u32;
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
/// - **全屏模式**（fullscreen=true）：overlay 覆盖整个屏幕，不跟随目标窗口位置，
///   但仍检测目标窗口最小化/非前台/销毁来显示/隐藏 overlay。
/// - **窗口模式**（fullscreen=false）：overlay 仅覆盖目标窗口客户区。
///   - live_drag=true：实时跟随。
///   - live_drag=false：目标窗口矩形变化期间隐藏 overlay，稳定 1200ms 后恢复。
/// - 目标窗口最小化时隐藏 Overlay（`SW_HIDE`）。
/// - 目标窗口恢复时重新显示 Overlay（`SW_SHOWNA`，不激活）。
///
/// `on_moved` 回调在每次成功调整 overlay 位置/尺寸后调用，
/// 用于通知渲染线程刷新一帧（拖拽实时显示时尤为重要：
/// 窗口仅移动而不改变尺寸时不会产生 `Resized` 事件，
/// 需要借助此回调触发重绘，避免准心位置停留在旧画面）。
///
/// 通过 `stop_rx` 可以优雅地终止轮询循环。
///
/// # 错误
///
/// Win32 API 调用失败，或接收端被丢弃时返回错误。
pub async fn follow_target_window(
    overlay: SendHwnd,
    target: SendHwnd,
    fullscreen: bool,
    live_drag: bool,
    mut stop_rx: oneshot::Receiver<()>,
    on_moved: impl FnMut() + Send + 'static,
) -> Result<()> {
    let mut last_rect = RECT::default();
    let mut visible = true;
    let mut interval = tokio::time::interval(Duration::from_millis(16));
    let mut on_moved = on_moved;

    // 全屏模式下记录上次屏幕尺寸，变化时（分辨率/DPI 缩放调整）立即更新 overlay。
    let mut last_screen_size: Option<(i32, i32)> = None;

    // 拖拽延迟：矩形变化后记录时间，稳定超过阈值才恢复显示。
    let drag_delay = Duration::from_millis(1200);
    let mut last_change_time: Option<tokio::time::Instant> = None;
    let mut dragging_hidden = false;

    loop {
        tokio::select! {
            _ = &mut stop_rx => return Err(OverlayError::Cancelled),
            _ = interval.tick() => {
                unsafe {
                    // 全屏模式：检测屏幕尺寸变化（分辨率/DPI 缩放调整），即时更新 overlay。
                    if fullscreen {
                        let screen_w = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                            windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
                        );
                        let screen_h = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                            windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN,
                        );
                        let need_update = last_screen_size != Some((screen_w, screen_h));
                        if need_update {
                            SetWindowPos(
                                overlay.0,
                                HWND_TOPMOST,
                                0,
                                0,
                                screen_w,
                                screen_h,
                                SWP_NOACTIVATE | SWP_NOOWNERZORDER,
                            )?;
                            last_screen_size = Some((screen_w, screen_h));
                            tracing::debug!(screen_w, screen_h, "update overlay to fullscreen");
                            on_moved();
                        }
                        continue;
                    }

                    // 窗口模式：检测目标窗口状态。

                    // 目标窗口已销毁/关闭：结束跟随。
                    if !IsWindow(target.0).as_bool() {
                        tracing::info!("target window no longer exists, ending follow");
                        let _ = ShowWindow(overlay.0, SW_HIDE);
                        return Err(OverlayError::Cancelled);
                    }

                    if IsIconic(target.0).as_bool() {
                        if visible {
                            let _ = ShowWindow(overlay.0, SW_HIDE);
                            visible = false;
                        }
                        continue;
                    }

                    // 目标窗口不是前台窗口时隐藏 overlay。
                    let foreground = GetForegroundWindow();
                    if foreground != target.0 {
                        if visible {
                            let _ = ShowWindow(overlay.0, SW_HIDE);
                            visible = false;
                        }
                        continue;
                    }

                    // 窗口模式：跟随目标窗口客户区。
                    let rect = match get_target_rect(target.0) {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::debug!("get_target_rect failed: {}, checking if window still exists", e);
                            if !IsWindow(target.0).as_bool() {
                                tracing::info!("target window no longer exists, ending follow");
                                let _ = ShowWindow(overlay.0, SW_HIDE);
                                return Err(OverlayError::Cancelled);
                            }
                            continue;
                        }
                    };

                    let rect_changed = !rect_eq(&rect, &last_rect);

                    if rect_changed {
                        if !live_drag {
                            // 拖拽延迟模式：矩形正在变化，隐藏 overlay。
                            if visible && !dragging_hidden {
                                let _ = ShowWindow(overlay.0, SW_HIDE);
                                dragging_hidden = true;
                                visible = false;
                            }
                            last_change_time = Some(tokio::time::Instant::now());
                            last_rect = rect;
                            continue;
                        }

                        // 实时跟随：直接更新 overlay 位置。
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
                        on_moved();
                    } else if !live_drag && dragging_hidden {
                        // 拖拽延迟模式：矩形已停止变化，检查是否超过延迟。
                        let ready = last_change_time
                            .map(|t| t.elapsed() >= drag_delay)
                            .unwrap_or(true);
                        if ready {
                            // 恢复显示。
                            let width = rect.right - rect.left;
                            let height = rect.bottom - rect.top;
                            SetWindowPos(
                                overlay.0,
                                HWND_TOPMOST,
                                rect.left,
                                rect.top,
                                width,
                                height,
                                SWP_NOACTIVATE | SWP_NOOWNERZORDER,
                            )?;
                            dragging_hidden = false;
                            visible = false; // 下面的 !visible 逻辑会重新 show
                            on_moved();
                        } else {
                            continue;
                        }
                    }

                    if !visible {
                        let _ = ShowWindow(overlay.0, SW_SHOWNA);
                        visible = true;
                    }
                }
            }
        }
    }
}

// ===== 动态输入采集（Step 10） =====

/// Win32 虚拟键码 → Peregrine 按键字符串的映射表。
///
/// 仅映射物料脚本可能关心的常用按键；未列出的按键不会出现在 key_state 中。
const VK_MAP: &[(i32, &str)] = &[
    // 字母键 A-Z（VK 0x41-0x5A）。
    (0x41, "a"),
    (0x42, "b"),
    (0x43, "c"),
    (0x44, "d"),
    (0x45, "e"),
    (0x46, "f"),
    (0x47, "g"),
    (0x48, "h"),
    (0x49, "i"),
    (0x4A, "j"),
    (0x4B, "k"),
    (0x4C, "l"),
    (0x4D, "m"),
    (0x4E, "n"),
    (0x4F, "o"),
    (0x50, "p"),
    (0x51, "q"),
    (0x52, "r"),
    (0x53, "s"),
    (0x54, "t"),
    (0x55, "u"),
    (0x56, "v"),
    (0x57, "w"),
    (0x58, "x"),
    (0x59, "y"),
    (0x5A, "z"),
    // 数字键 0-9（VK 0x30-0x39）。
    (0x30, "0"),
    (0x31, "1"),
    (0x32, "2"),
    (0x33, "3"),
    (0x34, "4"),
    (0x35, "5"),
    (0x36, "6"),
    (0x37, "7"),
    (0x38, "8"),
    (0x39, "9"),
    // 修饰键。
    (0x10, "shift"),
    (0x11, "ctrl"),
    (0x12, "alt"),
    (0x5B, "super"),
    (0x5C, "super"),
    // 方向键。
    (0x25, "left"),
    (0x26, "up"),
    (0x27, "right"),
    (0x28, "down"),
    // 功能键 F1-F12。
    (0x70, "f1"),
    (0x71, "f2"),
    (0x72, "f3"),
    (0x73, "f4"),
    (0x74, "f5"),
    (0x75, "f6"),
    (0x76, "f7"),
    (0x77, "f8"),
    (0x78, "f9"),
    (0x79, "f10"),
    (0x7A, "f11"),
    (0x7B, "f12"),
    // 特殊键。
    (0x20, "space"),
    (0x0D, "enter"),
    (0x1B, "escape"),
    (0x09, "tab"),
];

/// 读取当前动态输入上下文（Windows 实现）。
///
/// - 鼠标位置：`GetCursorPos`（屏幕坐标）。
/// - 键盘状态：`GetAsyncKeyState` 查询 VK_MAP 中的按键。
/// - 时间：自进程启动以来的毫秒数。
///
/// 日志隐私：仅记录按键数量，不记录具体按键代码。
pub fn poll_dynamic_context(_screen_w: f32, _screen_h: f32) -> peregrine_material::DynamicContext {
    use std::time::{SystemTime, UNIX_EPOCH};

    // 鼠标位置。
    let mut point = POINT { x: 0, y: 0 };
    let mouse_pos = unsafe {
        if GetCursorPos(&mut point).is_ok() {
            (point.x as f32, point.y as f32)
        } else {
            (0.0, 0.0)
        }
    };

    // 键盘状态：查询 VK_MAP 中每个键，按下的加入 key_state。
    let mut key_state = peregrine_material::KeyState::new();
    let mut pressed_count = 0u32;
    for (vk, name) in VK_MAP {
        unsafe {
            let state = GetAsyncKeyState(*vk);
            // GetAsyncKeyState 返回 i16，最高位为 1 表示当前按下。
            if (state as u16) & 0x8000 != 0 {
                key_state.press(name);
                pressed_count += 1;
            }
        }
    }

    // 时间戳（毫秒）。
    let time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    tracing::trace!(
        pressed_count,
        mouse_x = mouse_pos.0,
        mouse_y = mouse_pos.1,
        "polled dynamic context"
    );

    peregrine_material::DynamicContext {
        time_ms,
        mouse_pos,
        key_state,
        // 种子：基于时间戳 + 一次计数器，确保每帧不同。
        rng_seed: time_ms.wrapping_add(frame_counter()),
        version: time_ms,
    }
}

/// 简单的帧计数器（用于派生 RNG 种子）。
fn frame_counter() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
