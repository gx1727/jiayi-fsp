# 06 控制面（MQTT）

控制面是会话状态推进与可靠性裁决的唯一通道。Topic 规范详见 22-topic-spec。

## 6.1 职责

- 会话建立与参数协商（upload_start / upload_accept）
- UDP 可用性确认（udp_ready）
- 缺失统计与重传（missing_chunks）
- 会话完成与终止（upload_complete / session_abort）
- 会话恢复（resume_session）

## 6.2 载荷约束

- 所有业务载荷必须携带 session_id
- 载荷字段使用明确类型（uint32/uint64/string）
- 未识别消息类型必须忽略（协议向后兼容）

## 6.3 订阅发布约束

- Client：仅发布 client/{client_id}/...，仅订阅 server/{client_id}/...
- Server：订阅 client/+/+，发布 server/{client_id}/...
- ACL 强制约束见 21-acl-and-auth

## 6.4 状态机映射

- upload_start → INIT → NEGOTIATING
- upload_accept → NEGOTIATING → TRANSFERRING
- missing_chunks → 维持 TRANSFERRING
- upload_complete → COMPLETED
- session_abort → ABORTED
- resume_session → 维持 TRANSFERRING
