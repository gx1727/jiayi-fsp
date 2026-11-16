//! 流控 —— 规范 `11-flow-control.md`。
//!
//! ## 两个层次必须分清
//!
//! | 参数 | 归属 | 说明 |
//! |------|------|------|
//! | `window` | 协议层 | **在途上限**，Server 下发，**Client 不得擅自修改** |
//! | `burst_size` / `batch_gap_us` | 发送端本地 | AIMD 调节，不改协议语义 |
//!
//! v1.0.0 的 AIMD 直接收缩协议层 `window`，与「任何流控参数下协议行为都应一致」
//! 冲突 —— 窗口收缩会改变在途上限，进而改变发送方的阻塞行为。

/// 裁决周期（毫秒）—— 接收方**必须**至少每 50ms 下发一次缺失裁决（11 章）。
pub const VERDICT_PERIOD_MS: u64 = 50;

/// 裁决超时（毫秒）—— 发送方超过此时长未收到裁决即主动索要（11 章）。
pub const VERDICT_TIMEOUT_MS: u64 = 1000;

/// `mode=request` 的应答时限（毫秒）（11 章）。
pub const VERDICT_REPLY_MS: u64 = 50;

/// 单条 `missing_chunks` 载荷的条目上限（18 章）。
pub const MISSING_LIMIT: usize = 4096;

/// `batch_gap_us` 的上界。
pub const MAX_BATCH_GAP_US: u64 = 100_000;

/// 拥塞阈值。
pub const LOSS_CONGESTED: f64 = 0.05;
/// 通畅阈值。
pub const LOSS_CLEAR: f64 = 0.01;
/// 最小样本量：低于此值不得据此调整（11 章 AIMD 修正）。
pub const MIN_SAMPLES: u64 = 16;
/// 滑动窗口长度（tick 数）。
pub const LOSS_WINDOW_TICKS: usize = 20;

/// 丢包率估计器（11 章）。
///
/// v1.0.0 用单 tick 差值估算，在小窗口下会被量化噪声主导 ——
/// 每 tick 仅发送 16 块时丢 1 块即为 6.25%，直接越过 5% 拥塞阈值并触发
/// 窗口收缩，形成正反馈。本实现改用滑动窗口累计值 + 最小样本量 + 禁除零。
#[derive(Debug, Clone)]
pub struct LossEstimator {
    window: Vec<(u64, u64)>, // (received, sent)
    max_ticks: usize,
}

impl Default for LossEstimator {
    fn default() -> Self {
        LossEstimator { window: Vec::new(), max_ticks: LOSS_WINDOW_TICKS }
    }
}

impl LossEstimator {
    /// 记录一次采样。
    pub fn push(&mut self, received: u64, sent: u64) {
        self.window.push((received, sent));
        if self.window.len() > self.max_ticks {
            self.window.remove(0);
        }
    }

    /// 累计样本量。
    pub fn samples(&self) -> u64 {
        self.window.iter().map(|(_, sent)| *sent).sum()
    }

    /// 估算丢包率；样本不足或分母为 0 时返回 `None`。
    pub fn loss_rate(&self) -> Option<f64> {
        let sent: u64 = self.window.iter().map(|(_, s)| *s).sum();
        let received: u64 = self.window.iter().map(|(r, _)| *r).sum();
        if sent < MIN_SAMPLES || sent == 0 {
            return None;
        }
        Some(1.0 - (received as f64 / sent as f64))
    }
}

/// 发送端流控控制器。
#[derive(Debug, Clone)]
pub struct FlowController {
    /// 协议层在途上限（Server 下发，**不得**修改）。
    pub window: u32,
    /// 发送端本地：每批连续发送的 chunk 数。
    pub burst_size: u32,
    /// 发送端本地：批间隔（微秒）。
    pub batch_gap_us: u64,
    /// 已发送的 chunk 总数。
    pub sent_total: u64,
    /// 对端已确认接收的 chunk 总数（来自最近一次裁决）。
    pub acked_total: u64,
    /// 丢包率估计器。
    pub loss: LossEstimator,
}

impl FlowController {
    /// 新建控制器：`burst_size` 初值 = `window`，`batch_gap_us` 初值 = 2000。
    pub fn new(window: u32) -> Self {
        Self {
            window,
            burst_size: window,
            batch_gap_us: 2000,
            sent_total: 0,
            acked_total: 0,
            loss: LossEstimator::default(),
        }
    }

    /// 在途量估计 = 已发送 − 对端已确认（11 章窗口推进判据第 3 条）。
    pub fn in_flight(&self) -> u64 {
        self.sent_total.saturating_sub(self.acked_total)
    }

    /// 在途量是否已达上限（达到则**必须**暂停发送新数据）。
    pub fn should_pause(&self) -> bool {
        self.in_flight() >= u64::from(self.window)
    }

    /// 记录一次发送。
    pub fn on_sent(&mut self, count: u64) {
        self.sent_total += count;
    }

    /// 记录一次裁决：`acked` 是对端已接收的 chunk 总数。
    pub fn on_verdict(&mut self, acked: u64, sent_in_interval: u64, received_in_interval: u64) {
        self.acked_total = acked;
        self.loss.push(received_in_interval, sent_in_interval);
        self.apply_aimd();
    }

    /// AIMD 调整（11 章）。只调整**发送端本地**参数，不动协议层 `window`。
    pub fn apply_aimd(&mut self) {
        let Some(loss) = self.loss.loss_rate() else {
            return; // 样本不足：不调整
        };
        if loss > LOSS_CONGESTED {
            self.batch_gap_us = self.batch_gap_us.saturating_mul(3) / 2;
            self.batch_gap_us = self.batch_gap_us.clamp(200, MAX_BATCH_GAP_US);
            let floor = u64::from(self.window.min(64));
            self.burst_size = ((u64::from(self.burst_size) * 7 / 10) as u32).max(floor as u32);
        } else if loss < LOSS_CLEAR {
            self.batch_gap_us = self.batch_gap_us.saturating_sub(10);
            let ceiling = self.window;
            self.burst_size = self.burst_size.saturating_add(2).min(ceiling);
        }
        // 中间区间（1% ≤ loss ≤ 5%）：保持不变
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 初始值() {
        let f = FlowController::new(4096);
        assert_eq!(f.burst_size, 4096);
        assert_eq!(f.batch_gap_us, 2000);
        assert_eq!(f.in_flight(), 0);
        assert!(!f.should_pause());
    }

    #[test]
    fn 在途达上限才暂停() {
        let mut f = FlowController::new(64);
        f.on_sent(64);
        assert!(f.should_pause(), "在途量达 window 上限必须暂停发送新数据");
        f.on_verdict(64, 64, 64);
        assert!(!f.should_pause());
    }

    #[test]
    fn 样本不足不调整() {
        let mut f = FlowController::new(4096);
        // 只发 10 块，丢 1 块：单 tick 差值下是 10% 丢包，但样本不足
        f.on_verdict(9, 10, 9);
        assert_eq!(f.batch_gap_us, 2000, "样本不足不得触发 AIMD");
        assert_eq!(f.burst_size, 4096);
    }

    #[test]
    fn 拥塞时减速且不动协议层窗口() {
        let mut f = FlowController::new(4096);
        // 累积样本：发 100 收 50 → 50% 丢包
        f.on_verdict(50, 100, 50);
        assert!(f.batch_gap_us > 2000, "拥塞时必须增大批间隔");
        assert_eq!(f.window, 4096, "协议层 window 不得被 AIMD 修改");
        assert!(f.burst_size < 4096);
    }

    #[test]
    fn 通畅时加速但间隔不为负() {
        let mut f = FlowController::new(4096);
        for _ in 0..10 {
            f.on_verdict(1000, 1000, 1000);
        }
        assert_eq!(f.batch_gap_us, 1900);
        assert_eq!(f.burst_size, 4096, "burst_size 不得超过 window");
    }

    #[test]
    fn 丢包率为零时不除零() {
        let mut f = FlowController::new(4096);
        f.on_verdict(0, 0, 0);
        assert_eq!(f.batch_gap_us, 2000);
    }
}
