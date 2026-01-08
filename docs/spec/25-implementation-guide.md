# 25 实现者指南（Implementation Guide）

本文为非规范性文档，用于指导工程实现。若与协议规范存在冲突，以协议规范为准。

## 25.1 设计哲学

- 控制慢而可靠，数据快而可丢
- 状态推进、裁决、重传决策 → MQTT
- 高吞吐数据传输 → UDP（或 TCP fallback）

## 25.2 推荐分层

```
MQTT（控制 / 状态）
        ↓
Session State Machine
        ↓
UDP / TCP Data Plane
        ↓
Disk (Random Write)
```

实现约束：
- MQTT 回调 ≠ UDP IO
- UDP IO ≠ 状态机推进

## 25.3 模块划分

Client：
- SessionManager
- MqttController
- UdpSender / UdpReceiver
- ChunkScheduler
- DiskWriter
- BitmapTracker

Server：
- SessionRegistry
- MqttHandler
- UdpReceiver / UdpSender
- ResendController
- DiskWriter
- IntegrityVerifier

## 25.4 Session 管理

- 每个 session_id 对应独立状态机
- 状态机只能由 MQTT 消息推进
- UDP 仅更新 chunk 接收情况（bitmap）
- 状态必须可持久化（支持恢复）

## 25.5 UDP 数据面

发送端：
- 按 chunk_id 顺序发送
- 遵守 window 限制
- 不等待 ACK
- 发送间隔建议 2–5ms

接收端：
- 校验 header、crc32
- 校验通过后随机写盘
- 丢弃校验失败的 chunk
- 禁止将完整文件缓存到内存

## 25.6 Bitmap 实现

- 使用 bitset / bitmap 表示接收情况
- 位图可常驻内存并周期刷盘
- 避免每个 chunk 都同步写位图文件

## 25.7 重传策略

- Server 周期性统计 missing_chunks
- 合并缺失一次性通过 MQTT 下发
- 重传优先级低于新数据发送

## 25.8 TCP Fallback

- UDP 不可用时兜底
- 不并行使用 UDP 与 TCP
- TCP 模式下保持 chunk 语义与 bitmap 机制

## 25.9 移动端注意事项

Android：
- 长时间同步使用 Foreground Service
- 向用户展示同步状态

iOS：
- 数据传输仅在前台进行
- 进入后台视为会话中断
- 回到前台需 resume_session

## 25.10 常见错误

- 在 UDP 层实现 ACK / 拥塞控制
- 用 UDP 判定 session 完成
- 将大文件整体缓存到内存
- 忽略 crc32 校验失败

## 25.11 MVP 建议顺序

1. MQTT 控制流跑通
2. 单文件 UDP 上传
3. missing_chunks 重传
4. 中断恢复
5. TCP fallback

## 25.12 结语

稳定性来自清晰分层、严格边界、尊重真实网络环境。若必须“破坏分层才能跑通”，问题几乎一定在实现方式。 
