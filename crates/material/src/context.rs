//! 动态输入上下文。
//!
//! `DynamicContext` 聚合物料脚本可访问的全部动态输入（时间、鼠标位置、键盘状态、
//! 随机数种子），在每次物料求值时显式传入。host function（`time_ms` / `mouse_pos`
//! / `key_down` / `rand`）通过 closure 捕获 `DynamicContext` 的不可变引用实现。
//!
//! 设计目标：
//! - 物料求值无全局可变状态；同一输入产生同一输出（除了显式声明的动态输入）。
//! - 动态输入集可扩展：新增输入源只需在本结构体新增字段 + 新注册一个 host function。
//! - `version` 字段用于缓存失效：静态物料永远 version=0；动态物料每帧递增。

use std::collections::HashSet;

/// 物料求值时的动态输入快照。
#[derive(Debug, Clone)]
pub struct DynamicContext {
    /// 自进程启动以来的毫秒数，单调递增。
    pub time_ms: u64,
    /// 鼠标当前位置（逻辑屏幕坐标）。
    pub mouse_pos: (f32, f32),
    /// 当前按下的按键集合（小写字符串，如 `"shift"` / `"a"` / `"f1"`）。
    pub key_state: KeyState,
    /// 随机数种子，派生自 `(material_id, params_hash, frame_count)`。
    pub rng_seed: u64,
    /// 动态上下文版本号，用于物料缓存失效。
    ///
    /// 静态物料（`is_dynamic = false`）永远为 0；动态物料每帧递增。
    pub version: u64,
}

impl Default for DynamicContext {
    fn default() -> Self {
        Self {
            time_ms: 0,
            mouse_pos: (0.0, 0.0),
            key_state: KeyState::default(),
            rng_seed: 1,
            version: 0,
        }
    }
}

impl DynamicContext {
    /// 创建一个用于静态物料求值的上下文（所有动态输入为默认值，version=0）。
    ///
    /// 静态物料不调用任何动态输入 host function，因此上下文内容无关紧要，
    /// 但必须保证 version 固定为 0 以启用永久缓存。
    pub fn static_context() -> Self {
        Self::default()
    }

    /// 创建一个用于预览的快照上下文（使用调用瞬间的真实时间，鼠标居中，无按键）。
    pub fn preview_snapshot(screen_w: f32, screen_h: f32) -> Self {
        Self {
            time_ms: current_time_ms(),
            mouse_pos: (screen_w / 2.0, screen_h / 2.0),
            key_state: KeyState::default(),
            rng_seed: current_time_ms(),
            version: current_time_ms(),
        }
    }
}

/// 按键状态表，记录当前被按下的按键。
///
/// 按键代码统一为小写字符串：
/// - 字母键：`"a"` 到 `"z"`
/// - 数字键：`"0"` 到 `"9"`
/// - 修饰键：`"shift"` / `"ctrl"` / `"alt"` / `"super"`
/// - 方向键：`"up"` / `"down"` / `"left"` / `"right"`
/// - 功能键：`"f1"` 到 `"f12"`
/// - 特殊键：`"space"` / `"enter"` / `"escape"` / `"tab"`
#[derive(Debug, Clone, Default)]
pub struct KeyState {
    pressed: HashSet<String>,
}

impl KeyState {
    /// 创建空的按键状态表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 查询指定按键是否被按下（按键代码大小写不敏感，内部统一转小写）。
    pub fn is_down(&self, code: &str) -> bool {
        self.pressed.contains(&code.to_lowercase())
    }

    /// 标记某键被按下。
    pub fn press(&mut self, code: &str) {
        self.pressed.insert(code.to_lowercase());
    }

    /// 标记某键被松开。
    pub fn release(&mut self, code: &str) {
        self.pressed.remove(&code.to_lowercase());
    }

    /// 清空所有按键状态。
    pub fn clear(&mut self) {
        self.pressed.clear();
    }

    /// 返回当前按下的按键数量（聚合统计，可用于日志，不暴露具体按键）。
    pub fn pressed_count(&self) -> usize {
        self.pressed.len()
    }
}

/// 获取自进程启动以来的毫秒数。
fn current_time_ms() -> u64 {
    use std::sync::OnceLock;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    static START_UNIX: OnceLock<u64> = OnceLock::new();
    let start_unix = *START_UNIX.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    });

    static START_INSTANT: OnceLock<Instant> = OnceLock::new();
    let start_instant = START_INSTANT.get_or_init(Instant::now);

    // 优先用 Instant（单调），起点对齐到 UNIX 时间戳，便于物料脚本使用。
    start_unix.wrapping_add(start_instant.elapsed().as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_state_case_insensitive() {
        let mut s = KeyState::new();
        s.press("Shift");
        assert!(s.is_down("shift"));
        assert!(s.is_down("SHIFT"));
        s.release("SHIFT");
        assert!(!s.is_down("shift"));
    }

    #[test]
    fn key_state_count() {
        let mut s = KeyState::new();
        assert_eq!(s.pressed_count(), 0);
        s.press("a");
        s.press("b");
        s.press("ctrl");
        assert_eq!(s.pressed_count(), 3);
    }

    #[test]
    fn static_context_has_zero_version() {
        let ctx = DynamicContext::static_context();
        assert_eq!(ctx.version, 0);
    }

    #[test]
    fn preview_snapshot_has_nonzero_version() {
        let ctx = DynamicContext::preview_snapshot(1920.0, 1080.0);
        assert!(ctx.version > 0 || ctx.time_ms > 0 || ctx.rng_seed > 0);
        assert_eq!(ctx.mouse_pos, (960.0, 540.0));
    }
}
