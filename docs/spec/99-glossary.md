# 99 术语表（Glossary）

- jiayi-fsp：协议名称，面向可信局域网的文件同步协议
- MQTT：消息队列遥测传输，控制面协议
- UDP：用户数据报协议，数据面主通道
- TCP fallback：UDP 不可用时的降级通道
- session：一次文件传输会话
- session_id：会话唯一标识（uint64）
- client_id：客户端在 MQTT 层的唯一标识（string）
- chunk：文件分块单元
- chunk_id：分块索引（uint32）
- chunk_size：分块大小（bytes）
- payload_len：UDP 包的实际数据长度（uint16）
- flags：UDP 包控制位掩码（uint16）
- bitmap：接收进度位图
- missing_chunks：缺失块列表（uint32[]）
- udp_ready：UDP 可用性确认消息
- resume_session：会话恢复消息
