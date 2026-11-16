# 14 下载流程（Server → Client）

## 原则

- Client 主动发起
- Server 不向未知端口推送 UDP（客户端端口由 `download_accept.udp_port` 与 UDP 探测确定）
- 控制面与可靠性机制与上传一致
- 会话类型：Upload（C→S）、Download（S→C）

## 下载控制消息

| 消息 | 方向 | 用途 |
|------|------|------|
| download_start | Client→Server | 请求下载文件 |
| download_accept | Server→Client | 接受下载请求，下发会话参数与数据面端口 |
| missing_chunks | 双向 | 缺失 chunk 列表报告 / 索要（`mode=report` / `mode=request`） |
| download_complete | Client→Server | 下载完成（由接收方 Client 发布） |

## 下载流程时序

### 1. 会话建立

Client 发送 download_start → Server 返回 download_accept（含 `session_id` /
`chunk_count` / `window` / `udp_port` / `tcp_port`）

### 2. UDP 探测

Client 发送 UDP_PROBE → Server 返回 UDP_PROBE_ACK → Server 发送 udp_ready

Server 以 PROBE 报文的**源地址:端口**作为该会话的对端数据面地址。

**数据面端口的优先级（规范性）**：Client **必须**按以下顺序取用**最新**的端口值：

| 优先级 | 来源 | 生效时机 |
|--------|------|----------|
| 1（最高） | `resume_ack.udp_port` | 会话恢复后（端口可能因会话重建而变化） |
| 2 | `udp_ready.udp_port` | 探测完成时（最新的探测结果） |
| 3 | `download_accept.udp_port` | 参数协商时（初始值） |

三者不一致时**必须**取优先级最高者；端口变更**必须**通过上述消息显式下发，
Client **不得**自行推断。

### 3. 数据传输

Server 按 `window` 连续发送 UDP chunk（批量发送，节奏见 11 章）

### 4. 缺失重传

Server 每 50ms 下发 `missing_chunks`（`mode=report`）裁决；
Client 收到后按 11 章补发缺失块。
收到 Client 的 `missing_chunks`（`mode=request`）时，Server **必须**在 50ms 内应答。

### 5. 完成与终止

Client 检测位图全真 → 校验 hash → 原子重命名 → 发送 download_complete
（**接收方发布完成消息**，见 06 章）

## 与上传对称性

| 阶段 | 上传（C→S） | 下载（S→C） |
|------|-------------|-------------|
| 会话建立 | upload_start | download_start |
| 参数下发 | upload_accept | download_accept |
| 数据方向 | Client→Server | Server→Client |
| 完成通知 | upload_complete（**Server** 发布） | download_complete（**Client** 发布） |
| 缺失报告 | Server→Client（Server 报告自己缺的块） | Client→Server（Client 报告自己缺的块） |
| 缺失索要 | Client→Server（`mode=request`） | Server→Client（`mode=request`） |
| 裁决节奏 | 接收方（Server）每 50ms 下发 | 接收方（Client）每 50ms 下发 |

**完成消息一律由接收方发布（规范性）**：接收方持有位图与临时文件，只有它能完成
「校验 hash → 原子重命名」并判定成败。
