# 26 最小参考实现（Reference MVP）

> 本文档为 **非规范性文档（Non-Normative）**，用于指导实现一个最小可用的 FSP 端到端。

## 目标

- 单文件上传
- 控制面 MQTT
- 数据面 UDP
- 随机写盘与恢复

## 状态

```
INIT → NEGOTIATING → TRANSFERRING → VERIFY → COMMIT → COMPLETED
                          ↓
                      ABORTED (从任意状态可到达)
```

状态推进**仅由 MQTT 控制消息驱动**。

## Chunk

- chunk_size = 1200（建议范围 1024–1350，上限 1400）
- chunk_count = ceil(file_size / chunk_size)

## UDP 接收

```
verify_crc → match_session → pwrite(offset = chunk_id * chunk_size) → bitmap.mark
```

- 校验失败的包直接丢弃，**不标记位图**
- `session_id` 不匹配的包丢弃

## 位图

- `Vec<AtomicU64>` 位集，每 64 个 bit 一个字
- 周期刷盘（建议每 1s 或每 100 chunk 一次）

## 缺失计算与下发

- 遍历位图 → missing[] → MQTT 下发
- 以 u64 为单位批量跳过全 1 的块，仅对非全 1 的字逐位检查

## 重传

- Client 接收 `missing_chunks` 并重传
- 重传包置 `FLAG_RESEND`

## 恢复

- Server 持久化临时文件 / 位图 / 元数据
- 恢复时加载位图并下发缺失

## 完成

- bitmap 全真 → 校验文件 Blake3 hash → 原子重命名 → `upload_complete`

## 验收清单

- 乱序可完成
- 中断可继续
- 内存不随文件增长
- 位图丢失可恢复
- 无 UDP ACK
- CRC32 校验失败的数据包不污染位图
- `file_hash` 在全链路保持为 Blake3 十六进制串

## 下一阶段

- 下载、TCP fallback、并发 session、加密与认证
