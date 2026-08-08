//! Peregrine 物料运行时。
//!
//! 职责：加载 Rhai 脚本形式的物料（Material），并提供统一的求值接口。
//!
//! 物料是"参数 + 屏幕区域 → Element 列表"的纯映射，由 Rhai 脚本定义。
//! 本 crate 不依赖任何 UI / GPU / window 平台代码，仅依赖 `peregrine_config`
//! 的数据类型与 `rhai` 脚本引擎。
//!
//! 架构边界：
//! - 输入：`params: serde_json::Value`、`screen: peregrine_config::Rect`、`DynamicContext`
//! - 输出：`Vec<peregrine_config::Element>`
//! - 不持有任何窗口 / 渲染资源，可在任意线程调用。
//!
//! 物料脚本必须导出三个顶层函数：
//! - `fn defaults() -> Map`：返回默认参数
//! - `fn schema() -> Array`：返回参数元数据（供 UI 自动生成控件）
//! - `fn build(params, screen) -> Array`：根据参数和屏幕区域返回 Element 列表

pub mod context;
pub mod error;
pub mod material;
pub mod registry;

pub use context::{DynamicContext, KeyState};
pub use error::{MaterialError, MaterialResult};
pub use material::{Material, MaterialInfo, MaterialMetadata};
pub use registry::MaterialRegistry;

/// 内置物料 id 前缀。
pub const BUILTIN_PREFIX: &str = "builtin.";
/// 用户物料 id 前缀。
pub const USER_PREFIX: &str = "user.";

/// 内置物料源码通过 `include_str!` 嵌入二进制。
///
/// key = 物料 id（不含 `builtin.` 前缀），value = Rhai 源码。
pub const BUILTIN_MATERIALS: &[(&str, &str)] = &[
    ("cross", include_str!("../builtin/cross.rhai")),
    ("large_cross", include_str!("../builtin/large_cross.rhai")),
    ("edge_rect", include_str!("../builtin/edge_rect.rhai")),
    ("corner_dots", include_str!("../builtin/corner_dots.rhai")),
    ("ring", include_str!("../builtin/ring.rhai")),
    ("custom_orb", include_str!("../builtin/custom_orb.rhai")),
    ("random_orb", include_str!("../builtin/random_orb.rhai")),
    ("border_frame", include_str!("../builtin/border_frame.rhai")),
    ("edge_arrows", include_str!("../builtin/edge_arrows.rhai")),
    ("grid", include_str!("../builtin/grid.rhai")),
    ("image", include_str!("../builtin/image.rhai")),
    ("time", include_str!("../builtin/time.rhai")),
];
