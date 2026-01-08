# 02 术语（Terminology）

- session_id：传输会话唯一标识（uint64）
- file_id_hash：文件内容短哈希（如 SHA256 前 64 位，uint64）
- chunk_size：单个 chunk 的尺寸（bytes）
- chunk_id：chunk 索引，从 0 开始（uint32）
- payload_len：当前 UDP 包实际数据长度（uint16）
- flags：数据包控制位掩码（uint16）
- bitmap：位图，追踪已接收 chunk 进度
- window：发送端滑动窗口大小（uint32）
- client_id：MQTT 层客户端唯一标识（string）
- missing_chunks：缺失 chunk 列表（uint32[]）
- udp_ready：控制消息，表示 UDP 通道可用
- resume_session：控制消息，请求恢复会话
