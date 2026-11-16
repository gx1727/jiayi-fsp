# 24 端到端时序（MQTT + UDP）

## 时序流程

### 会话建立

upload_start → upload_accept（含会话参数与数据面端口）

### UDP 探测

UDP_PROBE → UDP_PROBE_ACK → udp_ready

> v1.0.0 草案的 24 章把同一握手写作 `UDP_HELLO` / `UDP_HELLO_ACK`，
> 与 07、08 章的 `UDP_PROBE` / `UDP_PROBE_ACK` 不一致，已在 v1.1.0 统一。

### 数据传输

Client 连续发送 UDP chunk；Server 乱序随机写盘

### 缺失统计与重传

接收方每 50ms 下发 `missing_chunks`（`mode=report`）裁决；
发送方按裁决重传缺失块（重传优先级高于新数据）

**裁决循环是发送方推进的唯一依据（规范性）**：接收方**必须**周期性下发裁决，
**即使缺失列表为空**（空数组 = 「可以继续推进」）。
发送方在 1000ms 内未收到任何裁决时，**必须**发布
`missing_chunks`（`mode=request`）主动索要。

### 校验与提交

接收方校验 hash、原子重命名，发布完成消息
（上传方向 `upload_complete`，下载方向 `download_complete`）

### 中断与恢复

resume_session → resume_ack（首个缺失块 + 数据面端口）→ 继续发送

### 下载对称

控制面一致，仅 UDP 方向互换；完成消息改由 Client 发布

## 状态机

状态机与合法转移以 10-state-machine 为准
