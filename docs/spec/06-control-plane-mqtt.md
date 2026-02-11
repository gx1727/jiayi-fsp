# 06 控制面（MQTT）

控制面是会话状态推进与可靠性裁决的唯一通道。

## Topic 规范

- **命名格式**: `/jiayi/fsp/{role}/{client_id}/{action}`
  - `role`: `client` 或 `server`
  - `client_id`: 设备唯一标识符
  - `action`: 消息类型

- **Client → Server**: `/jiayi/fsp/client/{client_id}/{action}`
- **Server → Client**: `/jiayi/fsp/server/{client_id}/{action}`

## 消息类型

### 认证类

| 消息 | 方向 | 用途 |
|------|------|------|
| login | Client→Server | 设备登录认证 |
| login_ack | Server→Client | 登录响应 |
| status | 双向 | 设备在线状态报告 |

### 会话建立

| 消息 | 方向 | 用途 |
|------|------|------|
| upload_start | Client→Server | 发起上传 |
| upload_accept | Server→Client | 接受上传，返回会话参数 |
| download_start | Client→Server | 发起下载 |
| download_accept | Server→Client | 接受下载，返回会话参数 |

### 传输控制

| 消息 | 方向 | 用途 |
|------|------|------|
| udp_ready | Server→Client | UDP 可用性通知 |
| missing_chunks | 双向 | 缺失 chunk 列表 |
| progress_update | 双向 | 传输进度更新 |
| session_pause | 双向 | 暂停会话 |

### 会话管理

| 消息 | 方向 | 用途 |
|------|------|------|
| resume_session | Client→Server | 恢复中断的会话 |
| resume_ack | Server→Client | 恢复确认 |

### 终止类

| 消息 | 方向 | 用途 |
|------|------|------|
| upload_complete | Server→Client | 上传完成 |
| download_complete | 双向 | 下载完成 |
| session_abort | 双向 | 终止会话 |

## 载荷格式

所有 MQTT 消息使用 JSON 格式，包含 `type` 字段标识消息类型。

### login

```json
{
  "type": "login",
  "device_id": "string",
  "username": "string|null"
}
```

### login_ack

```json
{
  "type": "login_ack",
  "result": "success|fail",
  "message": "string|null",
  "user": {
    "user_id": "i64",
    "username": "string",
    "device_id": "string",
    "status": "string",
    "last_login_time": "i64",
    "created_time": "i64"
  }
}
```

### upload_start

```json
{
  "type": "upload_start",
  "session_id": "string|null",
  "file_hash": "string",
  "file_name": "string",
  "size": "u64",
  "mtime": "u64",
  "chunk_size": "u32"
}
```

### upload_accept

```json
{
  "type": "upload_accept",
  "file_hash": "string",
  "session_id": "string",
  "chunk_count": "u32",
  "window": "u32",
  "udp_port": "u16",
  "chunk_size": "u32"
}
```

### missing_chunks

```json
{
  "type": "missing_chunks",
  "session_id": "string",
  "chunks": ["u32"]
}
```

### resume_session

```json
{
  "type": "resume_session",
  "session_id": "string"
}
```

### resume_ack

```json
{
  "type": "resume_ack",
  "session_id": "string",
  "next_chunk_id": "u32",
  "udp_port": "u16"
}
```

### progress_update

```json
{
  "type": "progress_update",
  "session_id": "string",
  "received": "u64",
  "total": "u64"
}
```

### udp_ready

```json
{
  "type": "udp_ready",
  "session_id": "string"
}
```

### upload_complete

```json
{
  "type": "upload_complete",
  "session_id": "string",
  "result": "string",
  "file_hash": "u64|null"
}
```

### session_pause

```json
{
  "type": "session_pause",
  "session_id": "string"
}
```

### session_abort

```json
{
  "type": "session_abort",
  "session_id": "string",
  "reason": "string"
}
```

### download_start

```json
{
  "type": "download_start",
  "session_id": "string",
  "file_hash": "string",
  "file_name": "string",
  "size": "u64",
  "chunk_size": "u32"
}
```

### download_accept

```json
{
  "type": "download_accept",
  "session_id": "string",
  "chunk_count": "u32",
  "window": "u32",
  "tcp_port": "u16"
}
```

### download_complete

```json
{
  "type": "download_complete",
  "session_id": "string",
  "result": "string"
}
```

### status

```json
{
  "type": "status",
  "device_id": "string",
  "status": "online|offline"
}
```

## 重要约束

1. **session_id**: u32 类型，在 JSON 中以字符串形式传输，由 Server 生成
2. **Client 发送 upload_start 时 session_id 可为空**
3. **未识别消息必须忽略，保持幂等性**
4. **Client 仅订阅 `/jiayi/fsp/server/{client_id}/#`，发布到 `/jiayi/fsp/client/{client_id}/{action}`**
5. **Server 订阅 `/jiayi/fsp/client/+`，发布到 `/jiayi/fsp/server/{client_id}/{action}`**

## 与状态机关系

- 映射以 10-state-machine 为准
- 状态转换必须符合合法状态转移规则
