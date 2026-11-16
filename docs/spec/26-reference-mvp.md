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
          │              │  ↑                        │
          │              │  └────────────────────────┘
          │              ↓
          │           PAUSED
          ↓
       ABORTED （从任意状态可到达）
```

状态推进**仅由 MQTT 控制消息驱动**（接收侧由本地事件驱动，
但每次推进**必须**通过 MQTT 同步给对端，见 10 章「推进规则」）。

## Chunk

- chunk_size = 1200（建议范围 1024–1350，上限 1400）
- chunk_count = ceil(file_size / chunk_size)

## UDP 接收

```
verify_crc → match_session → pwrite(offset = chunk_id * chunk_size) → bitmap.mark
```

- 校验失败的包直接丢弃，**不标记位图**
- `session_id` 不匹配的包丢弃
- `payload_len` 越界（> 1400 或 > 协商 chunk_size）的包丢弃
- `chunk_id >= chunk_count` 的包丢弃

## 位图

- `Vec<AtomicU64>` 位集，每 64 个 bit 一个字
- 周期刷盘（建议每 1s 或每 100 chunk 一次）

## 缺失计算与下发

- 遍历位图 → missing[] → MQTT 下发
- 以 u64 为单位批量跳过全 1 的块，仅对非全 1 的字逐位检查
- 单条载荷**不超过 4096 条**，超出则分批下发（见 18 章）
- **每 50ms 下发一次 `missing_chunks`（`mode=report`），包括空列表**
  —— 空列表是发送方继续推进的裁决

## 发送

- 按批次发送：连续 `burst_size` 个 chunk，批间隔 `batch_gap_us`
- `burst_size` 初值 = `window`（规范默认 **4096**）；`batch_gap_us` 初值 = 2000µs
- 窗口是**在途上限**：在途量触及 `window` 时暂停发送新数据（批次内部不中断），
  收到裁决后重新计算
- 裁决超时 1000ms → 发布 `missing_chunks`（`mode=request`）索要

## 重传

- Client 接收 `missing_chunks`（`mode=report`）并重传
- 重传包置 `FLAG_RESEND`

## 恢复

- Server 持久化临时文件 / 位图 / 元数据
- 恢复时加载位图，下发 `resume_ack`（`next_chunk_id` = **首个缺失块**）
- 空洞块由周期裁决持续指认并补发

## 完成

- bitmap 全真 → 校验文件 Blake3 hash → 原子重命名 → `upload_complete`
- 校验失败 → `session_abort`（`reason` = `FILE_HASH_MISMATCH`）

## 验收清单

- 乱序可完成
- 中断可继续，且**空洞块**（低于最大已收 id 的缺失块）能被补发
- 内存不随文件增长
- 位图丢失可恢复
- 无 UDP ACK
- CRC32 校验失败的数据包不污染位图
- `file_hash` 在全链路保持为 Blake3 十六进制串
- `session_id` 在全链路保持为 JSON 字符串
- 接收方在**零丢包**时也周期下发空裁决，发送方不会停滞

## 下一阶段

- 下载流、TCP fallback 的线路格式、并发 session、加密与认证
