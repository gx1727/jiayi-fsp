# 14 下载流程（Server → Client）

## 原则

- Client 主动发起；Server 不向未知端口推送 UDP；控制面与可靠性机制与上传一致

## 会话类型

- Upload（C→S）
- Download（S→C）

## 下载控制消息

- download_start（Client→Server）：请求下载文件
- download_accept（Server→Client）：接受下载请求，携带会话参数
- missing_chunks（Server→Client）：下发缺失 chunk 列表
- download_complete（Server→Client）：可选，下载完成通知

## 下载流程时序

### 1. 会话建立

- Client 发送 download_start（Topic: jiayi/fsp/client/{client_id}/download_start）
- Server 验证文件权限，返回 download_accept（Topic: jiayi/fsp/server/{client_id}/download_accept）
- 参数：session_id、chunk_count、window、tcp_port

### 2. UDP 探测（NAT 场景）

- Client 发送 UDP_PROBE 到 Server 的 udp_port
- Server 记录 Client 源端口，返回 UDP_PROBE_ACK
- Server 通过 MQTT 发送 udp_ready

### 3. 数据传输

- Server 按 window 连续发送 UDP chunk（与上传方向相反）
- Client 接收、校验、随机写盘、更新位图
- Server 周期统计缺失，下发 missing_chunks

### 4. 缺失重传

- Client 发送 missing_chunks 请求（或 Server 主动下发）
- Server 优先重传缺失 chunk

### 5. 完成与终止

- Client 检测位图全真 → 校验文件 hash
- 校验通过 → 原子重命名 → 发送 download_complete
- 失败 → 发送 session_abort

## UDP 行为（NAT）

- Client 主动发送 UDP_PROBE 建立 NAT 映射
- Server 仅向 Client 源端口回包
- Server 发送 chunk 时使用 Client 记录的端口

## 与上传对称性

| 阶段 | 上传（C→S） | 下载（S→C） |
|------|-------------|-------------|
| 会话建立 | upload_start | download_start |
| 参数下发 | upload_accept | download_accept |
| 数据方向 | Client→Server | Server→Client |
| 缺失请求 | missing_chunks | missing_chunks |
| 完成通知 | upload_complete | download_complete |
