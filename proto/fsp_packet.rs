//! JIAYI-FSP UDP 数据包格式定义
//!
//! 规范来源: `docs/spec/08-packet-format.md`（协议版本 v1.0.0）
//!
//! 全部多字节整数采用**小端序**（Little-Endian）。
//! 线上布局为：头部 12 字节 + payload + 尾部 4 字节 CRC32。

/// 数据包头部大小（字节）= 4(session_id) + 4(chunk_id) + 2(payload_len) + 2(flags)
pub const HEADER_SIZE: usize = 4 + 4 + 2 + 2;

/// CRC32 校验码大小（字节）
pub const CRC_SIZE: usize = 4;

/// 载荷长度上限（字节），避免 IP 分片
pub const MAX_PAYLOAD_LEN: usize = 1400;

/// Flags 位定义
pub const FLAG_DATA: u16 = 0x0001; // 数据包标志
pub const FLAG_RESEND: u16 = 0x0002; // 重传包标志
pub const FLAG_LAST_CHUNK: u16 = 0x0004; // 最后一个 chunk 标志
pub const FLAG_PROBE: u16 = 0x0008; // 探测包标志

/// FSP 数据包固定头部。
///
/// 完整数据包 = 本头部 + `payload`（`payload_len` 字节）+ `crc32`（4 字节）。
/// `crc32` 覆盖 header + payload。
#[repr(C)]
pub struct FspChunkPacket {
    /// 会话唯一标识，由 Server 生成
    pub session_id: u32,
    /// 逻辑位置（从 0 开始），对应文件偏移 `chunk_id * chunk_size`
    pub chunk_id: u32,
    /// 载荷长度（不含头部与 CRC）
    pub payload_len: u16,
    /// 标志位
    pub flags: u16,
}
