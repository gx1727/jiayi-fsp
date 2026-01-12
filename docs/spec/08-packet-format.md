# 08 UDP 数据包格式规范（Packet Format）

## 整体结构（网络字节序）

```
| session_id(64) | file_id_hash(64) | chunk_id(32) | payload_len(16) | flags(16) | payload(N) | crc32(32) |
```

## 字段说明

- `session_id`：会话唯一标识
- `file_id_hash`：防跨文件误写
- `chunk_id`：逻辑位置
- `payload_len` ≤ chunk_size
- `flags`：位掩码
  - bit0 DATA
  - bit1 RESEND
  - bit2 LAST_CHUNK
  - bit3 PROBE
  - 4–15 RESERVED=0
- `payload`：文件数据
- `crc32`：覆盖 header+payload

## 错误处理

任一关键字段不匹配或校验失败均丢弃。

## 扩展

- flags 预留位
- 不兼容扩展通过 MQTT 协议版本协商
