//! 缺失裁决 —— 规范 `11-flow-control.md` / `18-bitmap-and-chunk-map.md`。
//!
//! **裁决是发送方推进的唯一依据。** 接收方**必须**至少每
//! [`VERDICT_PERIOD_MS`] 毫秒下发一次 `missing_chunks`（`mode=report`），
//! **包括缺失列表为空的情况** —— 空数组是「可以继续推进」的裁决。
//!
//! 若接收方只在「有缺失」时才下发，发送方会在窗口填满后永远等不到裁决，
//! **协议死锁**。因此「周期性下发空数组裁决」是规范性要求，而非优化建议。

use crate::bitmap::Bitmap;
use crate::flow::{MISSING_LIMIT, VERDICT_PERIOD_MS};

/// 一次裁决。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// 尚未补齐的缺失块**总数**（可能大于 `chunks.len()`，因为分批下发）。
    pub missing_count: u32,
    /// 本批携带的缺失 chunk 列表（升序，最多 [`MISSING_LIMIT`] 条）。
    pub chunks: Vec<u32>,
}

impl Verdict {
    /// 是否表示「没有缺失，可继续推进」。
    pub fn is_clear(&self) -> bool {
        self.chunks.is_empty()
    }
}

/// 依据位图构造一次裁决。
///
/// - `chunks` 按 chunk_id **升序**；
/// - 单条载荷**不得超过 4096 条**，超出则分批下发（18 章）；
/// - `missing_count` 给出**总数**，用于发送方判断是否还有后续批次。
pub fn build_verdict(bitmap: &Bitmap) -> Verdict {
    let total = bitmap.chunk_count().saturating_sub(bitmap.received_count());
    let chunks = bitmap.missing_chunks_limit(MISSING_LIMIT);
    Verdict { missing_count: total, chunks }
}

/// 裁决调度器：跟踪下一次应当下发裁决的时刻。
#[derive(Debug, Clone)]
pub struct VerdictScheduler {
    /// 上次下发裁决的时刻（毫秒）。
    pub last_verdict_at: u64,
}

impl Default for VerdictScheduler {
    fn default() -> Self {
        VerdictScheduler { last_verdict_at: 0 }
    }
}

impl VerdictScheduler {
    /// 新建调度器。
    pub fn new(start_at: u64) -> Self {
        VerdictScheduler { last_verdict_at: start_at }
    }

    /// 到 `now` 为止是否应当下发一次裁决。
    pub fn due(&self, now: u64) -> bool {
        now.saturating_sub(self.last_verdict_at) >= VERDICT_PERIOD_MS
    }

    /// 记录一次下发。
    pub fn on_verdict_sent(&mut self, now: u64) {
        self.last_verdict_at = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 空位图给出空裁决() {
        let b = Bitmap::new(100);
        let v = build_verdict(&b);
        assert_eq!(v.missing_count, 100);
        assert_eq!(v.chunks.len(), 100);
        assert!(!v.is_clear());
    }

    #[test]
    fn 全真给出空数组裁决() {
        let b = Bitmap::new(100);
        for i in 0..100u32 {
            b.mark_received(i);
        }
        let v = build_verdict(&b);
        assert_eq!(v.missing_count, 0);
        assert!(v.chunks.is_empty());
        assert!(v.is_clear(), "空数组是「可以继续推进」的裁决");
    }

    #[test]
    fn 缺失列表分批且总数正确() {
        let b = Bitmap::new(10_000);
        b.mark_received(0);
        let v = build_verdict(&b);
        assert_eq!(v.chunks.len(), MISSING_LIMIT, "单条载荷上限 4096");
        assert_eq!(v.missing_count, 9_999, "missing_count 是总数，不是本批长度");
    }

    #[test]
    fn 调度周期为五十毫秒() {
        let mut s = VerdictScheduler::new(0);
        assert!(!s.due(49));
        assert!(s.due(50));
        s.on_verdict_sent(50);
        assert!(!s.due(99));
        assert!(s.due(100));
    }
}
