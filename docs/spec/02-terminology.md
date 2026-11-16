# 02 术语（Terminology）

## 基础术语

- `session_id`（u32）：会话唯一标识
- `file_hash`（string）：文件哈希
- `file_name`（string）：文件名
- `size`（u64）：文件大小（字节）
- `mtime`（u64）：文件修改时间戳
- `chunk_size`（u32）：数据块大小（字节）
- `chunk_id`（u32）：数据块标识
- `payload_len`（u16）：载荷长度
- `flags`（u16）：标志位

## 状态与控制

- `bitmap`：接收进度位图
- `window`（u32）：滑动窗口大小（**在途上限**，Server 下发，Client 不得修改）
- `client_id`（device_id，string）：客户端设备 ID
- `missing_chunks`（Vec<u32>）：缺失数据块列表；语义为「**发送方自己尚未收到的块**」
- `mode`（string）：`missing_chunks` 的判别字段，`report`（报告缺失）/ `request`（索要缺失列表）
- `udp_ready`：UDP 就绪通知
- `resume_session`：会话恢复请求
- `progress_update`：传输进度更新
- `session_pause`：会话暂停请求
