# 21 ACL 与认证（ACL and Auth）

本章定义在 MQTT Broker 层的访问控制建议，以及与 client_id 的关系。认证机制可按部署环境决定（如用户名/密码、TLS 客户端证书）。

## 21.1 目标

- 多 Client 隔离
- 防止跨 client_id 的订阅与发布
- 明确服务端裁决边界

## 21.2 ACL 策略（推荐）

```
Client(client_id=X):
  publish   jiayi/fsp/client/X/#
  subscribe jiayi/fsp/server/X/#
```

Server：
```
subscribe jiayi/fsp/client/+/+
publish   jiayi/fsp/server/{client_id}/#
```

## 21.3 认证建议

- 局域网部署可采用用户名/密码
- 如需更强安全：启用 TLS（MQTT over TLS）
- 可选：客户端证书绑定到 client_id

## 21.4 强约束（必须）

- Broker 必须拒绝 Client 订阅/发布非自身 client_id 的 Topic
- Server 不得向错误 client_id 发布消息
- Session 必须绑定到唯一的 client_id

## 21.5 日志与审计

- 记录跨 client_id 访问尝试
- 记录 session_abort 原因
- 建议对 Topic 访问进行周期审计
