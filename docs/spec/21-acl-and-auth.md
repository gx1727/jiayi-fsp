# 21 ACL 与认证（ACL and Auth）

## 目标

- 多 Client 隔离
- 防止跨 client_id
- 服务端裁决边界明确

## ACL 推荐

- Client publish jiayi/fsp/client/{client_id}/#
- subscribe jiayi/fsp/server/{client_id}/#
- Server subscribe client/+、publish server/{client_id}/#

## 认证

- 用户名/密码
- TLS、客户端证书可与 client_id 绑定

## 强约束

- Broker 拒绝跨 client_id
- Server 不得向错误 client_id 发布
- Session 必须绑定唯一 client_id

## 审计

- 记录跨 client_id 尝试与 session_abort
