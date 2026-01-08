# 22 MQTT Topic 规范（Control Plane Specification）

本文为规范性章节，定义 JIAYI-FSP 中 MQTT 控制面的 Topic 命名规则、消息方向、载荷结构与时序约束，并与状态机严格对应。

## 22.1 设计原则

- 单向职责明确：每个 Topic 只承载一种控制语义
- 方向可判定：仅通过 Topic 即可判断 Client → Server 或 Server → Client
- 幂等可重复：允许重复发布，不产生副作用
- Client 级隔离：所有 Topic 路径必须包含 client_id
- Session 强绑定：业务消息必须携带 session_id（Payload）

## 22.2 命名规则

统一前缀：

```
jiayi/fsp/
```

层级结构：

```
jiayi/fsp/{role}/{client_id}/{action}
```

- {role}：client | server
- {client_id}：客户端唯一标识（字符串）
- {action}：具体控制动作

## 22.3 订阅与发布约束

- Client 订阅：jiayi/fsp/server/{client_id}/#
- Client 发布：jiayi/fsp/client/{client_id}/upload_start、resume_session
- Client 禁止发布：jiayi/fsp/server/*
- Server 订阅：jiayi/fsp/client/+/+
- Server 发布：jiayi/fsp/server/{client_id}/...

## 22.4 会话建立

upload_start（Client → Server）

Topic：
```
jiayi/fsp/client/{client_id}/upload_start
```

Payload：
```json
{
  "session_id": "uint64",
  "file_id_hash": "uint64",
  "file_size": "uint64",
  "chunk_size": "uint32"
}
```

upload_accept（Server → Client）

Topic：
```
jiayi/fsp/server/{client_id}/upload_accept
```

Payload：
```json
{
  "session_id": "uint64",
  "chunk_count": "uint64",
  "window": "uint32"
}
```

## 22.5 UDP 可用性确认

udp_ready（Server → Client）

Topic：
```
jiayi/fsp/server/{client_id}/udp_ready
```

Payload：
```json
{
  "session_id": "uint64"
}
```

## 22.6 传输阶段

missing_chunks（Server → Client）

Topic：
```
jiayi/fsp/server/{client_id}/missing_chunks
```

Payload：
```json
{
  "session_id": "uint64",
  "missing_chunks": ["uint32", "uint32"]
}
```

约束：
- 允许重复发送
- Client 必须幂等处理

## 22.7 完成与终止

upload_complete（Server → Client）

Topic：
```
jiayi/fsp/server/{client_id}/upload_complete
```

Payload：
```json
{
  "session_id": "uint64",
  "status": "ok"
}
```

session_abort（Server → Client）

Topic：
```
jiayi/fsp/server/{client_id}/session_abort
```

Payload：
```json
{
  "session_id": "uint64",
  "reason": "string"
}
```

## 22.8 会话恢复

resume_session（Client → Server）

Topic：
```
jiayi/fsp/client/{client_id}/resume_session
```

Payload：
```json
{
  "session_id": "uint64"
}
```

## 22.9 Topic 与状态机映射

| Topic           | 允许状态         | 状态变化                       |
| --------------- | ---------------- | ------------------------------ |
| upload_start    | INIT             | INIT → NEGOTIATING             |
| upload_accept   | NEGOTIATING      | NEGOTIATING → TRANSFERRING     |
| missing_chunks  | TRANSFERRING     | 不变                           |
| upload_complete | TRANSFERRING     | TRANSFERRING → COMPLETED       |
| session_abort   | ANY              | → ABORTED                      |
| resume_session  | TRANSFERRING     | 不变                           |

## 22.10 非法行为（必须拒绝）

- Client 订阅非自身 {client_id} 的 Topic
- Client 发布 jiayi/fsp/server/*
- Server 向错误 {client_id} 发布消息
- Client 主动发布 upload_complete
- 在 COMPLETED 状态处理 missing_chunks

## 22.11 MQTT ACL 建议

```
Client 权限：
  publish   jiayi/fsp/client/{client_id}/#
  subscribe jiayi/fsp/server/{client_id}/#
```
