# Packet Definition

UDP 数据包格式定义，见 `fsp_packet.h` / `fsp_packet.rs`。

规范来源：[08 UDP 数据包格式规范](../docs/spec/08-packet-format.md)（协议版本 v1.0.0）。

| 偏移 | 字段 | 类型 | 说明 |
|------|------|------|------|
| 0-3 | `session_id` | u32 | 会话唯一标识，由 Server 生成 |
| 4-7 | `chunk_id` | u32 | 逻辑位置（从 0 开始） |
| 8-9 | `payload_len` | u16 | 载荷长度（不含头部与 CRC） |
| 10-11 | `flags` | u16 | 标志位 |
| 12.. | `payload` | bytes | 文件数据 |
| End-4 | `crc32` | u32 | CRC32 校验码（覆盖 header+payload） |

- 头部 `HEADER_SIZE = 12` 字节，`CRC_SIZE = 4` 字节
- 全部多字节整数采用**小端序**（Little-Endian）
- 载荷长度上限 `MAX_PAYLOAD_LEN = 1400` 字节（避免 IP 分片）
