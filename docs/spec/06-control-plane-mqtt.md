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

### 资产撤回类

| 消息 | 方向 | 用途 |
|------|------|------|
| asset_recall | Client→Server | 请求撤回（删除）已上传的素材记录 |
| asset_recall_complete | Server→Client | 撤回处理结果回执 |

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

| 字段 | 类型 | 说明 |
|------|------|------|
| session_id | string\|null | 可为空；由 Server 生成并在 upload_accept 中回填 |
| file_hash | string | **Blake3 哈希的 64 字符十六进制串**（小写） |
| file_name | string | 文件名 |
| size | u64 | 文件大小（字节） |
| mtime | u64 | 文件修改时间戳（UTC 秒）；Server 依此记录素材时间属性 |
| chunk_size | u32 | 期望的 chunk 大小（字节），建议 1024–1350 |

### upload_accept

```json
{
  "type": "upload_accept",
  "file_hash": "string",
  "session_id": "string",
  "chunk_count": "u32",
  "window": "u32",
  "udp_port": "u16",
  "chunk_size": "u32",
  "file_exists": "bool"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| file_hash | string | 回显请求中的文件哈希，Client 据此关联 session_id 与文件 |
| session_id | string | Server 生成的会话 ID |
| chunk_count | u32 | 总 chunk 数量 = ceil(size / chunk_size) |
| window | u32 | 滑动窗口大小（初始值），Client 可按流控动态调整 |
| udp_port | u16 | 数据面 UDP 端口 |
| chunk_size | u32 | 协商确定的 chunk 大小 |
| file_exists | bool | 秒传标志：Server 侧已存在相同 file_hash 时为 true，Client 可跳过上传 |

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

> `received` / `total` 的单位是 **chunk 数量**，不是字节数。
> Server 约每 500ms 推送一次；Client 依据二者差值估算丢包率并驱动 AIMD 流控（见 11 章）。

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
  "file_hash": "string|null"
}
```

> `file_hash` 与其他消息一致，为 **Blake3 十六进制串**（或 null）。
> v0.2 草案写作 `"u64|null"`，与全文的字符串哈希定义冲突，已在 v1.0.0 更正。

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

### asset_recall

```json
{
  "type": "asset_recall",
  "file_hash": "string",
  "asset_id": "string|null"
}
```

> 资产撤回请求：Client 请求 Server 删除已接收的素材记录。
> 该消息不关联 session，可独立于传输会话使用。

### asset_recall_complete

```json
{
  "type": "asset_recall_complete",
  "file_hash": "string",
  "asset_id": "string|null",
  "deleted": "bool",
  "result": "string"
}
```

> 资产撤回完成通知：Server 处理撤回后的回执。

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

1. **session_id**: u32 类型，在 JSON 中以**字符串**形式传输（`"1001"`），由 Server 生成
   - v1.0.0 草案示例曾写作 JSON 数字，已更正为字符串，与参考实现 `u32_string` serde 一致
2. **file_hash**: 一律为 **Blake3 哈希的 64 字符十六进制小写串**，禁止用 u64 承载
3. **Client 发送 upload_start 时 session_id 可为空**
4. **未识别消息必须忽略，保持幂等性**
5. **Client 仅订阅 `/jiayi/fsp/server/{client_id}/#`，发布到 `/jiayi/fsp/client/{client_id}/{action}`**
6. **Server 订阅 `/jiayi/fsp/client/+/#`，发布到 `/jiayi/fsp/server/{client_id}/{action}`**

> Topic 固定 5 层。v0.2 草案中 Server 的订阅过滤器只写了 4 层，匹配不到任何消息，
> 已在 v1.0.0 更正。详见 22 章「订阅与发布约束」。

## 与状态机关系

- 映射以 10-state-machine 为准
- 状态转换必须符合合法状态转移规则
