//! 事件信封：行情、执行、定时器、控制四类共用 [`Event`]（docs/03 §3）。

/// 进入事件内核的统一事件。
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    /// 公共行情（不含账户信息）。
    Market(crate::market::MarketEvent),
    /// 执行回报（M6 so-oms 接入；先占位以固定事件排序规则 docs/04 D3）。
    Exec(ExecEvent),
    /// 定时器到期。
    Timer(TimerEvent),
    /// 控制命令（暂停 / 只减仓 / 熔断等）。
    Control(ControlEvent),
}

/// 执行回报（占位：M3 so-oms 实现完整状态机时展开字段）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExecEvent {
    /// 订单被交易所接受。
    Accepted,
    /// 订单被拒绝。
    Rejected,
    /// 成交。
    Filled,
    /// 撤单确认。
    Canceled,
}

/// 定时器事件（策略经 `Ctx::set_timer` 注册）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TimerEvent {
    /// 注册时提供的标签，用于区分同一策略的多个定时器。
    pub tag: u32,
}

/// 控制命令（L2 组合风控与 gRPC 控制面下发，docs/03 §7）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ControlEvent {
    /// 只减仓。
    ReduceOnly,
    /// 熔断：撤单并停止一切新单。
    Halt,
    /// 撤销全部挂单。
    CancelAll,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_envelope_constructs() {
        let _ = Event::Timer(TimerEvent { tag: 7 });
        let _ = Event::Control(ControlEvent::Halt);
    }
}
