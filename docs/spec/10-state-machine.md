# 10 Session 状态机

定义传输会话的状态与合法状态转移，控制面（MQTT）为唯一裁决者。

## 10.1 状态定义

```
INIT → NEGOTIATING → TRANSFERRING → COMPLETED | ABORTED
```

可选扩展（验证与提交分步）：
```
TRANSFERRING → VERIFY → COMMIT → COMPLETED
```

## 10.2 状态推进规则（硬约束）

- 只有 MQTT 控制消息可以推进状态
- UDP 数据面不得直接改变会话状态

## 10.3 合法状态转移

```
INIT        → NEGOTIATING
NEGOTIATING → TRANSFERRING
TRANSFERRING→ COMPLETED | ABORTED | VERIFY
VERIFY      → COMMIT | TRANSFERRING
COMMIT      → COMPLETED
ANY         → ABORTED（超时 / 致命错误）
```

非法转移必须拒绝并记录日志。

## 10.4 典型触发消息

- upload_start：INIT → NEGOTIATING
- upload_accept：NEGOTIATING → TRANSFERRING
- missing_chunks：维持 TRANSFERRING，触发重传
- upload_complete：TRANSFERRING / COMMIT → COMPLETED
- session_abort：ANY → ABORTED
- resume_session：维持 TRANSFERRING，触发重传统计
