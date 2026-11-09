//! StrateOrigo 基础类型（入口文件：只做模块声明与重新导出）。
//!
//! 本 crate 属于**热路径**（docs/04 §1）：不分配堆内存、不加锁、不阻塞、不 panic、
//! 不读系统时间、价格与数量不用浮点。下面的 clippy 规则在非测试构建中强制执行。
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::float_arithmetic,
    )
)]

pub mod clock;
pub mod error;
pub mod fixed;
pub mod ids;
pub mod mode;
pub mod time;

pub use clock::{Clock, ManualClock};
pub use error::FixedError;
pub use fixed::{Price, Qty, Rounding, Scale};
pub use ids::{AccountId, InstrumentId, StrategyId, VenueId};
pub use mode::{AccountKind, ParseRunModeError, RunMode};
pub use time::UnixNanos;
