# jiayi-fsp 协议规范合集

> **本文件由 `tools/build-spec-compilation.py` 从 `docs/spec/` 自动生成，请勿手动编辑。**
>
> 单一事实来源是 `docs/spec/` 下的分章节文件。本文件仅供整体阅读与归档分发；
> 若存在差异，一律以 `docs/spec/` 的分章节原文件为准。
>
> 协议版本：v1.1.0

---


## 00 概览（Overview）

jiayi-fsp 是面向可信局域网的混合文件同步协议。通过"控制面 MQTT + 数据面 UDP"的双通道协作，在保证高吞吐的同时，实现可中断、可恢复、可裁决的文件同步能力。

### 核心特性

- 局域网内大文件（照片/视频）高效同步
- 不中断可恢复、乱序写盘、内存与文件大小解耦
- 控制与数据分离，可靠性由控制面裁决
- 控制面：MQTT；数据面：UDP；位图进度；会话恢复；NAT 探测

### 角色与视角

| 视角 | 定义 | 上传方向 | 下载方向 |
|------|------|----------|----------|
| 发送侧 | 发送数据的一方 | Client | Server |
| 接收侧 | 持有位图与临时文件的一方 | Server | Client |

- 完成消息一律由**接收方**发布
- 缺失报告一律由**接收方**发起
- Server 拥有最终的会话终止权

详见 10 章「视角定义」。

### 文档约定（规范性）

- **规范性要求**（含「必须」「不得」「应当」）**必须**写在正文，不得藏在引用块中。
- **引用块（`>`）仅用于历史变更说明**（例如「v1.0.0 草案写作 X，已在 v1.1.0 更正为 Y」）。
  引用块内容不承载规范性效力。
- **非规范性内容**（实现建议、背景说明）**必须**显式标注为「非规范性（Non-Normative）」
  或「推荐性」，见 25、26 章。
- 规范性用语的强度：
  - **必须 / 不得 / 应当**：强制要求，违反即为实现错误；
  - **建议 / 推荐**：强烈建议，偏离时**应当**说明理由；
  - **可以**：允许但不要求。
- `tools/check-spec-consistency.py` 会自动校验上述约定（引用块规范性守卫等）。

### 版本

- 协议版本：**v1.1.0**
- 状态：规范草案修订中（Draft），尚未冻结

> 本版本对齐参考实现 `jiayi-fsp-core` 0.2.0 及 `jiayi-client` 1.0.0。
> v1.0.0 相对 v0.2 的主要变更：统一端序为小端、载荷字段以参考实现为基准、
> 新增 `asset_recall` 资产撤回扩展、纳入 AIMD 动态流控与保活/看门狗机制。
>
> **v1.1.0 相对 v1.0.0 的主要变更**（详见 CHANGELOG）：
> 补全控制面节奏规范（裁决周期、窗口推进判据、全部超时阈值）、
> 定义 `missing_chunks` 的 `chunks` 语义并引入 `mode` 判别字段、
> 修正 `resume_ack.next_chunk_id` 定义、重定义状态机的发送侧/接收侧视角、
> 统一完成消息的发布方、明确 MQTT 服务属性、固定 CRC32 变体。

---

## 01 架构（Architecture）

### 分层架构

应用层 → 控制层（MQTT） → 数据层（UDP/TCP） → 网络层（LAN）。

### 各层职责

#### 控制面

- 登录认证
- 会话建立/协商
- 缺失统计与重传裁决
- UDP 可用性探测
- 完成与终止
- 在线状态管理
- 进度通知

#### 数据面

- Chunk 传输
- CRC32 校验
- 随机写盘与位图

### 角色与视角

协议区分两个角色视角（详见 10 章「视角定义」）：

| 视角 | 定义 | 上传方向 | 下载方向 |
|------|------|----------|----------|
| **发送侧** | 发送数据的一方 | Client | Server |
| **接收侧** | 持有位图与临时文件的一方 | Server | Client |

- **完成消息一律由接收方发布**：`upload_complete` 由 Server 发布，
  `download_complete` 由 Client 发布。
- **缺失报告一律由接收方发起**：`missing_chunks`（`mode=report`）由接收方下发，
  发送方据此补发。

### 原则

- 状态机仅由控制面推进；数据面只更新位图，不直接触发状态转移
- **Server 为最终裁决者**：任何进入 ABORTED 的转换只能由 Server 发起
- 控制面拥有最终裁决权；数据面仅负责搬运与落盘

---

## 02 术语（Terminology）

### 基础术语

- `session_id`（u32）：会话唯一标识
- `file_hash`（string）：文件哈希
- `file_name`（string）：文件名
- `size`（u64）：文件大小（字节）
- `mtime`（u64）：文件修改时间戳
- `chunk_size`（u32）：数据块大小（字节）
- `chunk_id`（u32）：数据块标识
- `payload_len`（u16）：载荷长度
- `flags`（u16）：标志位

### 状态与控制

- `bitmap`：接收进度位图
- `window`（u32）：滑动窗口大小（**在途上限**，Server 下发，Client 不得修改）
- `client_id`（device_id，string）：客户端设备 ID
- `missing_chunks`（Vec<u32>）：缺失数据块列表；语义为「**发送方自己尚未收到的块**」
- `mode`（string）：`missing_chunks` 的判别字段，`report`（报告缺失）/ `request`（索要缺失列表）
- `udp_ready`：UDP 就绪通知
- `resume_session`：会话恢复请求
- `progress_update`：传输进度更新
- `session_pause`：会话暂停请求

---

## 03 设计目标（Design Goals）

- 高效大文件同步
- 可恢复
- 乱序写盘
- 重传裁决清晰
- 内存与文件大小解耦
- 分层清晰
- 发送端策略简化与稳定

---

## 04 非目标（Non-Goals）

- 公网穿透与互联网规模传输
- 数据面的复杂拥塞控制与逐包 ACK
  - 拥塞调节仅以**发送端本地策略**（AIMD）作用于发送节奏，不进入协议层
- 多租户不可信网络
- 移动端长期后台无 UI 传输
- 端到端加密作为协议内建能力（由实现层提供，且不得改变 chunk 与偏移的映射）

---

## 05 协议栈（Protocol Stack）

### 分层结构

应用层 → 控制层（MQTT：状态机/重传裁决/会话管理） → 数据层（UDP/TCP：chunk 传输与兜底） → 网络层（LAN）。

### 职责划分

- 控制面拥有最终裁决权
- 数据面仅负责搬运与落盘
- **完成消息由接收方发布；缺失裁决由接收方发起**（见 10 章「视角定义」）

### 通道与成熟度

| 通道 | 方向 | 状态 |
|------|------|------|
| MQTT 控制面 | 双向 | **v1.1.0 已定义**（含服务属性与节奏语义） |
| UDP 数据面 | 双向 | **v1.1.0 已定义**（含 CRC32 变体与校验规则） |
| TCP 数据面 | 兜底 | **v1.1.0 仅定义语义，未定义线路格式**（见 07 章、ROADMAP） |

---

## 06 控制面（MQTT）

控制面是会话状态推进与可靠性裁决的唯一通道。

### Topic 规范

- **命名格式**: `/jiayi/fsp/{role}/{client_id}/{action}`
  - `role`: `client` 或 `server`
  - `client_id`: 设备唯一标识符
  - `action`: 消息类型

- **Client → Server**: `/jiayi/fsp/client/{client_id}/{action}`
- **Server → Client**: `/jiayi/fsp/server/{client_id}/{action}`

### 消息类型

#### 认证类

| 消息 | 方向 | 用途 |
|------|------|------|
| login | Client→Server | 设备登录认证 |
| login_ack | Server→Client | 登录响应 |
| status | 双向 | 设备在线状态报告 |

#### 会话建立

| 消息 | 方向 | 用途 |
|------|------|------|
| upload_start | Client→Server | 发起上传 |
| upload_accept | Server→Client | 接受上传，返回会话参数 |
| download_start | Client→Server | 发起下载 |
| download_accept | Server→Client | 接受下载，返回会话参数 |

#### 传输控制

| 消息 | 方向 | 用途 |
|------|------|------|
| udp_ready | Server→Client | UDP 可用性通知 |
| missing_chunks | 双向 | 缺失 chunk 列表报告 / 索要 |
| progress_update | 双向 | 传输进度更新 |
| session_pause | 双向 | 暂停会话 |

#### 会话管理

| 消息 | 方向 | 用途 |
|------|------|------|
| resume_session | Client→Server | 恢复中断的会话 |
| resume_ack | Server→Client | 恢复确认 |

#### 终止类

| 消息 | 方向 | 用途 |
|------|------|------|
| upload_complete | Server→Client | 上传完成（接收方 Server 上报） |
| download_complete | Client→Server | 下载完成（接收方 Client 上报） |
| session_abort | 双向 | 终止会话 |

#### 资产撤回类

| 消息 | 方向 | 用途 |
|------|------|------|
| asset_recall | Client→Server | 请求撤回（删除）已上传的素材记录 |
| asset_recall_complete | Server→Client | 撤回处理结果回执 |

### 载荷格式

所有 MQTT 消息使用 JSON 格式，包含 `type` 字段标识消息类型。

**记号约定（规范性）**：下文 JSON 片段中的 `"u32"`、`"string"`、`"string|null"` 等是**类型标注**，
**不是线上取值**。线上实际取值见每条消息下方的「字段」表与示例。
例如 `"session_id": "u32"` 表示该字段线上是一个字符串，内容为 u32 的十进制形式（如 `"1001"`）。

#### MQTT 服务属性（强制）

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

#### login

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

#### login_ack

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

#### upload_start

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

#### upload_accept

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

#### missing_chunks

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

#### resume_session

```json
{
  "type": "resume_session",
  "session_id": "string"
}
```

#### resume_ack

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

#### progress_update

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

#### udp_ready

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

#### upload_complete

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

#### session_pause

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

#### session_abort

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

#### asset_recall

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

#### asset_recall_complete

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

#### download_start

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

#### download_accept

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

#### download_complete

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

#### status

```json
{
  "type": "status",
  "device_id": "string",
  "status": "online|offline"
}
```

### 重要约束

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

### 与状态机关系

- 映射以 10-state-machine 为准
- 状态转换必须符合合法状态转移规则

---

## 07 数据面（UDP / TCP Fallback）

### UDP 原则

- 无连接、无逐包 ACK、允许丢包与乱序
- payload_len ≤ chunk_size（建议 chunk_size ≤ 1400，避免 IP 分片）
- crc32 包级校验

### 发送端

- **按批次发送**：连续发送 `burst_size` 个 chunk，批间隔 `batch_gap_us`
  （初值 2000µs；`burst_size` 初值 = `window`）。详见 11 章「发送节奏」。
  - v1.0.0 草案此处写作「间隔 2–5ms」（逐包读法），与 11 章的批量读法冲突，
    两者相差约 `window` 倍；本版本统一为**批量读法**。
- **仅对 `missing_chunks`（`mode=report`）报告的缺失块重传**，重传优先级高于新数据
- **数据面不得改变会话状态**，只能更新接收侧位图（见 10 章「推进规则」）

### 接收端

- 校验头与 crc32
- 随机写盘
- 标记位图
- 不直接推进状态机（位图变化只是接收侧状态推进的**输入**）

### UDP 可用性探测

#### 交互流程

1. Client 发送 UDP_PROBE
2. Server 返回 UDP_PROBE_ACK
3. Server 通过 MQTT 发送 udp_ready

> v1.0.0 草案的 24 章把同一握手写作 `UDP_HELLO` / `UDP_HELLO_ACK`，
> 本版本统一为 **`UDP_PROBE` / `UDP_PROBE_ACK`**。

#### PROBE 数据包

- flags.bit3=1
- payload 为 session_id（u32）的 **4 字节小端编码**
  - v0.2 草案曾写作"8 字节"，与 u32 的 session_id 定义矛盾，已在 v1.0.0 更正

#### PROBE_ACK 数据包

- flags.bit3=1
- payload 为 session_id + 1 字节状态码（0=成功，1=失败），共 **5 字节**
- **判别规则（规范性）**：PROBE 与 PROBE_ACK 共用 `flags.bit3`，
  接收方**必须**以 `payload_len` 区分：`payload_len == 4` 为 PROBE，
  `payload_len == 5` 为 PROBE_ACK。其余长度**必须**按 `INVALID_PACKET` 丢弃。

#### 端口与地址绑定（规范性）

- Server 的数据面 socket **必须**绑定 accept 消息中下发的 `udp_port`。
- Server **必须**以 PROBE 报文的**源地址:端口**作为该会话的对端数据面地址；
  Server 不得向未经 PROBE 确认的地址发送数据面数据包（见 20 章）。
- Server 返回的 PROBE_ACK **必须**从该会话的数据面端口发出，
  以保证客户端的 NAT 映射命中。

#### 超时

- Client 等待 PROBE_ACK 超时（建议 2s）则判定 UDP 不可用
- 回退到 TCP

### TCP 降级

- UDP 不可用或异常时启用
- 保持 chunk 与 bitmap 语义
- **TCP 通道的线路格式在 v1.1.0 中仍未定义**（见 26 章「下一阶段」）；
  本版本仅规定其**语义**要求：chunk 边界、bitmap 语义、状态机行为与 UDP 通道一致
- TCP 端口由 accept 消息的 `tcp_port` 字段下发；**`tcp_port = 0` 表示不支持 TCP 降级**，
  此时探测失败即判定传输不可用

---

## 08 UDP 数据包格式规范（Packet Format）

### 数据包结构（小端序 Little-Endian）

> **端序约定**：本协议所有多字节整数字段一律采用 **小端序（Little-Endian）**。
> v0.2 草案中"网络字节序，小端"的表述自相矛盾，已在 v1.0.0 中统一为小端；
> 参考实现 `jiayi-fsp-core` 采用 `to_le_bytes` / `from_le_bytes`，与本节一致。

| 字段 | 偏移 | 大小 | 类型 | 说明 |
|------|------|------|------|------|
| session_id | 0-3 | 4 | u32 | 会话唯一标识，由 Server 生成 |
| chunk_id | 4-7 | 4 | u32 | 逻辑位置（从 0 开始） |
| payload_len | 8-9 | 2 | u16 | 载荷长度（不含头部与 CRC） |
| flags | 10-11 | 2 | u16 | 标志位 |
| payload | 12.. | N | bytes | 文件数据 |
| crc32 | 12+N | 4 | u32 | CRC32 校验码（覆盖 header+payload） |

### 头部大小

```
HEADER_SIZE = 4 + 4 + 2 + 2 = 12 字节
CRC_SIZE = 4 字节
```

### 载荷长度上限

- `payload_len ≤ chunk_size`
- 建议 `chunk_size` 取 1024–1350 字节，**上限 1400 字节**，避免 IP 分片
- 参考实现默认 `chunk_size = 1200`
- **接收端必须校验**：`payload_len ≤ 1400` 且 `payload_len ≤ 本会话协商的 chunk_size`；
  越界即按 `INVALID_PACKET` 丢弃，**不得**写入位图
- 最后一个 chunk 的 `payload_len` **必须**等于 `size - chunk_id × chunk_size`；
  不等即按 `INVALID_PACKET` 丢弃

### CRC32 变体（规范性）

本协议的 `crc32` 字段使用 **CRC-32/ISO-HDLC**（又称 CRC-32、zlib `crc32`）：

| 参数 | 取值 |
|------|------|
| 宽度 | 32 |
| 多项式 | 0x04C11DB7（反射形式 0xEDB88320） |
| 初值 | 0xFFFFFFFF |
| 输入反射 | 是 |
| 输出反射 | 是 |
| 末尾异或 | 0xFFFFFFFF |
| 校验值 | `"123456789"` → 0xCBF43926 |

**覆盖区间（规范性）**：被校验的字节是数据包的 **第 0 字节到第 `12 + payload_len - 1` 字节**，
即 **12 字节头部 + payload**，**不含**尾部 4 字节 `crc32` 字段本身。

> v1.0.0 草案只写「CRC32」而未指定变体。CRC-32/ISO-HDLC 与 CRC-32C（Castagnoli，
> 多项式 0x1EDC6F41 / 反射 0x82F63B78）互不兼容：两侧各选一种会导致
> **所有包校验失败**，而按 12 章「校验失败直接丢弃」的策略，
> 该故障表现为「传输永远无法完成」，极难定位。本版本显式固定变体。

### Flags 位定义

| 位 | 常量 | 值 | 说明 |
|----|------|-----|------|
| bit0 | FLAG_DATA | 0x0001 | 数据包标志 |
| bit1 | FLAG_RESEND | 0x0002 | 重传包标志；接收方**无需**区别处理（重传是幂等的） |
| bit2 | FLAG_LAST_CHUNK | 0x0004 | 最后一个 chunk 标志，置位时该包的 `chunk_id` = `chunk_count - 1`；**可选**，接收方**不得**依赖它判断完成（完成以位图为准） |
| bit3 | FLAG_PROBE | 0x0008 | 探测包标志；PROBE 与 PROBE_ACK 共用，以 `payload_len` 区分（4 / 5） |
| bit4-15 | RESERVED | 0x0000 | 保留位；接收方**必须**忽略未知标志位 |

### 数据包示例

**普通数据包**:
```
session_id: 0x00000001
chunk_id: 0x00000010
payload_len: 1024
flags: 0x0001 (FLAG_DATA)
payload: [1024 bytes data]
crc32: [4 bytes checksum]
```

**探测包（PROBE）**:
```
session_id: 0x00000001
chunk_id: 0x00000000
payload_len: 4
flags: 0x0008 (FLAG_PROBE)
payload: [session_id in little-endian, 4 bytes]
crc32: [4 bytes checksum]
```

> 探测包 payload 为 `session_id`（u32）的 **4 字节小端编码**。
> v0.2 草案的 07 章曾写作"8 字节"，已在本版本更正为 4 字节。

**探测应答包（PROBE_ACK）**:
```
session_id: 0x00000001
chunk_id: 0x00000000
payload_len: 5
flags: 0x0008 (FLAG_PROBE)
payload: [session_id in little-endian, 4 bytes][status: 0x00]
crc32: [4 bytes checksum]
```

PROBE_ACK 的 payload 为 5 字节：4 字节 session_id + 1 字节状态码
（0 = 成功，1 = 失败）。
**PROBE 与 PROBE_ACK 共用 `flags.bit3`，以 `payload_len` 区分**（4 / 5）。

### 错误处理

任一关键字段不匹配或校验失败均丢弃。校验失败的数据包**不得**标记位图。

- `payload_len` 越界（> 1400 或 > 协商 chunk_size）→ `INVALID_PACKET`，丢弃
- `chunk_id` 越界（≥ chunk_count）→ `INVALID_CHUNK`，丢弃
- `session_id` 不匹配 → `SESSION_ID_MISMATCH`，丢弃
- CRC32 校验失败 → `CRC32_INVALID`，丢弃

### 扩展

- flags 预留位（bit4–15）；接收方**必须**忽略未知标志位
- 不兼容扩展通过 MQTT 协议版本协商：`login` 携带 `proto_version`，
  Server 在 `login_ack` 中回填选定的版本；Client 不支持该版本时**必须**中止会话
  （见 06 章）
- **版本比较规则（规范性）**：版本号形如 `"major.minor.patch"`，
  按 major → minor → patch 逐段**数值**比较。
  - Client 声明其**最高**支持的版本；Server **应当**回填两者都支持的**最高**版本。
  - Server 回填的版本其 major **必须**等于 Client 声明的 major
    —— **major 不同即不兼容**，Client **必须**中止会话，
    并以 `login_ack.code` = `PROTO_VERSION_UNSUPPORTED` 报告。
  - major 相同、minor 或 patch 不同即视为兼容：新增字段一律可选，
    接收方**必须**忽略不认识的字段。

---

## 09 会话模型（Session Model）

### 会话参数

| 字段 | 类型 | 说明 |
|------|------|------|
| session_id | u32 | 会话唯一标识，Server 生成 |
| client_id | String | 客户端设备 ID |
| direction | enum | `upload` 或 `download` —— **接收方 / 发送方角色** |
| file_hash | String | 文件哈希 |
| file_name | String | 文件名 |
| size | u64 | 文件大小（字节） |
| mtime | u64 | 文件修改时间戳 |
| state | SessionState | 会话状态（见 10 章） |
| chunk_size | u32 | Chunk 大小（字节） |
| window | u32 | 滑动窗口大小（**在途上限**） |
| chunk_count | u32 | 总 chunk 数量 = ceil(size / chunk_size) |
| udp_port | u16 | UDP 端口号 |
| tcp_port | u16 | TCP 端口号 |
| last_activity | u64 | 最后活动时间戳 |

**`direction` 字段是必需的（规范性）**：本协议的 `missing_chunks` 是**双向**消息，
其含义依赖会话方向（见 06 章）。Server 侧的会话记录**必须**能区分
「我是接收方」还是「我是发送方」，否则无法正确解释对端的缺失报告。

| direction | 接收方 | 发送方 | 完成消息发布方 |
|-----------|--------|--------|----------------|
| `upload` | Server | Client | Server（`upload_complete`） |
| `download` | Client | Server | Client（`download_complete`） |

### 创建规则

1. **session_id 由 Server 生成**
   - Server **必须**保证在自身生命周期内 `session_id` 唯一；
     冲突时**必须**重新生成而不是复用。
   - u32 的空间有限，实现**不得**使用裸时间戳低 32 位
     （秒级回绕 136 年、毫秒级回绕 49 天）。推荐使用
     「随机高位 + 递增低位」或 Snowflake（含机器位）等方式。
   - 冲突后果严重：`session_id` 同时是 MQTT 关联键与 UDP 头字段（见 08 章），
     串号会导致跨会话数据污染。
2. **同文件正在同步 → 返回现有会话**
   - 「同文件」的判定键为 `client_id + file_hash`（同一客户端的同一内容）。
   - 返回现有会话时，`upload_accept` **必须**复用既有的 `session_id`，
     并重新下发当前的 `window` / `chunk_size` / `udp_port`。
3. **已存在完整文件 → 秒传**
   - Server 侧已存在相同 `file_hash` 的完整文件时，`upload_accept.file_exists = true`，
     走 10 章定义的秒传路径（**不进入 TRANSFERRING，不产生位图**）。
4. **否则新建会话并生成新的 session_id**

### 持久化

- 临时文件、bitmap、元数据
- 位图使用 `AtomicU64` 数组存储，支持原子操作
- 支持保存到文件和从文件加载

### 超时与回收

- **会话活动超时**：**60 秒**内无任何控制面交互或数据面活动的会话，
  Server **必须**将其置为 ABORTED 并清理临时文件 / 位图 / 元数据。
- 客户端在超时前可通过 `resume_session` 恢复；超时后 `resume_session` **不合法**
  （ABORTED 是终态，见 10 章）。
- 该阈值对应 19 章的 `TIMEOUT_NO_ACTIVITY`。

---

## 10 Session 状态机

定义传输会话的状态与合法状态转移。控制面（MQTT）负责裁决与同步。

### 状态定义

```
INIT → NEGOTIATING → TRANSFERRING → VERIFY → COMMIT → COMPLETED
          │              │  ↑                        │
          │              │  └────────────────────────┘
          │              ↓
          │           PAUSED  ←─────────────────────────┘
          ↓
       ABORTED （从任意状态可到达）
```

| 状态 | 说明 |
|------|------|
| INIT | 初始化状态 - 会话刚创建 |
| NEGOTIATING | 协商状态 - 正在协商传输参数 |
| TRANSFERRING | 传输状态 - 正在收发数据 |
| PAUSED | 暂停状态 - 会话被显式暂停，位图与临时文件保留 |
| VERIFY | 验证状态 - 正在做文件级完整性校验 |
| COMMIT | 提交状态 - 正在原子提交（重命名） |
| COMPLETED | 完成状态 - 传输成功完成 |
| ABORTED | 中止状态 - 传输被终止；终态，无出边 |

### 视角定义（规范性）

同一个会话在**发送侧**与**接收侧**各自持有一份状态视图：

| 视角 | 定义 | 上传方向 | 下载方向 |
|------|------|----------|----------|
| **发送侧** | 发送数据的一方 | Client | Server |
| **接收侧** | 持有位图与临时文件的一方 | Server | Client |

- **接收侧**使用全部状态（INIT / NEGOTIATING / TRANSFERRING / PAUSED / VERIFY / COMMIT / COMPLETED / ABORTED）。
- **发送侧**只使用 INIT / NEGOTIATING / TRANSFERRING / PAUSED / COMPLETED / ABORTED ——
  发送侧不持有位图，**不存在** VERIFY 与 COMMIT 状态：文件级校验与原子提交
  天然发生在接收侧。
- 「Server 为最终裁决者」指**终止权**：任何进入 ABORTED 的转换只能由 Server 通过
  `session_abort` 发起（发送侧收到 `session_abort` 后同样转入 ABORTED）。

### 合法状态转移

**每个状态对每个视角都要有合法转移**。下表逐视角列出：

| 从状态 | 到状态 | 视角 | 触发 |
|--------|--------|------|------|
| INIT | NEGOTIATING | 发送侧 | upload_start / download_start |
| INIT | NEGOTIATING | 接收侧 | upload_start / download_start（收到传输请求） |
| INIT | ABORTED | 双方 | session_abort |
| NEGOTIATING | TRANSFERRING | 发送侧 | upload_accept / download_accept |
| NEGOTIATING | VERIFY | 接收侧 | 秒传：accept 携带 file_exists=true，跳过数据传输 |
| NEGOTIATING | COMPLETED | 发送侧 | 秒传：accept 携带 file_exists=true，不发送任何数据 |
| NEGOTIATING | ABORTED | 双方 | session_abort |
| TRANSFERRING | VERIFY | 接收侧 | 位图全真，进入文件级校验 |
| TRANSFERRING | PAUSED | 双方 | session_pause |
| TRANSFERRING | COMPLETED | 发送侧 | upload_complete / download_complete |
| TRANSFERRING | ABORTED | 双方 | session_abort |
| PAUSED | TRANSFERRING | 双方 | resume_session（发送侧发起）/ resume_ack（接收侧应答后） |
| PAUSED | ABORTED | 双方 | session_abort |
| VERIFY | COMMIT | 接收侧 | 文件级 Blake3 校验通过 |
| VERIFY | ABORTED | 双方 | session_abort（reason = FILE_HASH_MISMATCH） |
| COMMIT | COMPLETED | 接收侧 | 原子重命名完成，发布 upload_complete / download_complete |
| COMMIT | ABORTED | 双方 | session_abort |

**ABORTED 是终态（规范性）**：没有出边，也不存在 `ABORTED → ABORTED`。
清理动作（删除临时文件 / 位图 / 元数据）在进入 ABORTED 之后执行
（见 09 章「超时与回收」、13 章「失败与清理」）。

**两个视角的状态序列不同（规范性）**：接收侧经过 VERIFY / COMMIT，发送侧不经过
（发送侧没有位图，无法做文件级校验）。把两侧日志对齐时，
发送侧的 `TRANSFERRING → COMPLETED` 与接收侧的
`TRANSFERRING → VERIFY → COMMIT → COMPLETED` 对应**同一次完成**，
中间的 VERIFY / COMMIT 对发送侧不可见。

#### 终止权

- **Server 是会话状态的唯一仲裁者**：会话参数、状态、超时的裁决都由 Server 做出。
- **`session_abort` 的发布方有两类**：
  - **Server** —— 可因任意原因终止会话（超时、ACL、状态非法、资源不足等）；
  - **接收侧** —— **仅限**检测到不可恢复的数据完整性失败
    （`reason` = `FILE_HASH_MISMATCH`）。这是接收侧唯一有权发起的终止。
- 发送侧**不得**主动发布 `session_abort`（除非它也是该会话的接收侧）。
  发送侧需要终止时，**必须**发布 `session_pause` 或等待 Server 的裁决。

### 推进规则

- **发送侧状态仅由收到的 MQTT 控制消息推进**。UDP 数据包不得改变发送侧状态。
- **接收侧状态由本地事件推进**（位图全真 / Blake3 校验通过 / 原子重命名完成），
  但接收侧**每一次推进必须紧接着通过 MQTT 向发送侧发布一条控制消息**：

  | 接收侧推进 | 必须发布的控制消息 |
  |------------|--------------------|
  | 位图出现缺失 | `missing_chunks`（`mode=report`），携带缺失 chunk 列表 |
  | 位图全真 → VERIFY | 无需单独消息；随后的 `upload_complete` / `download_complete` 隐含告知 |
  | VERIFY → COMMIT | 无需单独消息 |
  | COMMIT → COMPLETED | `upload_complete`（上传方向）/ `download_complete`（下载方向） |

- **数据面（UDP）只能更新位图**，不得直接触发状态转移。
  位图变化是接收侧推进的**输入**，不是状态转移本身。
- **状态转换必须符合上述转移规则**；任何实现都不得进入表外的状态组合。

### 秒传路径（file_exists）

当 Server 侧已存在相同 `file_hash` 的完整文件时：

1. Server **必须**在生成 `upload_accept` **之前**完成既有文件的校验：
   对已存在的同 `file_hash` 文件重新计算 Blake3 并与 `upload_start.file_hash` 比对。
2. 校验**通过**时，Server 在 `upload_accept` 中置 `file_exists = true`；
   校验**不通过**时**不得**置该标志，按常规传输路径下发 `file_exists = false`。
   （`file_exists` 是单向发布的消息字段，一旦上线即不可撤回 ——
   因此校验必须发生在发布之前。）
3. 发送侧：NEGOTIATING → COMPLETED（**不发送任何数据**，直接等待完成通知）。
4. 接收侧：NEGOTIATING → VERIFY → COMMIT → COMPLETED → 发布 `upload_complete`。

秒传路径**不产生位图**，因此「bitmap 全真」这一完成条件**不适用于**秒传路径；
它的完成条件是「既有文件校验通过并原子提交」。
这与常规路径的完成条件是同一件事，只是数据来源不同。

### 暂停与恢复

- `session_pause` 使会话进入 PAUSED；位图、临时文件与已传输数据**全部保留**。
- PAUSED 状态下双方**必须**停止发送数据面数据包。
- 恢复**只有一条路径**：发送侧发布 `resume_session`，对端回 `resume_ack`
  （含续传起点与数据面端口），双方回到 TRANSFERRING。
- `session_pause` **不是**切换开关：重复发布是幂等的
  （已处于 `PAUSED` 时不产生状态变化）。
- `session_pause` 与 `session_abort` 的区别：暂停**可恢复**，中止**不可恢复**。
- `session_pause` 与 `resume_session` 的区别：`session_pause` 是**显式暂停**，
  `resume_session` 是**异常中断后的恢复**。两者的恢复动作相同（回到 TRANSFERRING）。

### 典型触发消息

| 消息 | 视角 | 状态效果 |
|------|------|----------|
| upload_start / download_start | 发送侧 | INIT → NEGOTIATING |
| upload_accept / download_accept | 发送侧 | NEGOTIATING → TRANSFERRING（或秒传 → COMPLETED） |
| missing_chunks（mode=report） | 接收侧 | 维持 TRANSFERRING，触发补发 |
| session_pause | 双方 | TRANSFERRING ⇄ PAUSED |
| resume_session / resume_ack | 发送侧 | PAUSED → TRANSFERRING（或恢复中断的会话） |
| upload_complete / download_complete | 发送侧 | TRANSFERRING → COMPLETED |
| session_abort | 双方 | 任意状态 → ABORTED |

---

## 11 流控（Flow Control）

### 基础机制：滑动窗口

- **滑动窗口（`window`）是「在途上限」**：已发送但尚未被对端确认接收的 chunk 数上限。
- `window` 由 Server 在 `upload_accept` / `download_accept` 中下发。
- **协议层参数**（互操作相关）：

  | 参数 | 归属 | 约束 |
  |------|------|------|
  | `window` | Server 下发 | **Client 不得擅自修改**；**规范默认值 4096** |
  | `chunk_size` | 协商确定 | 1024–1350，上限 1400（见 08 章） |

- **无逐包 ACK**：可靠性完全由 `missing_chunks` 裁决提供（见 12 章）。

#### 窗口与吞吐的关系（规范性）

稳态吞吐上限约为

```
throughput ≈ window × chunk_size / 裁决周期
```

以 `chunk_size = 1200`、裁决周期 50ms 计：

| window | 在途数据量 | 吞吐上限 |
|--------|-----------|----------|
| 1024 | 1.2 MB | 约 24 MB/s（≈195 Mbps） |
| 4096 | 4.9 MB | 约 98 MB/s（≈786 Mbps） |
| 8192 | 9.8 MB | 约 196 MB/s（≈1.5 Gbps） |

Server **应当**依据目标链路速率与 `chunk_size` 选取 `window`。
在 1 Gbps 局域网上的推荐取值为 **4096**。

### 发送节奏

- 发送方**必须**按**批次**发送：连续发送 `burst_size` 个 chunk（批内**不**逐包停顿），
  批间隔 `batch_gap_us`。
- `burst_size` 初值 = Server 下发的 `window`；`batch_gap_us` 初值 = **2000 µs**。
- `burst_size` **必须** ≤ `window`。
- 发送方**必须**持续推进批次，直到 `chunk_count` 全部发出；
  **唯一**的停顿条件是在途量触及 `window` 上限（见「窗口推进判据」第 3 条）。

> **口径统一**：v1.0.0 草案中，07 章写作「间隔 2–5ms」（逐包读法），
> 11 章写作「每发送 N 个 chunk 休眠 T ms」（批量读法），二者相差约 `window` 倍。
> 本版本统一为**批量读法**：间隔施加在**批**与**批**之间，而非包与包之间。

### 窗口推进判据（规范性）

这是发送方在「无逐包 ACK」下继续推进的**唯一**依据：

1. 发送方持续发送新 chunk，直到 `chunk_count` 全部发出。
2. 发送方**必须**以最近一次 `missing_chunks`（`mode=report`）裁决中报告的缺失列表
   作为重传依据，**重传优先级高于新数据**。
3. 发送方**必须**维护「已发送但尚未被对端确认接收」的 chunk 数估计
   （= 已发送数 − 最近一次裁决中对方已接收数），记为**在途量**。
   在途量**不得超过** `window`；达到上限时，发送方**必须**暂停发送**新数据**
   （重传不受此限），直至收到下一次裁决后重新计算。
   批次内部**不**因在途量触顶而中断 —— 一旦开始发送一个批次，**必须**发完该批次。
4. **裁决超时**：发送方距上一次裁决已超过 **1000 ms** 且仍在等待时，
   **必须**发布 `missing_chunks`（`mode=request`）主动索要裁决。
5. **裁决节奏**（接收侧义务）：
   - 接收方**必须**至少每 **50 ms** 下发一次 `missing_chunks`（`mode=report`），
     **即使缺失列表为空也必须下发**（空数组是「可以继续推进」的裁决）。
   - 接收方收到 `missing_chunks`（`mode=request`）时，**必须**在 **50 ms** 内
     以 `mode=report` 应答。

**空数组裁决是必需的（规范性）**：若接收方只在「有缺失」时才下发，发送方将在窗口填满后
永远等不到裁决，协议死锁。因此**周期性下发空数组裁决**是规范性要求，
而非优化建议。

### 动态流控：AIMD（推荐性）

本节描述**发送端本地策略**，**不属于互操作要求**。
两端实现**可以**采用完全不同的策略，协议行为**必须**保持一致。

**发送侧**依据对端周期性下发的 `progress_update` 估算链路状况，
据此调节 `burst_size` 与 `batch_gap_us`。

上传方向的发送侧是 Client，下载方向的发送侧是 Server ——
本节对**两个方向同等适用**。

#### 丢包率估算

```
loss_rate = 1.0 - (delta_received / delta_sent)
```

- `delta_received`：相邻两次 `progress_update` 之间 Server 确认的 chunk 数增量
- `delta_sent`：同一区间内 Client 实际发出的 chunk 数增量

**估算约束（规范性）**：

- 估算**必须**基于**滑动窗口内累计值**（窗口长度**建议**为最近 20 个 tick），而非单 tick 差值；
- 样本量不足时（`delta_sent < 16`）**不得**据此调整；
- `delta_sent` 为 0 时**不得**触发除法。

> **v1.1.0 修正**：v1.0.0 直接用单次 tick 的差值估算丢包率，在小窗口下会被
> 量化噪声主导 —— 例如每 tick 仅发送 16 块时，丢 1 块即为 6.25%，
> 直接越过 5% 拥塞阈值并触发窗口收缩，形成正反馈。

#### 调整规则

| 条件 | 动作 |
|------|------|
| 拥塞（loss_rate > 5%） | `batch_gap_us = min(max(batch_gap_us × 1.5, 200), 100000)`<br>`burst_size = max(burst_size × 0.7, min(64, window))` |
| 通畅（loss_rate < 1%） | `batch_gap_us = max(batch_gap_us - 10, 0)`<br>`burst_size = min(burst_size + 2, window)` |
| 中间区间（1% ≤ loss_rate ≤ 5%） | 保持不变 |

- `batch_gap_us` 范围 **[0, 100000] µs**；
  `burst_size` 范围 **[min(64, window), window]** —— 下界取 `min(64, window)`，
  以免 `window < 64` 时下界越过上界。
- 每次调整**应当**加入随机抖动（jitter），避免多个 Client 同步震荡。

> **v1.1.0 修正**：v1.0.0 的 AIMD 直接收缩协议层的 `window`，
> 这与「任何流控参数下协议行为都应一致」相冲突 ——
> 窗口收缩会改变在途上限，进而改变发送方的阻塞行为。
> 本版本改为调整**发送端本地**的 `burst_size` / `batch_gap_us`，
> 协议层 `window` **保持不变**。

#### 前提条件

- AIMD **只作用于发送节奏**，不改变协议语义；任何流控参数下协议行为都应一致
- AIMD 依赖 `progress_update` 的周期性下发（发送侧约每 500ms 收到一次）
- 数据面**不做**拥塞控制，AIMD 属于客户端策略而非协议层机制

#### 实现位置参考

- 桌面端：`jiayi-client/pc/windows/src-tauri/src/fsp/transport.rs`
- 移动端：`jiayi-client/app/lib/protocol/flow_control.dart`

---

## 12 可靠性模型（Reliability）

### 分层保障

- 控制面统计缺失并下发 `missing_chunks`（`mode=report`）；接收端乱序写盘与位图
- crc32 包级完整性；文件级内容 hash 校验（Blake3）
- 重传：UDP 不即时重传；聚合缺失后统一下发；重传优先级高于新数据
- 中断与恢复：不回滚已写入；恢复后加载位图并补传缺失
- 完成与提交：bitmap 全真 → 校验 hash → 原子重命名 → 发布完成消息

**裁决是唯一的推进依据（规范性）**：接收方**必须**至少每 50ms 下发一次 `missing_chunks`
（`mode=report`），**包括缺失列表为空的情况**；发送方据此推进窗口。
详见 11 章「窗口推进判据」。

### 保底轮询（Keep-Alive Polling）

- Client 发送完全部数据后，若 Server 持续返回 `missing_chunks`（`mode=report`）
  且 `chunks` 为空（缺失列表为空），但**始终未下发** `upload_complete`，
  Client 进入保底轮询状态。
- 轮询间隔 **2 秒**：Client **必须**发布 `missing_chunks`（`mode=request`）
  持续索要缺失列表，直至收到 `upload_complete` 或触发超时。
- 目的：防止因 MQTT 消息丢失导致的状态机死锁。

> **v1.1.0 修正**：v1.0.0 未定义「请求缺失列表」的消息，轮询只能靠重复发送
> 语义不明的 `missing_chunks` 实现。本版本以 `mode=request` 显式区分
> 「请求」与「报告」，两者载荷结构相同但 `chunks` 的取值约束不同（见 06 章）。

### 看门狗（Watchdog）

- Client 发出 `missing_chunks`（`mode=request`）后，若 **3 秒**内未收到任何
  `mode=report` 的回复，**必须**重发该请求。
- 重发不改变协议语义，与幂等要求兼容（见 06 章「未识别消息必须忽略，保持幂等性」）。

### 完整性校验

- **包级**：每个 UDP 包尾部携带 CRC32，校验失败直接丢弃，**不得**写入位图
- **文件级**：传输完成后，接收方计算全文件 Blake3 哈希，与请求中的 `file_hash` 比对；
  一致才进入 COMMIT 并发布完成消息（上传方向 `upload_complete`，
  下载方向 `download_complete`）
- **文件级校验失败**：接收方**必须**发布 `session_abort`（`reason` = `FILE_HASH_MISMATCH`）
  终止会话。
  这是**接收侧唯一有权发起的终止**（见 10 章「终止权」）；
  其余终止原因一律由 Server 发起。
  由于包级 CRC32 已经过校验，文件级失配通常意味着**会话参数错误**
  （如 `chunk_size` 协商不一致）而非个别数据损坏，因此**不作为可重传事件处理**。
- **秒传路径**：跳过数据传输时没有位图，完成条件为「既有文件校验通过并原子提交」
  （见 10 章「秒传路径」）。

---

## 13 中断与恢复（Resume & Recovery）

### 中断处理

- 可随时中断
- 保留已写入与位图
- 中断不回滚已写入数据

### 恢复流程

#### 发送侧发起

1. Client 发送 `resume_session {session_id}`
2. Server 加载位图 → 计算缺失 → **下发 `resume_ack`**
3. Client 按 `resume_ack` 继续发送

#### resume_ack 语义

| 字段 | 语义 |
|------|------|
| `next_chunk_id` | **续传顺序起点**：Server 位图中**第一个尚未收到的 chunk_id**；位图已全真时为 `chunk_count` |
| `missing_count` | Server 尚未收到的 chunk 总数 |
| `udp_port` | 数据面端口，可能因会话重建而变化，**Client 必须使用回执中的端口** |

> **v1.1.0 修正**：`next_chunk_id` 在 v1.0.0 中定义为「Server 已接收的**最大** chunk_id + 1」。
> 本协议显式支持乱序写与随机访问（见 01、17 章），位图可能出现**空洞** ——
> 存在低于最大已收 id 的缺失块。
> 若以 `max_received_id + 1` 作为续传起点，空洞块将**永远不会被重传**，
> 文件级 Blake3 校验必然失配并导致会话中止。
> 正确的续传起点是**第一个缺失块**。

#### 空洞块的补发

`next_chunk_id` 只决定**顺序发送流**从哪里继续；位图中低于当前发送位置的空洞
由周期下发的 `missing_chunks`（`mode=report`）负责指认：

- 接收方**必须**在裁决中报告**全部**未收到的 chunk_id，而不只是从
  `next_chunk_id` 开始的连续区间。
- 发送方**必须**优先重传裁决报告的缺失块，优先级高于新数据。

#### 接收侧发起

下载方向的接收方是 Client，中断恢复的对称流程为：

1. Client 重连 → 重新 `login`（见 06 章「持久会话与断线重连」）
2. Client 发送 `resume_session {session_id}`，Server 回 `resume_ack`
3. Server 按 `next_chunk_id` 起继续向 Client 推送 chunk；
   Client 以 `missing_chunks`（`mode=report`）报告自己的缺失块
4. 双方回到 TRANSFERRING

**ABORTED 是终态（规范性）**：会话一旦进入 ABORTED，`resume_session` **不构成**合法的恢复请求
（见 10 章）。Client **必须**发起新会话。

### 失败与清理

- 长时间未恢复可清理，删除临时文件/位图/元数据
- 清理前应确认会话已进入 ABORTED 状态
- 会话超时时长见 09 章「超时与回收」

---

## 14 下载流程（Server → Client）

### 原则

- Client 主动发起
- Server 不向未知端口推送 UDP（客户端端口由 `download_accept.udp_port` 与 UDP 探测确定）
- 控制面与可靠性机制与上传一致
- 会话类型：Upload（C→S）、Download（S→C）

### 下载控制消息

| 消息 | 方向 | 用途 |
|------|------|------|
| download_start | Client→Server | 请求下载文件 |
| download_accept | Server→Client | 接受下载请求，下发会话参数与数据面端口 |
| missing_chunks | 双向 | 缺失 chunk 列表报告 / 索要（`mode=report` / `mode=request`） |
| download_complete | Client→Server | 下载完成（由接收方 Client 发布） |

### 下载流程时序

#### 1. 会话建立

Client 发送 download_start → Server 返回 download_accept（含 `session_id` /
`chunk_count` / `window` / `udp_port` / `tcp_port`）

#### 2. UDP 探测

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

#### 3. 数据传输

Server 按 `window` 连续发送 UDP chunk（批量发送，节奏见 11 章）

#### 4. 缺失重传

Server 每 50ms 下发 `missing_chunks`（`mode=report`）裁决；
Client 收到后按 11 章补发缺失块。
收到 Client 的 `missing_chunks`（`mode=request`）时，Server **必须**在 50ms 内应答。

#### 5. 完成与终止

Client 检测位图全真 → 校验 hash → 原子重命名 → 发送 download_complete
（**接收方发布完成消息**，见 06 章）

### 与上传对称性

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

---

## 15 NAT 与网络限制

### 支持

- 单一局域网
- 二级路由（条件支持）
- 跨公网 NAT/蜂窝网络不支持

### 假设

- 同一局域网
- Client 可主动向 Server 发起 TCP/UDP
- 不提供穿透

### 建议

- Client 先发 UDP 建 NAT 映射
- mDNS 可能失效需手动配置

---

## 16 移动端后台限制

### Android

- Foreground Service
- 展示同步状态

### iOS

- 仅前台
- 后台视为中断
- 回前台需 resume_session

---

## 17 存储与内存模型

### 目标

- GB 级文件
- 内存与文件大小解耦
- 乱序写/中断恢复

### 临时文件

- 初始化创建
- 标记 incomplete
- 可预分配（稀疏）

### 写入

- offset=chunk_id*chunk_size
- 允许重复覆盖

### 内存

- 仅 payload、bitmap、Session 状态
- 与 chunk_size 成正比

### 完成与提交

- 校验 hash
- 原子重命名
- 标记 completed

---

## 18 位图与块图（Bitmap & Chunk Map）

#### 位图实现

- **存储**: 使用 `AtomicU64` 数组，每 64 个 bit 存储在一个 AtomicU64 中
- **大小**: `word_count = (chunk_count + 63) / 64`
- **持久化**: 支持保存到文件和从文件加载
- **原子操作**: 支持并发安全的状态更新

#### 位图操作

| 操作 | 说明 |
|------|------|
| mark_received(chunk_id) | 标记 chunk 为已接收，返回是否为新标记 |
| is_received(chunk_id) | 检查 chunk 是否已接收 |
| is_complete() | 检查所有 chunk 是否都已接收 |
| missing_chunks() | 获取缺失的 chunk 列表 |
| missing_chunks_limit(limit) | 获取缺失的 chunk 列表（限制数量） |
| max_received_id() | 获取已接收的最大 chunk ID |
| received_count() | 获取已接收的 chunk 数量 |
| to_bytes() | 将位图转换为字节数组 |
| from_bytes(bytes) | 从字节数组恢复位图 |
| save_to_file(path) | 将位图保存到文件 |
| load_from_file(path) | 从文件加载位图 |

#### 刷盘策略

- 建议每 1s 或每 100 chunk 刷盘一次
- 避免每 chunk 同步刷盘

#### 缺失计算优化

- 以 u64 为单位批量跳过全 1 的块
- 只有当 word != u64::MAX 时才逐位检查

#### 缺失列表的下发上限（规范性）

- 单条 `missing_chunks` 载荷中的 chunk 条目**不得超过 4096 条**
  （受 Broker 最大报文长度 ≥ 1 MiB 的约束，见 06 章）。
- 接收方**必须**按位图顺序（chunk_id 升序）**分批下发**缺失列表：
  一批被补齐后，下一批在随后的裁决周期中下发。
- 载荷中的 `missing_count` 字段给出**尚未补齐的缺失块总数**，
  用于发送方判断是否还有后续批次；**null 表示未知**（v1.0.0 对端不带此字段），
  此时发送方**必须**按「可能还有后续批次」处理。
- 缺口规模估算：100 GB 文件、`chunk_size = 1200` 约 8.9×10⁷ 块，
  即使缺失 1% 也达 8.9×10⁵ 条，**必须**依赖上述分批机制。

> v1.0.0 草案只提供了 `missing_chunks_limit(limit)` 这一 API，
> 但未规定 limit 取值，也未规定「列表过长时如何告知还有更多」。
> 本版本固定上限为 4096 并引入 `missing_count`，
> 使发送方能够确定缺失列表是否已下发完毕。

---

## 19 错误码（Error Codes）

### 分类

| 分类 | 说明 |
|------|------|
| SESSION | 会话级错误 |
| TRANSPORT | 传输层错误（UDP/TCP） |
| INTEGRITY | 完整性校验错误 |
| AUTH | 认证与 ACL 错误 |
| TIMEOUT | 超时错误 |
| STORAGE | 持久化与文件系统错误 |
| PROTOCOL | 协议格式与状态错误 |

### 错误码上线路（规范性）

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

### 建议错误码

#### SESSION

- `SESSION_NOT_FOUND`：会话不存在或已终止
- `SESSION_ABORTED`：会话被服务端终止
- `INVALID_STATE`：当前状态不允许该操作

#### TRANSPORT

- `UDP_UNAVAILABLE`：UDP 探测失败
- `TCP_ERROR`：TCP 降级通道错误
- `BIND_FAILED`：数据面端口绑定失败
- `CONNECTION_FAILED`：控制面连接失败
- `NETWORK_ERROR`：网络错误

#### INTEGRITY

- `CRC32_INVALID`：包级校验失败（丢弃该包，不标记位图）
- `FILE_HASH_MISMATCH`：文件级内容哈希不一致（**必须**以 `session_abort` 终止会话，见 12 章）
- `INVALID_CHUNK`：chunk_id 越界或非法
- `CHUNK_ALREADY_RECEIVED`：重复 chunk（幂等丢弃）

#### AUTH

- `ACL_DENY`：ACL 拒绝访问
- `CLIENT_ID_INVALID`：client_id 非法
- `LOGIN_FAILED`：登录认证失败
- `PROTO_VERSION_UNSUPPORTED`：协议版本不受支持（见 06 章 `login` / `login_ack`）

#### TIMEOUT

- `TIMEOUT_NO_ACTIVITY`：长时间无活动导致超时
- `TIMEOUT_PROBE`：UDP 探测超时

#### STORAGE

- `DATABASE_ERROR`：SQLite 持久化失败
- `FILESYSTEM_ERROR`：文件系统操作失败

#### PROTOCOL

- `INVALID_PACKET`：数据包格式非法（长度不足、越界等）
- `SESSION_ID_MISMATCH`：session_id 不匹配
- `FILE_ID_MISMATCH`：file_hash 不匹配
- `SERIALIZATION_ERROR`：JSON 编解码失败
- `CONTROL_ERROR`：控制面错误
- `UNKNOWN_ERROR`：未分类错误

### 行为

- 错误记录日志
- INTEGRITY 错误包直接丢弃，且**不得**写入位图
- SESSION 错误触发 `session_abort`（`reason` 填对应错误码）
- CHUNK_ALREADY_RECEIVED 属幂等情形，静默丢弃即可
- 同一错误不得导致状态机进入非法状态

---

## 20 安全模型（Security Model）

### 假设

- 可信局域网
- Broker 与 Server 受控
- 不穿透公网

### 控制面

- 建议启用 TLS
- 用户名/密码或客户端证书
- ACL 强制隔离 client_id 发布/订阅

### 数据面

- UDP 使用 CRC32（**完整性非安全性**）：CRC32 无密钥，任何能发包的主机都能伪造合法包
- 文件级内容 hash（Blake3）
- 数据加密由实现层自选
  - **约束**：应用层加密**不得**改变 chunk 与文件偏移的映射关系。
    若加密改变了 payload 长度或边界，则 `offset = chunk_id × chunk_size`
    的随机写盘语义失效。加密单元必须与 chunk 边界对齐，
    且密文长度**必须**恰好等于 `chunk_size`（末块除外）。
  - 数据面地址必须做**源地址校验**：接收方**必须**校验 UDP 包的源地址:端口
    与该会话经 PROBE 确认的对端地址一致，不一致即丢弃。
    这不能替代认证（地址可伪造），但能挡住非定向的误发与扫描。
- **重放保护**：接收方**必须**拒绝 `chunk_id` 超过该会话 `chunk_count` 的包，
  且**必须**校验 `session_id` 与本会话一致；两者之外的重放由位图幂等性兜底
  （重复 chunk 静默丢弃）

### 状态与权限

- UDP 会话必须绑定 MQTT Session
- Server 不向未知端口发送 UDP；目标地址必须来自 PROBE 报文（见 07 章）
- TCP 降级需重校验权限

### 审计

- 跨 client_id 访问与 session_abort 原因

---

## 21 ACL 与认证（ACL and Auth）

### 目标

- 多 Client 隔离
- 防止跨 client_id
- 服务端裁决边界明确

### ACL 推荐

所有 Topic **必须带前导斜杠**，与 22 章的 `/jiayi/fsp/{role}/{client_id}/{action}` 格式一致。

- **Client**
  - publish：`/jiayi/fsp/client/{client_id}/#`
  - subscribe：`/jiayi/fsp/server/{client_id}/#`
- **Server**
  - subscribe：`/jiayi/fsp/client/+/#`
  - publish：`/jiayi/fsp/server/{client_id}/#`

> v0.2 草案中 Server 的订阅写作 `/jiayi/fsp/client/+`（仅 4 层），
> 而消息 Topic 为 5 层，**该过滤器匹配不到任何消息**，已在 v1.0.0 更正为 `/jiayi/fsp/client/+/#`。
>
> 更严格的实现可按 action 逐个订阅（如 `/jiayi/fsp/client/+/login`、
> `/jiayi/fsp/client/+/upload_start`），语义等价且权限更小；
> 参考实现即采用此方式。

完整的 Mosquitto 配置示例见 [`examples/mosquitto/acl.conf`](../../examples/mosquitto/acl.conf)。

### 认证

- 用户名/密码
- TLS、客户端证书可与 client_id 绑定

### 强约束

- Broker 拒绝跨 client_id
- Server 不得向错误 client_id 发布
- Session 必须绑定唯一 client_id

### 审计

- 记录跨 client_id 尝试与 session_abort

---

## 22 MQTT Topic 规范（Control Plane Specification）

### 设计原则

- 单向职责
- 方向可判定
- 幂等可重复
- client_id 隔离
- session 强绑定

### Topic 格式

```
/jiayi/fsp/{role}/{client_id}/{action}
```

### 消息列表

#### Client → Server

- `/jiayi/fsp/client/{client_id}/login`
- `/jiayi/fsp/client/{client_id}/upload_start`
- `/jiayi/fsp/client/{client_id}/download_start`
- `/jiayi/fsp/client/{client_id}/resume_session`
- `/jiayi/fsp/client/{client_id}/missing_chunks`
- `/jiayi/fsp/client/{client_id}/progress_update`
- `/jiayi/fsp/client/{client_id}/session_pause`
- `/jiayi/fsp/client/{client_id}/session_abort`
- `/jiayi/fsp/client/{client_id}/status`
- `/jiayi/fsp/client/{client_id}/download_complete`
- `/jiayi/fsp/client/{client_id}/asset_recall`

#### Server → Client

- `/jiayi/fsp/server/{client_id}/login_ack`
- `/jiayi/fsp/server/{client_id}/upload_accept`
- `/jiayi/fsp/server/{client_id}/download_accept`
- `/jiayi/fsp/server/{client_id}/resume_ack`
- `/jiayi/fsp/server/{client_id}/missing_chunks`
- `/jiayi/fsp/server/{client_id}/progress_update`
- `/jiayi/fsp/server/{client_id}/udp_ready`
- `/jiayi/fsp/server/{client_id}/upload_complete`
- `/jiayi/fsp/server/{client_id}/session_pause`
- `/jiayi/fsp/server/{client_id}/session_abort`
- `/jiayi/fsp/server/{client_id}/status`
- `/jiayi/fsp/server/{client_id}/asset_recall_complete`

### 订阅与发布约束

- Client 仅订阅 `/jiayi/fsp/server/{client_id}/#`，发布到 `/jiayi/fsp/client/{client_id}/{action}`
- Server 订阅 `/jiayi/fsp/client/+/#`，发布到 `/jiayi/fsp/server/{client_id}/{action}`

> **层级说明**：Topic 固定为 5 层（`/jiayi/fsp/{role}/{client_id}/{action}`）。
> Server 侧的订阅过滤器必须匹配 5 层，`/jiayi/fsp/client/+` 仅 4 层，**匹配不到任何消息**——
> v0.2 草案此处有误，已在 v1.0.0 更正。
>
> 等价的更严格写法是按 action 逐个订阅（如 `/jiayi/fsp/client/+/login`），参考实现即采用此方式。

### 非法行为

- 跨 client_id 订阅/发布
- Client 发布 `server/*`
- **Client 发布 `upload_complete`** —— 上传方向的接收方是 Server，
  完成消息只能由 Server 发布
- **Server 发布 `download_complete`** —— 下载方向的接收方是 Client，
  完成消息只能由 Client 发布

**COMPLETED 状态下的应答义务（规范性）**：会话处于 COMPLETED 状态时，
接收方**必须**应答 `missing_chunks`（`mode=request`），
应答内容为**空数组**的 `mode=report`。

> v1.0.0 草案把「COMPLETED 状态处理 missing_chunks」列为非法行为，
> 这会抵消 12 章保底轮询「防止因 MQTT 消息丢失导致的状态机死锁」的目的：
> 若 `upload_complete` 在途中丢失而接收方已进入 COMPLETED，
> 接收方不得应答轮询 → 死锁依旧。本版本明确该义务。

**完成消息的发布方（规范性）**：

| 消息 | 方向 | 发布方 |
|------|------|--------|
| `upload_complete` | Server→Client | Server（上传方向的接收方） |
| `download_complete` | Client→Server | Client（下载方向的接收方） |

原因：只有接收方持有位图与临时文件，能完成「校验 hash → 原子重命名」
并判定成败。发送方通过收到该消息得知会话已提交。

> v1.0.0 草案把 `download_complete` 标为「双向」，同时又在「非法行为」中禁止
> Client 发布它，并在 14 章要求 Client 发布它 —— 三处互相矛盾，
> 已在 v1.1.0 统一。

### MQTT ACL 推荐

- **Client**: 发布 `/jiayi/fsp/client/{client_id}/#`，订阅 `/jiayi/fsp/server/{client_id}/#`
- **Server**: 订阅 `/jiayi/fsp/client/+/#`，发布 `/jiayi/fsp/server/{client_id}/#`

---

## 23 Client Identity 与设备绑定

### 目标

- 全局唯一、稳定可复用、设备级粒度、低耦合、安全可控

### 生成

- base32(UUIDv4) 或 hash(device_uuid+salt)，持久化安全存储，不得重启重生成
- client_id 与文件哈希（Blake3）**无关**，两者不可互相推导
- client_id 一旦分配即固定，不得因设备重装、换网或升级而改变

### 生命周期与存储

- Android EncryptedSharedPreferences/Keystore
- iOS Keychain
- 桌面加密文件

### 与登录消息的一致性

- `login` 载荷中的 `device_id` **必须**与 Topic 中的 `{client_id}` 一致。
- 不一致时 Server **必须**以 `CLIENT_ID_INVALID` 拒绝登录（见 19 章）。
- 该等价关系是 ACL 隔离（21 章）与跨 client_id 审计（20 章）的基础，
  两者都依赖「Topic 中的 client_id 即为已认证的设备身份」。

### 多设备

- 1 Device ⇔ 1 client_id；1 Account ⇔ N client_id（协议不感知账号）

### 与 Session

- client_id 可并发多 session；不得跨 client_id 使用；Server 校验一致性

### 与 MQTT ACL

- 基于 client_id 的发布/订阅隔离

### 隐私

- 不得包含敏感信息；推荐随机或哈希派生

### 非法行为

- client_id 非法；跨 client_id Topic；跨 client_id session 操作

---

## 24 端到端时序（MQTT + UDP）

### 时序流程

#### 会话建立

upload_start → upload_accept（含会话参数与数据面端口）

#### UDP 探测

UDP_PROBE → UDP_PROBE_ACK → udp_ready

> v1.0.0 草案的 24 章把同一握手写作 `UDP_HELLO` / `UDP_HELLO_ACK`，
> 与 07、08 章的 `UDP_PROBE` / `UDP_PROBE_ACK` 不一致，已在 v1.1.0 统一。

#### 数据传输

Client 连续发送 UDP chunk；Server 乱序随机写盘

#### 缺失统计与重传

接收方每 50ms 下发 `missing_chunks`（`mode=report`）裁决；
发送方按裁决重传缺失块（重传优先级高于新数据）

**裁决循环是发送方推进的唯一依据（规范性）**：接收方**必须**周期性下发裁决，
**即使缺失列表为空**（空数组 = 「可以继续推进」）。
发送方在 1000ms 内未收到任何裁决时，**必须**发布
`missing_chunks`（`mode=request`）主动索要。

#### 校验与提交

接收方校验 hash、原子重命名，发布完成消息
（上传方向 `upload_complete`，下载方向 `download_complete`）

#### 中断与恢复

resume_session → resume_ack（首个缺失块 + 数据面端口）→ 继续发送

#### 下载对称

控制面一致，仅 UDP 方向互换；完成消息改由 Client 发布

### 状态机

状态机与合法转移以 10-state-machine 为准

---

## 25 实现者指南（Implementation Guide）

> 本文档为 **非规范性文档（Non-Normative）**，用于指导工程实现。
> 规范内容以 `docs/spec/` 各章为准。

### 设计哲学

- 控制慢而可靠，数据快而可丢
- 状态推进/裁决/重传在 MQTT；高吞吐在 UDP/TCP

### 推荐分层与模块划分（Client/Server 侧）

#### 协议核心（jiayi-fsp-core）

协议无关的纯逻辑核心，建议独立为库（如 `jiayi-fsp-core`），供服务端与各端客户端复用：

| 模块 | 职责 |
|------|------|
| `protocol` | MQTT 消息定义、UDP 数据包编解码、Topic 构造与解析 |
| `session` | 会话状态、状态机与转移校验 |
| `bitmap` | 位图管理（AtomicU64 位集、缺失计算、持久化） |
| `storage` | SQLite 持久化（临时文件、位图、会话元数据） |
| `transport` | UDP / TCP 数据面抽象 |
| `control` | 控制面抽象（MQTT 客户端封装、消息编解码） |
| `util` | 文件哈希（Blake3）、CRC32、定位写盘 |

#### 端侧实现

- 桌面端：Rust 实现（`jiayi-client/pc/windows/src-tauri`）
- 移动端：Dart 实现（`jiayi-client/app`），流控逻辑独立成 `protocol/flow_control.dart`
- 服务端：Rust 实现（`jiayi-server/windows/src-tauri`）

### Session 管理

- 每个 session_id 独立状态机；发送侧仅由 MQTT 推进，接收侧由本地事件推进
  但每次推进必须通过 MQTT 同步对端；UDP 仅更新位图
- 状态可持久化
- 状态机转移必须通过 10 章定义的合法转移表校验

### UDP 数据面

- 发送端按批次顺序发送、遵守在途上限、不等待 ACK；仅当在途量触及 window 上限时才暂停发送新数据（批次内部不中断）
- 接收端校验后随机写盘、丢弃失败包
- **禁止整文件缓存**

### 位图实现

- bitset 表示接收情况；常驻内存周期刷盘
- 使用 `AtomicU64` 数组支持并发安全的状态更新
- 避免每 chunk 同步刷盘

### 重传策略

- 接收方每 50ms 统计缺失并合并下发 `missing_chunks`（`mode=report`），**包括空列表**
- 单条载荷不超过 4096 条，超出分批
- 重传优先级高于新数据

### 流控

- 基础为固定在途窗口 `window`；发送节奏由 `burst_size` / `batch_gap_us` 控制
- Client 可按 11 章的 AIMD 启发式调整发送节奏（仅本地策略，不改协议语义）

### TCP fallback

- UDP 不可用时兜底
- 不与 UDP 并行
- 保持 chunk 与 bitmap 语义
- v1.1.0 尚未定义 TCP 通道的线路格式，实现前需先补齐

### 移动端注意

- Android 前台服务；iOS 前台传输，后台中断，前台恢复

### 常见错误

- 在 UDP 层实现 ACK/拥塞
- 用 UDP 判定完成
- 整文件缓存
- 忽略 crc32
- 把 `file_hash` 当作 u64 传递（应为 Blake3 十六进制串）
- 把 `session_id` 序列化为 JSON 数字（应为字符串）
- 用 `max_received_id + 1` 作为续传起点（应为**首个缺失块**）
- 接收方只在「有缺失」时才下发裁决（发送方会在窗口填满后永久停滞）
- 在协议层修改 `window`（应由 AIMD 调整本地 `burst_size` 代替）
- 选择与对端不同的 CRC32 变体（必须使用 CRC-32/ISO-HDLC）

### MVP 顺序

1. MQTT 认证与连接
2. 单文件 UDP 上传
3. 重传机制
4. 断点续传
5. TCP fallback
6. 下载功能

---

## 26 最小参考实现（Reference MVP）

> 本文档为 **非规范性文档（Non-Normative）**，用于指导实现一个最小可用的 FSP 端到端。

### 目标

- 单文件上传
- 控制面 MQTT
- 数据面 UDP
- 随机写盘与恢复

### 状态

```
INIT → NEGOTIATING → TRANSFERRING → VERIFY → COMMIT → COMPLETED
          │              │  ↑                        │
          │              │  └────────────────────────┘
          │              ↓
          │           PAUSED
          ↓
       ABORTED （从任意状态可到达）
```

状态推进**仅由 MQTT 控制消息驱动**（接收侧由本地事件驱动，
但每次推进**必须**通过 MQTT 同步给对端，见 10 章「推进规则」）。

### Chunk

- chunk_size = 1200（建议范围 1024–1350，上限 1400）
- chunk_count = ceil(file_size / chunk_size)

### UDP 接收

```
verify_crc → match_session → pwrite(offset = chunk_id * chunk_size) → bitmap.mark
```

- 校验失败的包直接丢弃，**不标记位图**
- `session_id` 不匹配的包丢弃
- `payload_len` 越界（> 1400 或 > 协商 chunk_size）的包丢弃
- `chunk_id >= chunk_count` 的包丢弃

### 位图

- `Vec<AtomicU64>` 位集，每 64 个 bit 一个字
- 周期刷盘（建议每 1s 或每 100 chunk 一次）

### 缺失计算与下发

- 遍历位图 → missing[] → MQTT 下发
- 以 u64 为单位批量跳过全 1 的块，仅对非全 1 的字逐位检查
- 单条载荷**不超过 4096 条**，超出则分批下发（见 18 章）
- **每 50ms 下发一次 `missing_chunks`（`mode=report`），包括空列表**
  —— 空列表是发送方继续推进的裁决

### 发送

- 按批次发送：连续 `burst_size` 个 chunk，批间隔 `batch_gap_us`
- `burst_size` 初值 = `window`（规范默认 **4096**）；`batch_gap_us` 初值 = 2000µs
- 窗口是**在途上限**：在途量触及 `window` 时暂停发送新数据（批次内部不中断），
  收到裁决后重新计算
- 裁决超时 1000ms → 发布 `missing_chunks`（`mode=request`）索要

### 重传

- Client 接收 `missing_chunks`（`mode=report`）并重传
- 重传包置 `FLAG_RESEND`

### 恢复

- Server 持久化临时文件 / 位图 / 元数据
- 恢复时加载位图，下发 `resume_ack`（`next_chunk_id` = **首个缺失块**）
- 空洞块由周期裁决持续指认并补发

### 完成

- bitmap 全真 → 校验文件 Blake3 hash → 原子重命名 → `upload_complete`
- 校验失败 → `session_abort`（`reason` = `FILE_HASH_MISMATCH`）

### 验收清单

- 乱序可完成
- 中断可继续，且**空洞块**（低于最大已收 id 的缺失块）能被补发
- 内存不随文件增长
- 位图丢失可恢复
- 无 UDP ACK
- CRC32 校验失败的数据包不污染位图
- `file_hash` 在全链路保持为 Blake3 十六进制串
- `session_id` 在全链路保持为 JSON 字符串
- 接收方在**零丢包**时也周期下发空裁决，发送方不会停滞

### 下一阶段

- 下载流、TCP fallback 的线路格式、并发 session、加密与认证

---

## 99 术语表（Glossary）

### 协议相关

| 术语 | 说明 |
|------|------|
| jiayi-fsp | 协议名称（JIAYI File Sync Protocol） |
| MQTT | 控制面协议（信令 / 裁决） |
| UDP | 数据面主通道 |
| TCP fallback | 降级通道 |
| 小端序 | 本协议所有多字节整数的字节序（Little-Endian） |

### 会话与数据

| 术语 | 说明 |
|------|------|
| session | 一次文件传输会话 |
| session_id | 会话唯一标识（u32，由 Server 生成，JSON 中以字符串传输） |
| client_id | 设备唯一标识（设备级粒度） |
| chunk | 文件分块 |
| chunk_id | 分块序号（u32，从 0 开始） |
| chunk_size | 分块大小（字节，建议 1024–1350，上限 1400） |
| chunk_count | 总块数 = ceil(size / chunk_size) |
| payload_len | 载荷长度（u16） |
| flags | 数据包标志位（u16） |
| bitmap | 位图，记录各 chunk 的接收状态 |
| missing_chunks | 缺失块列表；`chunks` 恒为「发送方自己尚未收到的块」 |
| udp_ready | UDP 可用性就绪通知 |
| resume_session | 恢复会话请求 |
| file_hash | 文件哈希，Blake3 的 64 字符十六进制串 |
| window | 滑动窗口大小（**在途上限**，Server 下发，Client 不得修改） |
| burst_size | 发送批次大小（发送端本地参数，初值 = `window`） |
| batch_gap_us | 批间隔（微秒，发送端本地参数，初值 2000） |
| direction | 会话方向（`upload` / `download`），决定接收方与发送方 |
| AIMD | 加性增乘性减动态流控算法（**发送端策略**，非协议层机制） |

### 控制面消息

| 消息 | 方向 | 说明 |
|------|------|------|
| login / login_ack | C→S / S→C | 设备登录认证与响应 |
| status | 双向 | 设备在线状态报告 |
| upload_start / upload_accept | C→S / S→C | 上传会话建立 |
| download_start / download_accept | C→S / S→C | 下载会话建立 |
| udp_ready | S→C | UDP 可用性通知 |
| missing_chunks | 双向 | 缺失块列表报告（`mode=report`）/ 索要（`mode=request`） |
| progress_update | 双向 | 传输进度更新（单位：chunk 数） |
| session_pause / session_abort | 双向 | 会话暂停 / 中止 |
| resume_session / resume_ack | C→S / S→C | 会话恢复 |
| upload_complete / download_complete | S→C / C→S | 传输完成（**完成消息一律由接收方发布**） |
| asset_recall / asset_recall_complete | C→S / S→C | 资产撤回请求与回执 |

---
