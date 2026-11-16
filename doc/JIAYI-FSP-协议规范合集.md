# jiayi-fsp 协议规范合集

> **本文件由 `tools/build-spec-compilation.py` 从 `docs/spec/` 自动生成，请勿手动编辑。**
>
> 单一事实来源是 `docs/spec/` 下的分章节文件。本文件仅供整体阅读与归档分发；
> 若存在差异，一律以 `docs/spec/` 的分章节原文件为准。
>
> 协议版本：v1.0.0

---


## 00 概览（Overview）

jiayi-fsp 是面向可信局域网的混合文件同步协议。通过"控制面 MQTT + 数据面 UDP"的双通道协作，在保证高吞吐的同时，实现可中断、可恢复、可裁决的文件同步能力。

### 核心特性

- 局域网内大文件（照片/视频）高效同步
- 不中断可恢复、乱序写盘、内存与文件大小解耦
- 控制与数据分离，可靠性由控制面裁决
- 控制面：MQTT；数据面：UDP；位图进度；会话恢复；NAT 探测

### 角色

- Client：发起与发送
- Server：裁决与提交

### 版本

- 协议版本：**v1.0.0**
- 状态：生产可用（Production Ready）

> 本版本对齐参考实现 `jiayi-fsp-core` 0.2.0 及 `jiayi-client` 1.0.0。
> v1.0.0 相对 v0.2 的主要变更：统一端序为小端、载荷字段以参考实现为基准、
> 新增 `asset_recall` 资产撤回扩展、纳入 AIMD 动态流控与保活/看门狗机制。

---

## 01 架构（Architecture）

### 分层架构

应用层 → 控制层（MQTT） → 数据层（UDP/TCP） → 网络层（LAN）。

### 各层职责

#### 控制面

- 登录认证
- 会话建立/协商
- 缺失统计与重传
- UDP 可用性探测
- 完成与终止
- 在线状态管理
- 进度通知

#### 数据面

- Chunk 传输
- CRC32 校验
- 随机写盘与位图

### 原则

- 状态机仅由控制面推进
- 数据面不得改变会话状态
- Server 为最终裁决者

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
- `window`（u32）：滑动窗口大小
- `client_id`（device_id，string）：客户端设备 ID
- `missing_chunks`（Vec<u32>）：缺失数据块列表
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
- 复杂拥塞控制与逐包 ACK
- 多租户不可信网络
- 移动端长期后台无 UI 传输

---

## 05 协议栈（Protocol Stack）

### 分层结构

应用层 → 控制层（MQTT：状态机/重传裁决/会话管理） → 数据层（UDP/TCP：chunk 传输与兜底） → 网络层（LAN）。

### 职责划分

- 控制面拥有最终裁决权
- 数据面仅负责搬运与落盘

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
| missing_chunks | 双向 | 缺失 chunk 列表 |
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
| upload_complete | Server→Client | 上传完成 |
| download_complete | 双向 | 下载完成 |
| session_abort | 双向 | 终止会话 |

#### 资产撤回类

| 消息 | 方向 | 用途 |
|------|------|------|
| asset_recall | Client→Server | 请求撤回（删除）已上传的素材记录 |
| asset_recall_complete | Server→Client | 撤回处理结果回执 |

### 载荷格式

所有 MQTT 消息使用 JSON 格式，包含 `type` 字段标识消息类型。

#### login

```json
{
  "type": "login",
  "device_id": "string",
  "username": "string|null"
}
```

#### login_ack

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

#### missing_chunks

```json
{
  "type": "missing_chunks",
  "session_id": "string",
  "chunks": ["u32"]
}
```

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
  "udp_port": "u16"
}
```

#### progress_update

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

#### udp_ready

```json
{
  "type": "udp_ready",
  "session_id": "string"
}
```

#### upload_complete

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

#### session_pause

```json
{
  "type": "session_pause",
  "session_id": "string"
}
```

#### session_abort

```json
{
  "type": "session_abort",
  "session_id": "string",
  "reason": "string"
}
```

#### asset_recall

```json
{
  "type": "asset_recall",
  "file_hash": "string",
  "asset_id": "string|null"
}
```

> 资产撤回请求：Client 请求 Server 删除已接收的素材记录。
> 该消息不关联 session，可独立于传输会话使用。

#### asset_recall_complete

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

#### download_start

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

#### download_accept

```json
{
  "type": "download_accept",
  "session_id": "string",
  "chunk_count": "u32",
  "window": "u32",
  "tcp_port": "u16"
}
```

#### download_complete

```json
{
  "type": "download_complete",
  "session_id": "string",
  "result": "string"
}
```

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
3. **Client 发送 upload_start 时 session_id 可为空**
4. **未识别消息必须忽略，保持幂等性**
5. **Client 仅订阅 `/jiayi/fsp/server/{client_id}/#`，发布到 `/jiayi/fsp/client/{client_id}/{action}`**
6. **Server 订阅 `/jiayi/fsp/client/+/#`，发布到 `/jiayi/fsp/server/{client_id}/{action}`**

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

- 按 window 连续发送
- 间隔 2–5ms（初值；由 AIMD 流控动态调整，见 11 章）
- 仅对 missing_chunks 重传

### 接收端

- 校验头与 crc32
- 随机写盘
- 标记位图
- 不直接推进状态机

### UDP 可用性探测

#### 交互流程

1. Client 发送 UDP_PROBE
2. Server 返回 UDP_PROBE_ACK
3. Server 通过 MQTT 发送 udp_ready

#### PROBE 数据包

- flags.bit3=1
- payload 为 session_id（u32）的 **4 字节小端编码**
  - v0.2 草案曾写作"8 字节"，与 u32 的 session_id 定义矛盾，已在 v1.0.0 更正

#### PROBE_ACK 数据包

- flags.bit3=1
- payload 为 session_id + 1 字节状态码（0=成功，1=失败）

#### 超时

- Client 等待 PROBE_ACK 超时（建议 2s）则判定 UDP 不可用
- 回退到 TCP

### TCP 降级

- UDP 不可用或异常时启用
- 保持 chunk 与 bitmap 语义

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

### Flags 位定义

| 位 | 常量 | 值 | 说明 |
|----|------|-----|------|
| bit0 | FLAG_DATA | 0x0001 | 数据包标志 |
| bit1 | FLAG_RESEND | 0x0002 | 重传包标志 |
| bit2 | FLAG_LAST_CHUNK | 0x0004 | 最后一个 chunk 标志 |
| bit3 | FLAG_PROBE | 0x0008 | 探测包标志 |
| bit4-15 | RESERVED | 0x0000 | 保留位 |

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

**探测包**:
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

### 错误处理

任一关键字段不匹配或校验失败均丢弃。校验失败的数据包**不得**标记位图。

### 扩展

- flags 预留位
- 不兼容扩展通过 MQTT 协议版本协商

---

## 09 会话模型（Session Model）

### 会话参数

| 字段 | 类型 | 说明 |
|------|------|------|
| session_id | u32 | 会话唯一标识，Server 生成 |
| client_id | String | 客户端设备 ID |
| file_hash | String | 文件哈希 |
| file_name | String | 文件名 |
| size | u64 | 文件大小（字节） |
| mtime | u64 | 文件修改时间戳 |
| state | SessionState | 会话状态 |
| chunk_size | u32 | Chunk 大小（字节） |
| window | u32 | 滑动窗口大小 |
| chunk_count | u32 | 总 chunk 数量 = ceil(size / chunk_size) |
| udp_port | u16 | UDP 端口号 |
| tcp_port | u16 | TCP 端口号 |
| last_activity | u64 | 最后活动时间戳 |

### 创建规则

1. **session_id 由 Server 生成**（使用时间戳或 Snowflake 算法生成 u32，确保全局唯一）
2. **同文件正在同步 → 返回现有会话**
3. **已存在完整文件 → 直接完成**
4. **否则新建会话并生成新的 session_id**

### 持久化

- 临时文件、bitmap、元数据
- 位图使用 `AtomicU64` 数组存储，支持原子操作
- 支持保存到文件和从文件加载

### 超时与回收

长时间无活动 → ABORTED 并清理临时文件/位图/元数据

---

## 10 Session 状态机

定义传输会话的状态与合法状态转移，控制面（MQTT）为唯一裁决者。

### 状态定义

```
INIT → NEGOTIATING → TRANSFERRING → VERIFY → COMMIT → COMPLETED
                          ↓
                      ABORTED (从任意状态可到达)
```

### 状态说明

| 状态 | 说明 |
|------|------|
| INIT | 初始化状态 - 会话刚创建 |
| NEGOTIATING | 协商状态 - 正在协商传输参数 |
| TRANSFERRING | 传输状态 - 正在进行文件传输 |
| VERIFY | 验证状态 - 验证传输完整性 |
| COMMIT | 提交状态 - 提交传输结果 |
| COMPLETED | 完成状态 - 传输成功完成 |
| ABORTED | 中止状态 - 传输被中止 |

### 合法状态转移

| 从状态 | 到状态 | 触发消息 |
|--------|--------|----------|
| INIT | NEGOTIATING | upload_start / download_start |
| INIT | ABORTED | session_abort |
| NEGOTIATING | TRANSFERRING | upload_accept / download_accept |
| NEGOTIATING | ABORTED | session_abort |
| TRANSFERRING | VERIFY | 接收完成，开始验证 |
| TRANSFERRING | COMPLETED | 小文件一次性传输完成 |
| TRANSFERRING | ABORTED | session_abort |
| VERIFY | COMMIT | 验证通过 |
| VERIFY | TRANSFERRING | 发现缺失 chunk，需要重传 |
| VERIFY | ABORTED | session_abort |
| COMMIT | COMPLETED | 提交成功 |
| COMMIT | ABORTED | session_abort |
| 任意状态 | ABORTED | session_abort |

### 推进规则

- **仅 MQTT 控制消息可以推进状态**
- **UDP 不得改变状态**
- **状态转换必须符合上述转移规则**

### 典型触发消息

- upload_start：INIT → NEGOTIATING
- upload_accept：NEGOTIATING → TRANSFERRING
- missing_chunks：维持 TRANSFERRING，触发重传
- upload_complete：COMMIT → COMPLETED
- session_abort：任意状态 → ABORTED
- resume_session：根据位图状态继续

---

## 11 流控（Flow Control）

### 基础机制：滑动窗口

- 固定滑动窗口（window），由 Server 在 upload_accept / download_accept 中下发
- 发送规则：顺序填充与发送；达上限暂停；等待 missing_chunks 重传
- 节奏：每发送 N 个 chunk 休眠 T ms（建议 N=window，T=2–5ms）
- 确认与重传：无逐包 ACK；仅通过 missing_chunks 裁决

> 窗口大小是**初始值**，Client 允许按下方 AIMD 算法动态调整。

### 动态流控：AIMD

Client 依据 Server 周期性下发的 `progress_update` 估算丢包率，据此动态调节发送速率与窗口。

#### 丢包率估算

```
loss_rate = 1.0 - (delta_received / delta_sent)
```

- `delta_received`：相邻两次 `progress_update` 之间 Server 确认的 chunk 数增量
- `delta_sent`：同一区间内 Client 实际发出的 chunk 数增量

#### 调整规则

| 条件 | 动作 |
|------|------|
| 拥塞（loss_rate > 5%） | `packet_gap_us *= 1.5`（发送间隔增大，减速）<br>`window_size *= 0.7`（窗口减小） |
| 通畅（loss_rate < 1%） | `packet_gap_us -= 10us`（发送间隔减小，加速）<br>`window_size += 2`（窗口线性增大） |
| 中间区间（1% ≤ loss_rate ≤ 5%） | 保持不变 |

- `packet_gap_us` 与 `window_size` 均需设置上下限，避免退化（间隔趋近 0 或窗口趋近 0）
- 每次调整应加随机抖动（jitter），避免多个 Client 同步震荡

#### 实现位置参考

- 桌面端：`jiayi-client/pc/windows/src-tauri/src/fsp/transport.rs`
- 移动端：`jiayi-client/app/lib/protocol/flow_control.dart`

#### 前提条件

- AIMD **只作用于发送节奏**，不改变协议语义；任何流控参数下协议行为都应一致
- AIMD 依赖 `progress_update` 的周期性下发（建议 Server 约 500ms 一次）
- 数据面**不做**拥塞控制，AIMD 属于客户端策略而非协议层机制

---

## 12 可靠性模型（Reliability）

### 分层保障

- 控制面统计缺失并下发 missing_chunks；接收端乱序写盘与位图
- crc32 包级完整性；文件级内容 hash 校验（Blake3）
- 重传：UDP 不即时重传；聚合缺失后统一下发；重传优先级高于新数据
- 中断与恢复：不回滚已写入；恢复后加载位图并补传缺失
- 完成与提交：bitmap 全真 → 校验 hash → 原子重命名 → upload_complete

### 保活轮询（Keep-Alive Polling）

- Client 发送完全部数据后，若 Server 返回 `missing_chunks: []`（缺失列表为空）但**始终未下发** `upload_complete`，Client 进入保底轮询状态
- 轮询间隔 **2 秒**，持续请求缺失列表，直至收到 `upload_complete` 或触发超时
- 目的：防止因 MQTT 消息丢失导致的状态机死锁

### 看门狗（Watchdog）

- Client 发出 `missing_chunks` 请求后，若 **3 秒**内未收到任何回复，强制重发该请求
- 重发不改变协议语义，与 12 章的幂等要求兼容（见 06 章"未识别消息必须忽略，保持幂等性"）

### 完整性校验

- **包级**：每个 UDP 包尾部携带 CRC32，校验失败直接丢弃，**不得**写入位图
- **文件级**：传输完成后，Server 计算全文件 Blake3 哈希，与 `upload_start` 中的 `file_hash` 比对；一致才发送 `upload_complete`

---

## 13 中断与恢复（Resume & Recovery）

### 中断处理

- 可随时中断
- 保留已写入与位图
- 中断不回滚已写入数据

### 恢复流程

1. Client 发送 `resume_session`
2. Server 加载位图 → 计算缺失 → 下发 `missing_chunks` 或 `resume_ack`
3. Client 根据 `next_chunk_id` 继续发送

#### resume_ack 语义

- `next_chunk_id`：Server 已接收的最大 chunk_id + 1，即续传起点
- `udp_port`：数据面端口，可能因会话重建而变化，**Client 必须使用回执中的端口**

### 失败与清理

- 长时间未恢复可清理，删除临时文件/位图/元数据
- 清理前应确认会话已进入 ABORTED 状态

---

## 14 下载流程（Server → Client）

### 原则

- Client 主动发起
- Server 不向未知端口推送 UDP
- 控制面与可靠性机制与上传一致
- 会话类型：Upload（C→S）、Download（S→C）

### 下载控制消息

| 消息 | 方向 | 用途 |
|------|------|------|
| download_start | Client→Server | 请求下载文件 |
| download_accept | Server→Client | 接受下载请求 |
| missing_chunks | Client→Server | 客户端报告缺失 chunk |
| download_complete | 双向 | 下载完成通知 |

### 下载流程时序

#### 1. 会话建立

Client 发送 download_start → Server 返回 download_accept

#### 2. UDP 探测

Client 发送 UDP_PROBE → Server 返回 UDP_PROBE_ACK → Server 发送 udp_ready

#### 3. 数据传输

Server 按 window 连续发送 UDP chunk

#### 4. 缺失重传

Client 发送 missing_chunks → Server 重传缺失 chunk

#### 5. 完成与终止

Client 检测位图全真 → 校验 hash → 原子重命名 → 发送 download_complete

### 与上传对称性

| 阶段 | 上传（C→S） | 下载（S→C） |
|------|-------------|-------------|
| 会话建立 | upload_start | download_start |
| 参数下发 | upload_accept | download_accept |
| 数据方向 | Client→Server | Server→Client |
| 缺失请求 | Server→Client | Client→Server |
| 完成通知 | upload_complete | download_complete |

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
- `FILE_HASH_MISMATCH`：文件级内容哈希不一致
- `INVALID_CHUNK`：chunk_id 越界或非法
- `CHUNK_ALREADY_RECEIVED`：重复 chunk（幂等丢弃）

#### AUTH

- `ACL_DENY`：ACL 拒绝访问
- `CLIENT_ID_INVALID`：client_id 非法
- `LOGIN_FAILED`：登录认证失败

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
- SESSION 错误触发 `session_abort`
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

- UDP 使用 CRC32（完整性非安全性）
- 文件级内容 hash
- 数据加密由实现层自选

### 状态与权限

- UDP 会话必须绑定 MQTT Session
- Server 不向未知端口发送 UDP
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
> 而消息 Topic 为 5 层，**该过滤器匹配不到任何消息**，已在 v1.0.0 更正为 `+/+`。
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
- `/jiayi/fsp/server/{client_id}/download_complete`
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
- Client 主动发布 `upload_complete` / `download_complete`
- COMPLETED 状态处理 missing_chunks

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

upload_start → upload_accept（含会话参数）

#### UDP 探测

UDP_HELLO → UDP_HELLO_ACK → udp_ready

#### 数据传输

Client 连续发送 UDP chunk；Server 乱序随机写盘

#### 缺失统计与重传

Server 下发 missing_chunks；Client 按指令重传

#### 校验与提交

Server 校验 hash、原子重命名、upload_complete

#### 中断与恢复

resume_session → missing_chunks

#### 下载对称

控制面一致，仅 UDP 方向互换

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

- 每个 session_id 独立状态机；仅由 MQTT 推进；UDP 仅更新位图
- 状态可持久化
- 状态机转移必须通过 10 章定义的合法转移表校验

### UDP 数据面

- 发送端顺序发送、遵守 window、不等待 ACK
- 接收端校验后随机写盘、丢弃失败包
- **禁止整文件缓存**

### 位图实现

- bitset 表示接收情况；常驻内存周期刷盘
- 使用 `AtomicU64` 数组支持并发安全的状态更新
- 避免每 chunk 同步刷盘

### 重传策略

- Server 周期统计缺失并合并下发
- 重传优先级高于新数据

### 流控

- 基础为固定滑动窗口；Client 可按 11 章的 AIMD 动态调整发送节奏

### TCP fallback

- UDP 不可用时兜底
- 不与 UDP 并行
- 保持 chunk 与 bitmap 语义

### 移动端注意

- Android 前台服务；iOS 前台传输，后台中断，前台恢复

### 常见错误

- 在 UDP 层实现 ACK/拥塞
- 用 UDP 判定完成
- 整文件缓存
- 忽略 crc32
- 把 `file_hash` 当作 u64 传递（应为 Blake3 十六进制串）
- 把 `session_id` 序列化为 JSON 数字（应为字符串）

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
                          ↓
                      ABORTED (从任意状态可到达)
```

状态推进**仅由 MQTT 控制消息驱动**。

### Chunk

- chunk_size = 1200（建议范围 1024–1350，上限 1400）
- chunk_count = ceil(file_size / chunk_size)

### UDP 接收

```
verify_crc → match_session → pwrite(offset = chunk_id * chunk_size) → bitmap.mark
```

- 校验失败的包直接丢弃，**不标记位图**
- `session_id` 不匹配的包丢弃

### 位图

- `Vec<AtomicU64>` 位集，每 64 个 bit 一个字
- 周期刷盘（建议每 1s 或每 100 chunk 一次）

### 缺失计算与下发

- 遍历位图 → missing[] → MQTT 下发
- 以 u64 为单位批量跳过全 1 的块，仅对非全 1 的字逐位检查

### 重传

- Client 接收 `missing_chunks` 并重传
- 重传包置 `FLAG_RESEND`

### 恢复

- Server 持久化临时文件 / 位图 / 元数据
- 恢复时加载位图并下发缺失

### 完成

- bitmap 全真 → 校验文件 Blake3 hash → 原子重命名 → `upload_complete`

### 验收清单

- 乱序可完成
- 中断可继续
- 内存不随文件增长
- 位图丢失可恢复
- 无 UDP ACK
- CRC32 校验失败的数据包不污染位图
- `file_hash` 在全链路保持为 Blake3 十六进制串

### 下一阶段

- 下载、TCP fallback、并发 session、加密与认证

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
| missing_chunks | 缺失块列表 |
| udp_ready | UDP 可用性就绪通知 |
| resume_session | 恢复会话请求 |
| missing_chunks | 缺失块列表（双向：请求 / 返回） |
| file_hash | 文件哈希，Blake3 的 64 字符十六进制串 |
| window | 滑动窗口大小 |
| AIMD | 加性增乘性减动态流控算法 |

### 控制面消息

| 消息 | 方向 | 说明 |
|------|------|------|
| login / login_ack | C→S / S→C | 设备登录认证与响应 |
| status | 双向 | 设备在线状态报告 |
| upload_start / upload_accept | C→S / S→C | 上传会话建立 |
| download_start / download_accept | C→S / S→C | 下载会话建立 |
| udp_ready | S→C | UDP 可用性通知 |
| missing_chunks | 双向 | 缺失块列表请求 / 返回 |
| progress_update | 双向 | 传输进度更新（单位：chunk 数） |
| session_pause / session_abort | 双向 | 会话暂停 / 中止 |
| resume_session / resume_ack | C→S / S→C | 会话恢复 |
| upload_complete / download_complete | S→C / 双向 | 传输完成 |
| asset_recall / asset_recall_complete | C→S / S→C | 资产撤回请求与回执 |

---
