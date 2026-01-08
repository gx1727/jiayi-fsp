# 08 UDP 数据包格式规范（Packet Format）

本章为强制规范，定义 UDP 数据包的字段布局、语义与实现要求。任何实现必须严格遵守本章节定义。

## 8.1 包整体结构

```
| session_id (64) | file_id_hash (64) |
| chunk_id (32) | payload_len (16) | flags (16) |
| payload (N bytes) |
| crc32 (32) |
```

- 字段采用网络字节序（Big Endian）
- 所有字段按 32 / 64 位自然对齐

## 8.2 字段定义

| 字段名          | 位宽 | 必选 | 说明                           |
| ---------------- | ---: | :--: | ------------------------------ |
| session_id       | 64   | MUST | 会话唯一标识，与 MQTT 一致     |
| file_id_hash     | 64   | MUST | 文件内容哈希（如 SHA256 前 64 位） |
| chunk_id         | 32   | MUST | 数据块索引，从 0 开始          |
| payload_len      | 16   | MUST | payload 实际字节数             |
| flags            | 16   | MUST | 控制位掩码                     |
| payload          |  N   | MUST | 文件数据，长度 = payload_len   |
| crc32            | 32   | MUST | 包完整性校验（header + payload） |

## 8.3 字段语义

- session_id：由 Server 生成，在会话生命周期内唯一，必须与控制面一致
- file_id_hash：用于防止跨 session / 跨文件误写，整个会话内保持不变
- chunk_id：表示 payload 在文件中的逻辑位置；offset = chunk_id * chunk_size；可乱序、可重复
- payload_len：真实长度，必须 ≤ 协商的 chunk_size
- flags：16 位位掩码
  - bit 0: DATA（普通数据包）
  - bit 1: RESEND（重传数据包）
  - bit 2: LAST_CHUNK（最后一个 chunk）
  - bit 3: PROBE（探测/测试包）
  - 4–15: RESERVED（保留为 0）
- payload：仅包含原始文件数据，不包含控制信息
- crc32：覆盖 [session_id ... payload]，用于包级完整性校验

## 8.4 错误处理

- session_id 不匹配 → 丢弃
- file_id_hash 不匹配 → 丢弃并记录错误
- payload_len 非法 → 丢弃
- crc32 校验失败 → 丢弃

## 8.5 扩展性

- flags 预留位用于未来扩展
- 不兼容扩展须通过 MQTT 协议版本协商完成
