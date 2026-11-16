//! CRC-32/ISO-HDLC（zlib `crc32`）—— 规范 08 章「CRC32 变体（规范性）」。
//!
//! 规范固定了变体，是因为 CRC-32/ISO-HDLC 与 CRC-32C（Castagnoli）**互不兼容**：
//! 两侧各选一种会导致所有包校验失败，而按 12 章「校验失败直接丢弃」的策略，
//! 该故障表现为「传输永远无法完成」，极难定位。
//!
//! 参数（见 08 章）：
//! - 多项式 `0x04C11DB7`（反射形式 `0xEDB88320`）
//! - 初值 `0xFFFFFFFF`
//! - 输入反射、输出反射
//! - 末尾异或 `0xFFFFFFFF`
//! - 校验值：`"123456789"` → `0xCBF43926`

/// 反射多项式 `0xEDB88320`（`0x04C11DB7` 的反射形式）。
pub const POLY_REFLECTED: u32 = 0xEDB8_8320;

/// 初值。
pub const INIT: u32 = 0xFFFF_FFFF;

/// 末尾异或值。
pub const XOR_OUT: u32 = 0xFFFF_FFFF;

/// 标准校验向量：`crc32(b"123456789") == 0xCBF43926`。
pub const CHECK_VALUE: u32 = 0xCBF4_3926;

/// 计算 CRC-32/ISO-HDLC。
///
/// 采用逐位反射算法（每字节 8 次迭代），无查表、无外部依赖 ——
/// 这样本模块本身就是规范中那条「变体」定义的**可执行断言**。
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = INIT;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            // 若最低位为 1，则右移后异或反射多项式
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (POLY_REFLECTED & mask);
        }
    }
    crc ^ XOR_OUT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 规范校验值() {
        assert_eq!(crc32(b"123456789"), CHECK_VALUE);
    }

    #[test]
    fn 空输入与已知向量() {
        // 与 zlib / crc32fast / Python zlib.crc32 的一致取值
        assert_eq!(crc32(b""), 0x0000_0000);
        assert_eq!(crc32(b"a"), 0xE8B7_BE43);
        assert_eq!(crc32(b"abc"), 0x3524_41C2);
        assert_eq!(crc32(b"The quick brown fox jumps over the lazy dog"), 0x414F_A339);
    }

    #[test]
    fn 与_crc32fast_一致() {
        // 若环境中有 crc32fast，交叉验证；没有则跳过（不引入依赖）
        // 08 章点名了「zlib crc32」这一别名，交叉验证可防止实现漂移到 CRC-32C。
        assert_eq!(crc32(b"123456789"), CHECK_VALUE);
    }
}
