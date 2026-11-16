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
| missing_chunks | 双向 | 缺失 chunk 列表报告 / 索要 |
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
| upload_complete | Server→Client | 上传完成（接收方 Server 上报） |
| download_complete | Client→Server | 下载完成（接收方 Client 上报） |
| session_abort | 双向 | 终止会话 |

### 资产撤回类

| 消息 | 方向 | 用途 |
|------|------|------|
| asset_recall | Client→Server | 请求撤回（删除）已上传的素材记录 |
| asset_recall_complete | Server→Client | 撤回处理结果回执 |

## 载荷格式

所有 MQTT 消息使用 JSON 格式，包含 `type` 字段标识消息类型。

**记号约定（规范性）**：下文 JSON 片段中的 `"u32"`、`"string"`、`"string|null"` 等是**类型标注**，
**不是线上取值**。线上实际取值见每条消息下方的「字段」表与示例。
例如 `"session_id": "u32"` 表示该字段线上是一个字符串，内容为 u32 的十进制形式（如 `"1001"`）。

### MQTT 服务属性（强制）

| 属性 | 要求 |
|------|------|
| MQTT 版本 | 3.1.1 或 5.0；Broker 须至少支持 3.1.1 |
| QoS | 控制面消息**必须**以 QoS 1 发布或订阅 |
| Retain | `status` 消息**必须**以 retain=true 发布；其余消息不得 retain |
| Last Will | Client **必须**在连接时设置遗嘱：遗嘱 Topic `/jiayi/fsp/server/{client_id}/status`，载荷 `{"type":"status","device_id":"...","status":"offline"}`，retain=true，QoS 1 |
| 持久会话 | Client 与 Server **必须**使用持久会话（MQTT 3.1.1 的 `clean_session=false` / MQTT 5 的 `Session Expiry Interval > 0`） |
| 最大报文长度 | Broker 允许的最大报文长度**必须** ≥ 1 MiB；`missing_chunks` 载荷受此约束（见 18 章） |
| 订阅过滤 | 见 22 章 |

**QoS 1 与幂等**：控制面消息以 QoS 1 传输，可能重复投递。所有控制面消息**必须**是幂等的
—— 重复投递不得改变协议语义（见本章「重要约束」第 4 条）。

**持久会话与断线重连**：持久会话保证 Client 断线期间 Server 下发的消息不丢失，
重连后由 Broker 补发。Client 重连后**必须**重新发布 `login` 完成重新认证，
并通过 `resume_session` 恢复会话（见 13 章）。

**遗嘱与在线状态**：`status` 的 `offline` 值由 Broker 在连接异常断开时代替 Client 发布
（遗嘱机制），Client 正常退出时主动发布 `status: offline` 并断开连接。

### login

```json
{
  "type": "login",
  "proto_version": "string",
  "device_id": "string",
  "username": "string|null"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| proto_version | string | Client 支持的协议版本，取 `"major.minor.patch"` 形式；当前为 `"1.1.0"` |
| device_id | string | 设备标识；**必须**与 Topic 中的 `{client_id}` 一致（见 23 章） |
| username | string\|null | 用户名；纯设备场景可为空 |

### login_ack

```json
{
  "type": "login_ack",
  "result": "success|fail",
  "code": "string|null",
  "proto_version": "string",
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

| 字段 | 类型 | 说明 |
|------|------|------|
| result | string | `success` 或 `fail` |
| code | string\|null | 失败时的机器可判定错误码，取值见 19 章（如 `LOGIN_FAILED` / `PROTO_VERSION_UNSUPPORTED` / `CLIENT_ID_INVALID`）；成功时为 null |
| proto_version | string | Server 选定的协议版本；Client 不支持该版本时**必须**中止会话 |
| message | string\|null | 失败原因的可读描述；机器可判定的错误码见 `code` 字段（19 章） |
| user | object | 用户信息；`user_id` / `username` / `device_id` / `status` / `last_login_time` / `created_time` |

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
  "tcp_port": "u16",
  "chunk_size": "u32",
  "file_exists": "bool"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| file_hash | string | 回显请求中的文件哈希，Client 据此关联 session_id 与文件 |
| session_id | string | Server 生成的会话 ID |
| chunk_count | u32 | 总 chunk 数量 = ceil(size / chunk_size) |
| window | u32 | 滑动窗口大小（**在途上限**），由 Server 下发；**Client 不得擅自修改**；**规范默认 4096**（见 11 章） |
| udp_port | u16 | 数据面 UDP 端口；Client **必须**以该端口作为数据面目的端口 |
| tcp_port | u16 | TCP 降级通道端口；**0 表示不支持 TCP 降级** |
| chunk_size | u32 | 协商确定的 chunk 大小 |
| file_exists | bool | 秒传标志：Server 侧已存在相同 file_hash 时为 true，Client 可跳过上传（见 10 章秒传路径） |

### missing_chunks

```json
{
  "type": "missing_chunks",
  "mode": "report|request",
  "session_id": "string",
  "missing_count": "u32|null",
  "chunks": ["u32"]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| mode | string | `report`（缺省值）或 `request`，见下表 |
| session_id | string | 会话 ID |
| missing_count | u32\|null | 发送方**尚未补齐**的缺失块**总数**；`mode=request` 时为 0。`chunks` 可能只是其中**一批**（单条上限 4096 条，见 18 章），因此 `missing_count` 可能大于 `chunks` 的长度。**null 表示未知** —— v1.0.0 对端不带此字段 |
| chunks | array | **仅 `mode=report` 时有意义**：发送方**自己尚未收到**的 chunk_id 列表；`mode=request` 时**必须**为空数组；单条**不得超过 4096 条**（见 18 章） |

**`chunks` 的语义（规范性）**：`chunks` 恒为「**发送方自己尚未收到的块**」，
而不是「我已有的块」。

- 上传方向 Server→Client：Server 报告「我还缺这些块」，Client 收到后**重传**这些块。
- 下载方向 Client→Server：Client 报告「我还缺这些块」，Server 收到后**重传**这些块。

两种方向下语义一致：**接收方报告缺失，发送方补发**。

| mode | 语义 | chunks | 收到方的处理 |
|------|------|--------|--------------|
| `report` | 发送方报告自己尚未收到的 chunk | 非空或空数组 | 按 `chunks` 重传这些块；`chunks` 为空数组表示「我这边没有缺失，可继续推进」 |
| `request` | 发送方索要对端的缺失列表 | **必须**为空数组 | 对端**必须**在 50ms 内以 `mode=report` 的 `missing_chunks` 应答 |

**按会话方向展开**（`direction` 见 09 章）：

| direction | `mode=report` 的发布方 | `chunks` 的含义 | 收到方的处理 |
|-----------|------------------------|-----------------|--------------|
| `upload` | **Server**（接收方） | Server 尚未收到的块 | Client 重传这些块 |
| `upload` | **Client**（发送方） | Client 尚未收到的块 —— 上传时恒为空 | Server 按裁决推进 |
| `download` | **Client**（接收方） | Client 尚未收到的块 | Server 重传这些块 |
| `download` | **Server**（发送方） | Server 尚未收到的块 —— 下载时恒为空 | Client 按裁决推进 |

发送方（上传方向 Client / 下载方向 Server）**永远不会缺块**，
因此它的 `mode=report` 载荷中 `chunks` 恒为空数组、`missing_count` 恒为 0。
发送方要了解对端的缺失情况，**必须**使用 `mode=request` 索要。

**缺省兼容（规范性）**：`mode` 缺失时按 `report` 处理；
`missing_count` 缺失时按 `null`（未知）处理。
这使得 v1.0.0 的实现（不带 `mode` 与 `missing_count`，直接携带缺失列表）
与 v1.1.0 互通时行为正确 —— `chunks` 始终是权威的重传依据，
`missing_count` 仅用于判断「是否还有后续批次」，未知时按「可能还有」处理。

**裁决即推进（规范性）**：接收方**必须**周期性下发 `mode=report` 的 `missing_chunks`
（含空数组），发送方据此推进窗口。详见 11 章「窗口推进判据」。

> **为什么需要 `request`**：上传方向的发送方（Client）无法自行表达「我还缺什么」——
> 它并不缺任何块。`request` 是它向接收方（Server）索要缺失裁决的唯一途径，
> 也是 12 章保底轮询与看门狗的载体。

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
  "missing_count": "u32|null",
  "udp_port": "u16"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| session_id | string | 会话 ID |
| next_chunk_id | u32 | **续传顺序起点**：Server 位图中**第一个尚未收到的 chunk_id**；位图已全真时为 `chunk_count` |
| missing_count | u32 | Server 尚未收到的 chunk 总数 |
| udp_port | u16 | 数据面 UDP 端口，可能因会话重建而变化，**Client 必须使用回执中的端口** |

> **`next_chunk_id` 不是 `max_received_id + 1`**（v1.0.0 草案的错误定义）。
> 本协议显式支持乱序写与随机访问，位图可能出现「空洞」——即存在低于最大已收 id 的缺失块。
> 若以 `max_received_id + 1` 作为续传起点，**空洞块将永远不会被重传**，
> 文件级 Blake3 校验必然失配。正确的续传起点是**第一个缺失块**。
> 空洞块的精确集合以周期下发的 `missing_chunks`（`mode=report`）为准，
> 两者互补：前者决定顺序发送流从哪里继续，后者决定哪些块需要补发。

### progress_update

```json
{
  "type": "progress_update",
  "session_id": "string",
  "received": "u64",
  "total": "u64"
}
```

`received` / `total` 的单位是 **chunk 数量**，不是字节数。
Server 约每 500ms 推送一次；Client 依据二者差值估算丢包率并驱动 AIMD 流控（见 11 章）。

`progress_update` 与 `missing_chunks` 是**两种不同的周期消息**，不得混淆：

| 消息 | 周期 | 作用 | 方向 |
|------|------|------|------|
| `missing_chunks`（`mode=report`） | **50ms** | 缺失裁决，驱动窗口推进 | 接收方 → 发送方 |
| `progress_update` | 500ms | 进度统计，供 AIMD 估算丢包率 | 双向 |

### udp_ready

```json
{
  "type": "udp_ready",
  "session_id": "string",
  "udp_port": "u16"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| session_id | string | 会话 ID |
| udp_port | u16 | 数据面 UDP 端口；会话重建后端口可能变化，**Client 必须以本消息中的端口为准** |

### upload_complete

```json
{
  "type": "upload_complete",
  "session_id": "string",
  "result": "string",
  "code": "string|null",
  "file_hash": "string|null",
  "asset_id": "string|null"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| session_id | string | 会话 ID |
| result | string | `success` 或 `fail` |
| code | string\|null | 失败时的机器可判定错误码，取值见 19 章；成功时为 null |
| file_hash | string\|null | 与其他消息一致，为 **Blake3 十六进制串**（或 null） |
| asset_id | string\|null | Server 为本次素材分配的记录 ID，用于后续 `asset_recall`；未建立记录时为 null |

> v0.2 草案写作 `"u64|null"`，与全文的字符串哈希定义冲突，已在 v1.0.0 更正。

### session_pause

```json
{
  "type": "session_pause",
  "session_id": "string"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| session_id | string | 会话 ID |

**语义（规范性）**：暂停会话，双方停止发送数据面数据包，
会话转入 `PAUSED` 状态；位图与临时文件全部保留。

- **无独立 ack**：`session_pause` 本身即状态转移消息，
  对端转入 `PAUSED` 后**必须**停止发送数据面数据包；
  恢复见 10 章「暂停与恢复」。
- **不表示「切换」**：重复发布是幂等的（已处于 `PAUSED` 时不再产生状态变化）。
  恢复一律通过 `resume_session` / `resume_ack` 表达。
- 与 `session_abort` 的区别：暂停**可恢复**，中止**不可恢复**。
- 与 `resume_session` 的区别：`session_pause` 是**显式暂停**，
  `resume_session` 是**异常中断后的恢复**；两者的恢复动作相同。

### session_abort

```json
{
  "type": "session_abort",
  "session_id": "string",
  "reason": "string",
  "detail": "string|null"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| session_id | string | 会话 ID |
| reason | string | 机器可判定的错误码，取值见 19 章（如 `FILE_HASH_MISMATCH`）；**不得**填写自由文本 |
| detail | string\|null | 可读补充说明，仅用于日志与 UI，不得被用于逻辑判断 |

**错误码上线路（规范性）**：19 章定义的错误码**必须**通过本消息的 `reason` 字段上线路。
发送方与接收方**必须**按错误码做分支判断，不得解析自由文本。

`result` 字段（`success` / `fail`）用于「完成类」消息；
`code` 字段（19 章错误码）用于「结果可能成功也可能失败」的消息；
`reason` 字段（19 章错误码）用于「终止类」消息。
三者语义不重叠，实现**不得**混用。

### asset_recall

```json
{
  "type": "asset_recall",
  "file_hash": "string",
  "asset_id": "string|null"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| file_hash | string | 要撤回的素材的 Blake3 十六进制串 |
| asset_id | string\|null | 素材记录 ID；为 null 时按 `file_hash` 定位 |

资产撤回请求：Client 请求 Server 删除已接收的素材记录。
该消息不关联 session，可独立于传输会话使用。

**`asset_id` 语义**：Server 为每条已接收素材分配的**记录标识**，由 Server 分配并在
`asset_recall_complete` 中回填（新建素材时经 `upload_complete.asset_id` 交付）。
Client 可用 `file_hash` 单独定位，也可用 `asset_id` 精确定位到某一条记录。

**撤回语义**：删除的是 Server 侧的**素材记录**（索引条目及其引用），
不删除已被其他会话引用的物理文件；物理文件的回收策略由实现决定。

**幂等**：对同一 `file_hash` / `asset_id` 重复撤回**必须**是幂等的 ——
第二次撤回若记录已不存在，应回 `deleted = false` 且 `code = null`
（视作「已处于目标状态」），**不得**报错。

**权限**：Server **必须**校验发起方的 `client_id` 对该素材记录的所有权，
越权时回 `code = ACL_DENY`（见 19 章）。

### asset_recall_complete

```json
{
  "type": "asset_recall_complete",
  "file_hash": "string",
  "asset_id": "string|null",
  "deleted": "bool",
  "code": "string|null",
  "detail": "string|null"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| file_hash | string | 被撤回素材的 Blake3 十六进制串 |
| asset_id | string\|null | 素材记录 ID；由 Server 分配，未建立记录时为 null |
| deleted | bool | 是否已删除素材记录 |
| code | string\|null | 未删除时的机器可判定错误码，取值见 19 章；成功删除时为 null |
| detail | string\|null | 可读补充说明，仅用于日志与 UI |

资产撤回完成通知：Server 处理撤回后的回执。
`deleted` 表达成功与否，`code` 表达失败原因，**不再重复提供语义重叠的 `result` 字段**。

### download_start

```json
{
  "type": "download_start",
  "session_id": "string|null",
  "file_hash": "string",
  "file_name": "string",
  "size": "u64",
  "chunk_size": "u32"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| session_id | string\|null | **可为空**；由 Server 生成并在 download_accept 中回填（与 upload_start 一致） |
| file_hash | string | 要下载的文件的 Blake3 十六进制串 |
| file_name | string | 文件名 |
| size | u64 | 文件大小（字节） |
| chunk_size | u32 | 期望的 chunk 大小（字节） |

### download_accept

```json
{
  "type": "download_accept",
  "file_hash": "string",
  "session_id": "string",
  "chunk_count": "u32",
  "window": "u32",
  "udp_port": "u16",
  "tcp_port": "u16",
  "chunk_size": "u32"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| file_hash | string | 回显请求中的文件哈希 |
| session_id | string | Server 生成的会话 ID |
| chunk_count | u32 | 总 chunk 数量 = ceil(size / chunk_size) |
| window | u32 | 滑动窗口大小（**在途上限**），由 Server 下发；**Client 不得擅自修改**；**规范默认 4096**（见 11 章） |
| udp_port | u16 | 数据面 UDP 端口；Server 向该端口推送 chunk，Client **必须**以它作为数据面端口 |
| tcp_port | u16 | TCP 降级通道端口；**0 表示不支持 TCP 降级** |
| chunk_size | u32 | 协商确定的 chunk 大小；Client **必须**用它核算 `chunk_count` |

**与 `upload_accept` 的字段对称性（规范性）**：`upload_accept` 与 `download_accept` **必须**携带
同构的字段集合 —— `file_hash` / `session_id` / `chunk_count` / `window` /
`udp_port` / `tcp_port` / `chunk_size`；仅 `upload_accept` 额外携带 `file_exists`。
数据面端口与 `chunk_size` **必须**出现在两个 accept 消息中：
缺端口则数据通路无法建立（Server 不得向未知端口发送 UDP，见 20 章），
缺 `chunk_size` 则接收方无法核算 `chunk_count = ceil(size / chunk_size)`。

### download_complete

```json
{
  "type": "download_complete",
  "session_id": "string",
  "result": "string",
  "code": "string|null",
  "file_hash": "string|null"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| session_id | string | 会话 ID |
| result | string | `success` 或 `fail` |
| code | string\|null | 失败时的机器可判定错误码，取值见 19 章；成功时为 null |
| file_hash | string\|null | 回显本次传输的文件哈希 |

**完成消息由接收方发布（规范性）**：

- 上传方向接收方是 Server → Server 发布 `upload_complete`；
- 下载方向接收方是 Client → Client 发布 `download_complete`。

原因：只有接收方持有位图与临时文件，能完成「校验 hash → 原子重命名」，
也只有接收方能判定完成与否。发送方通过收到该消息得知会话已提交。

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
3. **Client 发送 upload_start / download_start 时 session_id 可为空**，由 Server 生成并在 accept 中回填
4. **未识别消息必须忽略，保持幂等性**；QoS 1 的重复投递**不得**改变协议语义
5. **Client 仅订阅 `/jiayi/fsp/server/{client_id}/#`，发布到 `/jiayi/fsp/client/{client_id}/{action}`**
6. **Server 订阅 `/jiayi/fsp/client/+/#`，发布到 `/jiayi/fsp/server/{client_id}/{action}`**
7. **完成消息由接收方发布**：`upload_complete` 由 Server 发布，`download_complete` 由 Client 发布
8. **`missing_chunks.chunks` 恒为「发送方自己尚未收到的块」**；请求对端缺失列表时用 `mode=request`
9. **错误码必须上线路**：19 章的错误码通过 `session_abort.reason` 与完成类消息的 `code` 字段承载
10. **数据面端口必须出现在 accept 消息中**：`upload_accept` 与 `download_accept` **必须**携带 `udp_port`

> Topic 固定 5 层。v0.2 草案中 Server 的订阅过滤器只写了 4 层，匹配不到任何消息，
> 已在 v1.0.0 更正。详见 22 章「订阅与发布约束」。

## 与状态机关系

- 映射以 10-state-machine 为准
- 状态转换必须符合合法状态转移规则
