//! 物料运行时错误类型。

use thiserror::Error;

/// 物料相关操作的统一错误类型。
#[derive(Debug, Error)]
pub enum MaterialError {
    /// Rhai 脚本解析错误（语法错误）。
    #[error("material '{id}' parse error: {source}")]
    Parse {
        id: String,
        #[source]
        source: rhai::ParseError,
    },

    /// 物料脚本缺少必需的导出函数（`build` / `defaults` / `schema`）。
    #[error("material '{id}' missing required function: {function}")]
    MissingFunction { id: String, function: &'static str },

    /// 物料求值时 Rhai 运行时错误（异常、类型错误等）。
    #[error("material '{id}' evaluation error: {message}")]
    Evaluation { id: String, message: String },

    /// 物料求值超出操作数限制（死循环保护）。
    #[error("material '{id}' exceeded operations limit")]
    OperationsLimitExceeded { id: String },

    /// 物料返回值类型不匹配（不是 Array 或 Array 元素非 map）。
    #[error("material '{id}' returned invalid type: {detail}")]
    InvalidReturnType { id: String, detail: String },

    /// Element 中包含未知的图元类型。
    #[error("material '{id}' returned unknown element type: {element_type}")]
    UnknownElementType {
        id: String,
        element_type: String,
    },

    /// Element 字段缺失或类型错误。
    #[error("material '{id}' element field error: {detail}")]
    ElementField { id: String, detail: String },

    /// 物料 id 在注册表中不存在。
    #[error("material not found: {0}")]
    NotFound(String),

    /// 用户物料文件 IO 错误。
    #[error("io error loading material '{name}': {source}")]
    Io {
        name: String,
        #[source]
        source: std::io::Error,
    },

    /// 物料目录扫描失败。
    #[error("failed to scan material directory {dir}: {source}")]
    DirScan {
        dir: String,
        #[source]
        source: std::io::Error,
    },
}

/// 物料模块统一 Result 别名。
pub type MaterialResult<T> = std::result::Result<T, MaterialError>;
