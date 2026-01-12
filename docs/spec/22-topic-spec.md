# 22 MQTT Topic 规范（Control Plane Specification）

## 设计原则

- 单向职责、方向可判定、幂等可重复、client_id 隔离、session 强绑定
- 命名：jiayi/fsp/{role}/{client_id}/{action}（role=client|server）
- 订阅与发布：Client 订阅 server/{client_id}/#，发布 client/{client_id}/upload_start 与 resume_session；Server 订阅 client/+，发布 server/{client_id}/...

## 会话建立

- upload_start（Client→Server）Topic: jiayi/fsp/client/{client_id}/upload_start，Payload: session_id、file_id_hash、file_size、chunk_size
- upload_accept（Server→Client）Topic: jiayi/fsp/server/{client_id}/upload_accept，Payload: session_id、chunk_count、window

## UDP 可用性

- udp_ready（Server→Client）Topic: jiayi/fsp/server/{client_id}/udp_ready，Payload: session_id

## 传输阶段

- missing_chunks（Server→Client）Topic: jiayi/fsp/server/{client_id}/missing_chunks，Payload: session_id、missing_chunks[]

## 完成与终止

- upload_complete（Server→Client）
- session_abort（Server→Client）

## 恢复

- resume_session（Client→Server）

## Topic 与状态机映射

- 详见原表（06 控制面与 10 状态机一致）

## 非法行为

- 跨 client_id 订阅/发布、Client 发布 server/*、Client 主动发布 upload_complete、COMPLETED 状态处理 missing_chunks

## MQTT ACL 建议

- Client publish client/{client_id}/#；subscribe server/{client_id}/#
