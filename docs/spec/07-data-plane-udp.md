# 07 数据面（UDP / TCP Fallback）

## UDP 原则

- 无连接、无逐包 ACK、允许丢包与乱序
- payload_len ≤ chunk_size（建议 chunk_size ≤ 1400，避免 IP 分片）
- crc32 包级校验

## 发送端

- 按 window 连续发送
- 间隔 2–5ms（初值；由 AIMD 流控动态调整，见 11 章）
- 仅对 missing_chunks 重传

## 接收端

- 校验头与 crc32
- 随机写盘
- 标记位图
- 不直接推进状态机

## UDP 可用性探测

### 交互流程

1. Client 发送 UDP_PROBE
2. Server 返回 UDP_PROBE_ACK
3. Server 通过 MQTT 发送 udp_ready

### PROBE 数据包

- flags.bit3=1
- payload 为 session_id（u32）的 **4 字节小端编码**
  - v0.2 草案曾写作"8 字节"，与 u32 的 session_id 定义矛盾，已在 v1.0.0 更正

### PROBE_ACK 数据包

- flags.bit3=1
- payload 为 session_id + 1 字节状态码（0=成功，1=失败）

### 超时

- Client 等待 PROBE_ACK 超时（建议 2s）则判定 UDP 不可用
- 回退到 TCP

## TCP 降级

- UDP 不可用或异常时启用
- 保持 chunk 与 bitmap 语义
