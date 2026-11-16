# 09 会话模型（Session Model）

## 会话参数

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

## 创建规则

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

## 持久化

- 临时文件、bitmap、元数据
- 位图使用 `AtomicU64` 数组存储，支持原子操作
- 支持保存到文件和从文件加载

## 超时与回收

- **会话活动超时**：**60 秒**内无任何控制面交互或数据面活动的会话，
  Server **必须**将其置为 ABORTED 并清理临时文件 / 位图 / 元数据。
- 客户端在超时前可通过 `resume_session` 恢复；超时后 `resume_session` **不合法**
  （ABORTED 是终态，见 10 章）。
- 该阈值对应 19 章的 `TIMEOUT_NO_ACTIVITY`。
