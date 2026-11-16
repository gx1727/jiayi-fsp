# 08 UDP 数据包格式规范（Packet Format）

## 数据包结构（小端序 Little-Endian）

> **端序约定**：本协议所有多字节整数字段一律采用 **小端序（Little-Endian）**。
> v0.2 草案中"网络字节序，小端"的表述自相矛盾，已在 v1.0.0 中统一为小端；
> 参考实现 `jiayi-fsp-core` 采用 `to_le_bytes` / `from_le_bytes`，与本节一致。

| 字段 | 偏移 | 大小 | 类型 | 说明 |
|------|------|------|------|------|
| session_id | 0-3 | 4 | u32 | 会话唯一标识，由 Server 生成 |
| chunk_id | 4-7 | 4 | u32 | 逻辑位置（从 0 开始） |
| payload_len | 8-9 | 2 | u16 | 载荷长度（不含头部与 CRC） |
| flags | 10-11 | 2 | u16 | 标志位 |
| payload | 12.. | N | bytes | 文件数据 |
| crc32 | 12+N | 4 | u32 | CRC32 校验码（覆盖 header+payload） |

## 头部大小

```
HEADER_SIZE = 4 + 4 + 2 + 2 = 12 字节
CRC_SIZE = 4 字节
```

## 载荷长度上限

- `payload_len ≤ chunk_size`
- 建议 `chunk_size` 取 1024–1350 字节，**上限 1400 字节**，避免 IP 分片
- 参考实现默认 `chunk_size = 1200`

## Flags 位定义

| 位 | 常量 | 值 | 说明 |
|----|------|-----|------|
| bit0 | FLAG_DATA | 0x0001 | 数据包标志 |
| bit1 | FLAG_RESEND | 0x0002 | 重传包标志 |
| bit2 | FLAG_LAST_CHUNK | 0x0004 | 最后一个 chunk 标志 |
| bit3 | FLAG_PROBE | 0x0008 | 探测包标志 |
| bit4-15 | RESERVED | 0x0000 | 保留位 |

## 数据包示例

**普通数据包**:
```
session_id: 0x00000001
chunk_id: 0x00000010
payload_len: 1024
flags: 0x0001 (FLAG_DATA)
payload: [1024 bytes data]
crc32: [4 bytes checksum]
```

**探测包**:
```
session_id: 0x00000001
chunk_id: 0x00000000
payload_len: 4
flags: 0x0008 (FLAG_PROBE)
payload: [session_id in little-endian, 4 bytes]
crc32: [4 bytes checksum]
```

> 探测包 payload 为 `session_id`（u32）的 **4 字节小端编码**。
> v0.2 草案的 07 章曾写作"8 字节"，已在本版本更正为 4 字节。

## 错误处理

任一关键字段不匹配或校验失败均丢弃。校验失败的数据包**不得**标记位图。

## 扩展

- flags 预留位
- 不兼容扩展通过 MQTT 协议版本协商
