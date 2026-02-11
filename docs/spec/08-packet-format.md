# 08 UDP 数据包格式规范（Packet Format）

## 数据包结构（网络字节序，小端）

| 字段 | 大小 | 类型 | 说明 |
|------|------|------|------|
| session_id | 4 | u32 | 会话唯一标识 |
| chunk_id | 4 | u32 | 逻辑位置 |
| payload_len | 2 | u16 | 载荷长度 |
| flags | 2 | u16 | 标志位 |
| payload | N | bytes | 文件数据 |
| crc32 | 4 | u32 | CRC32 校验码（覆盖 header+payload） |

## 头部大小

```
HEADER_SIZE = 4 + 4 + 2 + 2 = 12 字节
CRC_SIZE = 4 字节
```

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
payload: [session_id in little-endian]
crc32: [4 bytes checksum]
```

## 错误处理

任一关键字段不匹配或校验失败均丢弃。

## 扩展

- flags 预留位
- 不兼容扩展通过 MQTT 协议版本协商
