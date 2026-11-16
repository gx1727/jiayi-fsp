# 24 端到端时序（MQTT + UDP）

## 时序流程

### 会话建立

upload_start → upload_accept（含会话参数）

### UDP 探测

UDP_HELLO → UDP_HELLO_ACK → udp_ready

### 数据传输

Client 连续发送 UDP chunk；Server 乱序随机写盘

### 缺失统计与重传

Server 下发 missing_chunks；Client 按指令重传

### 校验与提交

Server 校验 hash、原子重命名、upload_complete

### 中断与恢复

resume_session → missing_chunks

### 下载对称

控制面一致，仅 UDP 方向互换

## 状态机

状态机与合法转移以 10-state-machine 为准
