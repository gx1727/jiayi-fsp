# 07 数据面（UDP / TCP Fallback）

## UDP 原则

- 无连接、无逐包 ACK、允许丢包与乱序
- payload_len ≤ chunk_size（建议 chunk_size ≤ 1400，避免 IP 分片）
- crc32 包级校验

## 发送端

- **按批次发送**：连续发送 `burst_size` 个 chunk，批间隔 `batch_gap_us`
  （初值 2000µs；`burst_size` 初值 = `window`）。详见 11 章「发送节奏」。
  - v1.0.0 草案此处写作「间隔 2–5ms」（逐包读法），与 11 章的批量读法冲突，
    两者相差约 `window` 倍；本版本统一为**批量读法**。
- **仅对 `missing_chunks`（`mode=report`）报告的缺失块重传**，重传优先级高于新数据
- **数据面不得改变会话状态**，只能更新接收侧位图（见 10 章「推进规则」）

## 接收端

- 校验头与 crc32
- 随机写盘
- 标记位图
- 不直接推进状态机（位图变化只是接收侧状态推进的**输入**）

## UDP 可用性探测

### 交互流程

1. Client 发送 UDP_PROBE
2. Server 返回 UDP_PROBE_ACK
3. Server 通过 MQTT 发送 udp_ready

> v1.0.0 草案的 24 章把同一握手写作 `UDP_HELLO` / `UDP_HELLO_ACK`，
> 本版本统一为 **`UDP_PROBE` / `UDP_PROBE_ACK`**。

### PROBE 数据包

- flags.bit3=1
- payload 为 session_id（u32）的 **4 字节小端编码**
  - v0.2 草案曾写作"8 字节"，与 u32 的 session_id 定义矛盾，已在 v1.0.0 更正

### PROBE_ACK 数据包

- flags.bit3=1
- payload 为 session_id + 1 字节状态码（0=成功，1=失败），共 **5 字节**
- **判别规则（规范性）**：PROBE 与 PROBE_ACK 共用 `flags.bit3`，
  接收方**必须**以 `payload_len` 区分：`payload_len == 4` 为 PROBE，
  `payload_len == 5` 为 PROBE_ACK。其余长度**必须**按 `INVALID_PACKET` 丢弃。

### 端口与地址绑定（规范性）

- Server 的数据面 socket **必须**绑定 accept 消息中下发的 `udp_port`。
- Server **必须**以 PROBE 报文的**源地址:端口**作为该会话的对端数据面地址；
  Server 不得向未经 PROBE 确认的地址发送数据面数据包（见 20 章）。
- Server 返回的 PROBE_ACK **必须**从该会话的数据面端口发出，
  以保证客户端的 NAT 映射命中。

### 超时

- Client 等待 PROBE_ACK 超时（建议 2s）则判定 UDP 不可用
- 回退到 TCP

## TCP 降级

- UDP 不可用或异常时启用
- 保持 chunk 与 bitmap 语义
- **TCP 通道的线路格式在 v1.1.0 中仍未定义**（见 26 章「下一阶段」）；
  本版本仅规定其**语义**要求：chunk 边界、bitmap 语义、状态机行为与 UDP 通道一致
- TCP 端口由 accept 消息的 `tcp_port` 字段下发；**`tcp_port = 0` 表示不支持 TCP 降级**，
  此时探测失败即判定传输不可用
