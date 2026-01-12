# 06 控制面（MQTT）

控制面是会话状态推进与可靠性裁决的唯一通道。Topic 规范详见 22-topic-spec。

## 职责（完整列表）

- 会话建立：upload_start（Client→Server）、upload_accept（Server→Client）
- UDP 探测：udp_ready（Server→Client）
- 传输控制：missing_chunks（Server→Client）
- 会话终止：upload_complete、session_abort（Server→Client）
- 恢复：resume_session（Client→Server）
- 下载（可选）：download_start、download_accept、download_complete

## 载荷约束

- 所有消息必须携带 session_id（String 类型）
- upload_start：file_id（String）、file_name（String）、size（u64）、mtime（u64）、chunk_size（u32）
- upload_accept：chunk_count（u32）、window（u32）、udp_port（u16）
- missing_chunks：chunks（Vec<u32>）
- 未识别消息必须忽略，保持幂等性

## 订阅发布约束

- Client 仅 client/{client_id}/... 发布，server/{client_id}/... 订阅
- Server 订阅 client/+ 发布 server/{client_id}/...

## 与状态机关系

- 映射以 10-state-machine 为准
- Topic 语义与方向详见 22-topic-spec
