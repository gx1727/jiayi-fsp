//! 位图与块图 —— 规范 `18-bitmap-and-chunk-map.md`。
//!
//! 位图用 `AtomicU64` 数组存储，每 64 个 bit 一个字，支持并发安全的状态更新。
//!
//! ## 关键语义：`first_missing_id`
//!
//! 规范 13 章把 `resume_ack.next_chunk_id` 定义为**第一个尚未收到的 chunk_id**，
//! 而不是「最大已收 id + 1」。本协议显式支持乱序写与随机访问，位图会出现
//! **空洞**（低于最大已收 id 的缺失块）；若以 `max_received_id + 1` 作为续传起点，
//! 空洞块将**永远不会被重传**，文件级 Blake3 校验必然失配。

use std::fs;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};

/// 接收进度位图。
#[derive(Debug)]
pub struct Bitmap {
    words: Vec<AtomicU64>,
    chunk_count: u32,
}

impl Bitmap {
    /// 构造一个空位图（`word_count = (chunk_count + 63) / 64`）。
    pub fn new(chunk_count: u32) -> Self {
        let word_count = (chunk_count as usize + 63) / 64;
        let words = (0..word_count).map(|_| AtomicU64::new(0)).collect();
        Self { words, chunk_count }
    }

    /// 总 chunk 数量。
    pub fn chunk_count(&self) -> u32 {
        self.chunk_count
    }

    /// 标记 chunk 为已接收；返回是否为**新标记**（重复 chunk 静默丢弃，幂等）。
    pub fn mark_received(&self, chunk_id: u32) -> bool {
        if chunk_id >= self.chunk_count {
            return false;
        }
        let word = chunk_id as usize / 64;
        let bit = chunk_id as usize % 64;
        let mask = 1u64 << bit;
        let prev = self.words[word].fetch_or(mask, Ordering::AcqRel);
        prev & mask == 0
    }

    /// 检查 chunk 是否已接收。
    pub fn is_received(&self, chunk_id: u32) -> bool {
        if chunk_id >= self.chunk_count {
            return false;
        }
        let word = chunk_id as usize / 64;
        let bit = chunk_id as usize % 64;
        self.words[word].load(Ordering::Acquire) & (1u64 << bit) != 0
    }

    /// 是否所有 chunk 都已接收。
    pub fn is_complete(&self) -> bool {
        self.received_count() == self.chunk_count
    }

    /// 已接收的 chunk 数量。
    pub fn received_count(&self) -> u32 {
        let mut count = 0u32;
        for word in &self.words {
            count += word.load(Ordering::Acquire).count_ones();
        }
        count
    }

    /// 已接收的最大 chunk ID；一个都没有时返回 `None`。
    pub fn max_received_id(&self) -> Option<u32> {
        for word_index in (0..self.words.len()).rev() {
            let word = self.words[word_index].load(Ordering::Acquire);
            if word != 0 {
                return Some((word_index as u32) * 64 + (63 - word.leading_zeros()));
            }
        }
        None
    }

    /// **第一个尚未收到的 chunk_id**；全部接收时返回 `None`。
    ///
    /// 这是 `resume_ack.next_chunk_id` 的权威来源（见 13 章）。
    pub fn first_missing_id(&self) -> Option<u32> {
        for word_index in 0..self.words.len() {
            let word = self.words[word_index].load(Ordering::Acquire);
            if word != u64::MAX {
                // 第一个为 0 的 bit —— 逐位检查非全 1 的字
                let inv = !word;
                let bit = inv.trailing_zeros() as usize;
                let id = (word_index as u32) * 64 + bit as u32;
                return if id < self.chunk_count { Some(id) } else { None };
            }
        }
        None
    }

    /// 缺失的 chunk 列表（升序）。
    ///
    /// 缺失计算优化：以 u64 为单位批量跳过全 1 的块，
    /// 只有当 `word != u64::MAX` 时才逐位检查。
    pub fn missing_chunks(&self) -> Vec<u32> {
        self.missing_chunks_limit(self.chunk_count as usize)
    }

    /// 缺失的 chunk 列表，限制条数（升序）。
    pub fn missing_chunks_limit(&self, limit: usize) -> Vec<u32> {
        let mut out = Vec::new();
        for word_index in 0..self.words.len() {
            let word = self.words[word_index].load(Ordering::Acquire);
            if word == u64::MAX {
                continue; // 批量跳过全 1 的块
            }
            let mut inv = !word;
            while inv != 0 {
                let bit = inv.trailing_zeros() as usize;
                inv &= inv - 1;
                let id = (word_index as u32) * 64 + bit as u32;
                if id >= self.chunk_count {
                    break;
                }
                out.push(id);
                if out.len() >= limit {
                    return out;
                }
            }
        }
        out
    }

    /// 序列化为字节数组（每字小端）。
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.words.len() * 8);
        for word in &self.words {
            out.extend_from_slice(&word.load(Ordering::Acquire).to_le_bytes());
        }
        out
    }

    /// 从字节数组恢复位图。
    pub fn from_bytes(bytes: &[u8], chunk_count: u32) -> Self {
        let mut bitmap = Bitmap::new(chunk_count);
        for (index, word) in bitmap.words.iter_mut().enumerate() {
            let offset = index * 8;
            if offset + 8 > bytes.len() {
                break;
            }
            let mut raw = [0u8; 8];
            raw.copy_from_slice(&bytes[offset..offset + 8]);
            word.store(u64::from_le_bytes(raw), Ordering::Release);
        }
        bitmap
    }

    /// 保存位图到文件。
    pub fn save_to_file(&self, path: &std::path::Path) -> io::Result<()> {
        fs::write(path, self.to_bytes())
    }

    /// 从文件加载位图。
    pub fn load_from_file(path: &std::path::Path, chunk_count: u32) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        Ok(Bitmap::from_bytes(&bytes, chunk_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 标记与幂等() {
        let b = Bitmap::new(200);
        assert!(b.mark_received(5));
        assert!(!b.mark_received(5), "重复 chunk 必须静默丢弃");
        assert!(b.is_received(5));
        assert_eq!(b.received_count(), 1);
    }

    #[test]
    fn 空洞块的第一个缺失_id() {
        let b = Bitmap::new(1000);
        for id in 0..1000u32 {
            if id != 50 {
                b.mark_received(id);
            }
        }
        // 关键：空洞在 50，而最大已收 id 是 999
        assert_eq!(b.max_received_id(), Some(999));
        assert_eq!(b.first_missing_id(), Some(50), "续传起点必须是第一个缺失块");
        assert_eq!(b.missing_chunks(), vec![50]);
    }

    #[test]
    fn 全真与首个缺失() {
        let b = Bitmap::new(64);
        for id in 0..64u32 {
            b.mark_received(id);
        }
        assert!(b.is_complete());
        assert_eq!(b.first_missing_id(), None);
        assert_eq!(b.max_received_id(), Some(63));
    }

    #[test]
    fn 缺失列表升序且可限量() {
        let b = Bitmap::new(300);
        for id in [0u32, 1, 64, 65, 200, 299] {
            b.mark_received(id);
        }
        let all = b.missing_chunks();
        assert!(all.windows(2).all(|w| w[0] < w[1]), "必须升序");
        assert_eq!(all.len(), 300 - 6);
        assert_eq!(b.missing_chunks_limit(10).len(), 10);
    }

    #[test]
    fn 序列化往返() {
        let b = Bitmap::new(130);
        b.mark_received(0);
        b.mark_received(129);
        let restored = Bitmap::from_bytes(&b.to_bytes(), 130);
        assert!(restored.is_received(0));
        assert!(restored.is_received(129));
        assert!(!restored.is_received(64));
        assert_eq!(restored.received_count(), 2);
    }
}
