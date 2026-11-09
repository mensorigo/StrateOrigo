//! 定点数：值 = `raw × 10^(-scale)`。本文件只做模块声明与重新导出。
//!
//! - `Price` / `Qty` 只存整数 `raw`；精度 [`Scale`] 来自品种元数据，不随值存储。
//! - 中间结果用 `i128`，最终结果超出 `i64` 时返回 [`FixedError::Overflow`](crate::FixedError::Overflow)。
//! - 不使用浮点；解析交易所字符串时精确转换，多出的非零小数位报错而不是截断。

mod arith;
mod display;
mod parse;
mod scale;
mod types;

pub use arith::{div, div_round, mul, rescale};
pub use display::{DecimalDisplay, display};
pub use parse::parse_decimal;
pub use scale::{MAX_SCALE, Rounding, Scale};
pub use types::{Price, Qty, notional};
