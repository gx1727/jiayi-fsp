# 19 错误码（Error Codes）

## 分类

| 分类 | 说明 |
|------|------|
| SESSION | 会话级错误 |
| TRANSPORT | 传输层错误（UDP/TCP） |
| INTEGRITY | 完整性校验错误 |
| AUTH | 认证与 ACL 错误 |
| TIMEOUT | 超时错误 |
| STORAGE | 持久化与文件系统错误 |
| PROTOCOL | 协议格式与状态错误 |

## 错误码上线路（规范性）

本章的错误码**必须**通过控制面消息上线路，字段映射如下：

| 消息 | 字段 | 语义 |
|------|------|------|
| `session_abort` | `reason` | 终止原因，**必填**，取值来自本章 |
| `session_abort` | `detail` | 可读补充说明，仅用于日志与 UI，**不得**用于逻辑判断 |
| `upload_complete` | `code` | 失败时的错误码；成功时为 null |
| `download_complete` | `code` | 失败时的错误码；成功时为 null |
| `asset_recall_complete` | `code` | 未删除时的错误码；成功删除时为 null |

- 发送方与接收方**必须**按错误码做分支判断，**不得**解析 `detail` 自由文本。
- `result` 字段只表达「成功 / 失败」，**不承载**错误码；
  `code` / `reason` 字段**必须**是本章中定义过的标识符。
- **未列在本章的错误码**一律视为 `UNKNOWN_ERROR`。

## 建议错误码

### SESSION

- `SESSION_NOT_FOUND`：会话不存在或已终止
- `SESSION_ABORTED`：会话被服务端终止
- `INVALID_STATE`：当前状态不允许该操作

### TRANSPORT

- `UDP_UNAVAILABLE`：UDP 探测失败
- `TCP_ERROR`：TCP 降级通道错误
- `BIND_FAILED`：数据面端口绑定失败
- `CONNECTION_FAILED`：控制面连接失败
- `NETWORK_ERROR`：网络错误

### INTEGRITY

- `CRC32_INVALID`：包级校验失败（丢弃该包，不标记位图）
- `FILE_HASH_MISMATCH`：文件级内容哈希不一致（**必须**以 `session_abort` 终止会话，见 12 章）
- `INVALID_CHUNK`：chunk_id 越界或非法
- `CHUNK_ALREADY_RECEIVED`：重复 chunk（幂等丢弃）

### AUTH

- `ACL_DENY`：ACL 拒绝访问
- `CLIENT_ID_INVALID`：client_id 非法
- `LOGIN_FAILED`：登录认证失败
- `PROTO_VERSION_UNSUPPORTED`：协议版本不受支持（见 06 章 `login` / `login_ack`）

### TIMEOUT

- `TIMEOUT_NO_ACTIVITY`：长时间无活动导致超时
- `TIMEOUT_PROBE`：UDP 探测超时

### STORAGE

- `DATABASE_ERROR`：SQLite 持久化失败
- `FILESYSTEM_ERROR`：文件系统操作失败

### PROTOCOL

- `INVALID_PACKET`：数据包格式非法（长度不足、越界等）
- `SESSION_ID_MISMATCH`：session_id 不匹配
- `FILE_ID_MISMATCH`：file_hash 不匹配
- `SERIALIZATION_ERROR`：JSON 编解码失败
- `CONTROL_ERROR`：控制面错误
- `UNKNOWN_ERROR`：未分类错误

## 行为

- 错误记录日志
- INTEGRITY 错误包直接丢弃，且**不得**写入位图
- SESSION 错误触发 `session_abort`（`reason` 填对应错误码）
- CHUNK_ALREADY_RECEIVED 属幂等情形，静默丢弃即可
- 同一错误不得导致状态机进入非法状态
