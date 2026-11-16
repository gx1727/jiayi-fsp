//! 确定性伪随机数发生器（xorshift64*）。
//!
//! 仿真必须可复现：同一个种子必须得到同一条丢包/乱序序列，
//! 否则「断言规范性语义」的测试本身就不确定。

/// xorshift64* 伪随机数发生器。
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// 以给定种子构造。
    pub fn new(seed: u64) -> Self {
        // 种子为 0 时 xorshift 会退化为恒零，这里做一个非零替换
        Rng { state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed } }
    }

    /// 下一个 u64。
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// `[0, n)` 范围内的整数；`n == 0` 时返回 0。
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next_u64() % n
        }
    }

    /// 以概率 `numerator / denominator` 返回 true。
    pub fn chance(&mut self, numerator: u64, denominator: u64) -> bool {
        denominator != 0 && self.below(denominator) < numerator
    }
}
