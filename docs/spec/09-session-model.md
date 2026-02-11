# 09 会话模型（Session Model）

## 会话参数

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

## 创建规则

1. **session_id 由 Server 生成**（使用时间戳或 Snowflake 算法生成 u32，确保全局唯一）
2. **同文件正在同步 → 返回现有会话**
3. **已存在完整文件 → 直接完成**
4. **否则新建会话并生成新的 session_id**

## 持久化

- 临时文件、bitmap、元数据
- 位图使用 `AtomicU64` 数组存储，支持原子操作
- 支持保存到文件和从文件加载

## 超时与回收

长时间无活动 → ABORTED 并清理临时文件/位图/元数据
