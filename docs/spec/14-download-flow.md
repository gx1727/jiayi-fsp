# 14 下载流程（Server → Client）

## 原则

- Client 主动发起
- Server 不向未知端口推送 UDP
- 控制面与可靠性机制与上传一致
- 会话类型：Upload（C→S）、Download（S→C）

## 下载控制消息

| 消息 | 方向 | 用途 |
|------|------|------|
| download_start | Client→Server | 请求下载文件 |
| download_accept | Server→Client | 接受下载请求 |
| missing_chunks | Client→Server | 客户端报告缺失 chunk |
| download_complete | 双向 | 下载完成通知 |

## 下载流程时序

### 1. 会话建立

Client 发送 download_start → Server 返回 download_accept

### 2. UDP 探测

Client 发送 UDP_PROBE → Server 返回 UDP_PROBE_ACK → Server 发送 udp_ready

### 3. 数据传输

Server 按 window 连续发送 UDP chunk

### 4. 缺失重传

Client 发送 missing_chunks → Server 重传缺失 chunk

### 5. 完成与终止

Client 检测位图全真 → 校验 hash → 原子重命名 → 发送 download_complete

## 与上传对称性

| 阶段 | 上传（C→S） | 下载（S→C） |
|------|-------------|-------------|
| 会话建立 | upload_start | download_start |
| 参数下发 | upload_accept | download_accept |
| 数据方向 | Client→Server | Server→Client |
| 缺失请求 | Server→Client | Client→Server |
| 完成通知 | upload_complete | download_complete |
