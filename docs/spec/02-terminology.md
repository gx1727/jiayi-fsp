# 02 术语（Terminology）

## 基础术语

- `session_id`（uint64）：会话唯一标识
- `file_id_hash`（uint64）：文件哈希标识
- `chunk_size`（bytes）：数据块大小
- `chunk_id`（uint32）：数据块标识
- `payload_len`（uint16）：载荷长度
- `flags`（uint16）：标志位

## 状态与控制

- `bitmap`：接收进度位图
- `window`（uint32）：滑动窗口大小
- `client_id`（string）：客户端唯一标识
- `missing_chunks`（uint32[]）：缺失数据块列表
- `udp_ready`：UDP 就绪通知
- `resume_session`：会话恢复请求
