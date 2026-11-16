//! UDP 数据包格式 —— 规范 `08-packet-format.md`。
//!
//! 全部多字节整数采用**小端序**；头部 12 字节 + payload + 尾部 4 字节 CRC32。
//!
//! ```text
//! 偏移  字段          类型     说明
//! 0-3   session_id   uint32   会话唯一标识，由 Server 生成
//! 4-7   chunk_id     uint32   逻辑位置（从 0 开始）
//! 8-9   payload_len  uint16   载荷长度（不含头部与 CRC）
//! 10-11 flags        uint16   标志位
//! 12..  payload      bytes    文件数据
//! 12+N  crc32        uint32   覆盖 [0, 12+N)，即 header + payload
//! ```

use crate::crc32::crc32;
use std::fmt;

/// 头部大小（字节）= 4 + 4 + 2 + 2。
pub const HEADER_SIZE: usize = 4 + 4 + 2 + 2;

/// CRC32 校验码大小（字节）。
pub const CRC_SIZE: usize = 4;

/// 载荷长度上限（字节），避免 IP 分片。
pub const MAX_PAYLOAD_LEN: usize = 1400;

/// 建议的 chunk 大小下限。
pub const CHUNK_SIZE_MIN: u32 = 1024;

/// 建议的 chunk 大小上限。
pub const CHUNK_SIZE_MAX: u32 = 1400;

/// 规范默认的 `chunk_size`（见 08 章「参考实现默认」）。
pub const DEFAULT_CHUNK_SIZE: u32 = 1200;

/// 规范默认的滑动窗口 `window`（见 11 章）。
pub const DEFAULT_WINDOW: u32 = 4096;

/// Flags 位定义（08 章）。
pub mod flags {
    /// 数据包标志。
    pub const DATA: u16 = 0x0001;
    /// 重传包标志；接收方无需区别处理（重传是幂等的）。
    pub const RESEND: u16 = 0x0002;
    /// 最后一个 chunk 标志，置位时该包的 `chunk_id` = `chunk_count - 1`；
    /// 可选，接收方不得依赖它判断完成。
    pub const LAST_CHUNK: u16 = 0x0004;
    /// 探测包标志；PROBE 与 PROBE_ACK 共用，以 `payload_len` 区分（4 / 5）。
    pub const PROBE: u16 = 0x0008;
}

/// 数据面协议错误（对应 19 章的 `TRANSPORT` / `INTEGRITY` / `PROTOCOL` 分类）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketError {
    /// `INVALID_PACKET`：长度不足或包长非法。
    InvalidPacket,
    /// `INVALID_PACKET`：`payload_len` 超过 1400 或超过协商的 chunk_size。
    PayloadTooLarge,
    /// `INVALID_CHUNK`：`chunk_id` 越界。
    ChunkOutOfRange,
    /// `SESSION_ID_MISMATCH`：`session_id` 与本会话不符。
    SessionMismatch,
    /// `CRC32_INVALID`：CRC32 校验失败。
    CrcMismatch,
    /// `INVALID_PACKET`：末块长度与 `size` 不符。
    LengthMismatch,
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PacketError::InvalidPacket => "INVALID_PACKET",
            PacketError::PayloadTooLarge => "INVALID_PACKET (payload too large)",
            PacketError::ChunkOutOfRange => "INVALID_CHUNK",
            PacketError::SessionMismatch => "SESSION_ID_MISMATCH",
            PacketError::CrcMismatch => "CRC32_INVALID",
            PacketError::LengthMismatch => "INVALID_PACKET (length mismatch)",
        };
        f.write_str(s)
    }
}

impl std::error::Error for PacketError {}

/// 接收端校验所需的会话上下文。
#[derive(Debug, Clone, Copy)]
pub struct DecodeCtx {
    /// 本会话的 `session_id`。
    pub session_id: u32,
    /// 协商确定的 `chunk_size`。
    pub chunk_size: u32,
    /// 总 chunk 数量。
    pub chunk_count: u32,
    /// 文件总大小（用于校验末块长度）。
    pub size: u64,
}

impl DecodeCtx {
    /// 按 `chunk_size` 与 `size` 推导 `chunk_count`（`ceil(size / chunk_size)`）。
    pub fn new(session_id: u32, chunk_size: u32, size: u64) -> Self {
        let chunk_count = if chunk_size == 0 {
            0
        } else {
            size.div_ceil(u64::from(chunk_size)) as u32
        };
        Self { session_id, chunk_size, chunk_count, size }
    }
}

/// 一个已解码的数据包。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    /// 会话唯一标识。
    pub session_id: u32,
    /// 逻辑位置（从 0 开始），对应文件偏移 `chunk_id * chunk_size`。
    pub chunk_id: u32,
    /// 标志位。
    pub flags: u16,
    /// 文件数据。
    pub payload: Vec<u8>,
}

impl Packet {
    /// 构造一个数据包。
    pub fn new(session_id: u32, chunk_id: u32, flags: u16, payload: Vec<u8>) -> Self {
        Self { session_id, chunk_id, flags, payload }
    }

    /// 序列化为线上字节。
    ///
    /// CRC32 覆盖 `[0, 12 + payload_len)`，即头部与 payload，不含尾部 CRC 字段本身。
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_SIZE + self.payload.len() + CRC_SIZE);
        out.extend_from_slice(&self.session_id.to_le_bytes());
        out.extend_from_slice(&self.chunk_id.to_le_bytes());
        out.extend_from_slice(&(self.payload.len() as u16).to_le_bytes());
        out.extend_from_slice(&self.flags.to_le_bytes());
        out.extend_from_slice(&self.payload);
        let sum = crc32(&out);
        out.extend_from_slice(&sum.to_le_bytes());
        out
    }

    /// 从线上字节解码，并按 08 章的规则校验。
    pub fn decode(buf: &[u8], ctx: &DecodeCtx) -> Result<Self, PacketError> {
        if buf.len() < HEADER_SIZE + CRC_SIZE {
            return Err(PacketError::InvalidPacket);
        }
        let session_id = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let chunk_id = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let payload_len = u16::from_le_bytes([buf[8], buf[9]]) as usize;
        let flags = u16::from_le_bytes([buf[10], buf[11]]);

        if session_id != ctx.session_id {
            return Err(PacketError::SessionMismatch);
        }
        // payload_len 必须落在 [0, min(1400, chunk_size)] 内
        let cap = MAX_PAYLOAD_LEN.min(ctx.chunk_size as usize);
        if payload_len > cap {
            return Err(PacketError::PayloadTooLarge);
        }
        if buf.len() != HEADER_SIZE + payload_len + CRC_SIZE {
            return Err(PacketError::InvalidPacket);
        }
        // CRC32 覆盖头部 + payload，不含尾部 CRC 字段
        let expect = u32::from_le_bytes([
            buf[HEADER_SIZE + payload_len],
            buf[HEADER_SIZE + payload_len + 1],
            buf[HEADER_SIZE + payload_len + 2],
            buf[HEADER_SIZE + payload_len + 3],
        ]);
        if crc32(&buf[..HEADER_SIZE + payload_len]) != expect {
            return Err(PacketError::CrcMismatch);
        }
        // 数据包类才校验 chunk_id 范围与末块长度；探测包另行处理（07 章）
        if flags & flags::PROBE == 0 {
            if chunk_id >= ctx.chunk_count {
                return Err(PacketError::ChunkOutOfRange);
            }
            let expect_len = expected_chunk_len(ctx, chunk_id);
            if payload_len != expect_len {
                return Err(PacketError::LengthMismatch);
            }
        }
        Ok(Self {
            session_id,
            chunk_id,
            flags,
            payload: buf[HEADER_SIZE..HEADER_SIZE + payload_len].to_vec(),
        })
    }
}

/// 某个 chunk 的期望载荷长度（末块通常更短）。
pub fn expected_chunk_len(ctx: &DecodeCtx, chunk_id: u32) -> usize {
    let chunk_size = u64::from(ctx.chunk_size);
    let offset = u64::from(chunk_id) * chunk_size;
    if offset >= ctx.size {
        return 0;
    }
    let remain = ctx.size - offset;
    remain.min(chunk_size) as usize
}

/// UDP 探测包（07 章）：payload 为 `session_id` 的 4 字节小端编码。
pub fn encode_probe(session_id: u32) -> Vec<u8> {
    Packet::new(session_id, 0, flags::PROBE, session_id.to_le_bytes().to_vec()).encode()
}

/// UDP 探测应答包（07 章）：payload 为 `session_id` + 1 字节状态码，共 5 字节。
pub fn encode_probe_ack(session_id: u32, ok: bool) -> Vec<u8> {
    let mut payload = session_id.to_le_bytes().to_vec();
    payload.push(u8::from(ok));
    Packet::new(session_id, 0, flags::PROBE, payload).encode()
}

/// 区分 PROBE 与 PROBE_ACK 的判别规则（07 章）：以 `payload_len` 区分（4 / 5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeKind {
    /// 探测请求（payload_len = 4）。
    Probe,
    /// 探测应答（payload_len = 5）。
    ProbeAck,
}

/// 按 `payload_len` 判定探测包种类；其余长度按 `INVALID_PACKET` 处理。
pub fn classify_probe(buf: &[u8]) -> Result<ProbeKind, PacketError> {
    if buf.len() < HEADER_SIZE + CRC_SIZE {
        return Err(PacketError::InvalidPacket);
    }
    let flags = u16::from_le_bytes([buf[10], buf[11]]);
    if flags & flags::PROBE == 0 {
        return Err(PacketError::InvalidPacket);
    }
    let payload_len = u16::from_le_bytes([buf[8], buf[9]]) as usize;
    match payload_len {
        4 => Ok(ProbeKind::Probe),
        5 => Ok(ProbeKind::ProbeAck),
        _ => Err(PacketError::InvalidPacket),
    }
}
