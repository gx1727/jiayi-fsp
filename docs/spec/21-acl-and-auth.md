# 21 ACL 与认证（ACL and Auth）

## 目标

- 多 Client 隔离
- 防止跨 client_id
- 服务端裁决边界明确

## ACL 推荐

所有 Topic **必须带前导斜杠**，与 22 章的 `/jiayi/fsp/{role}/{client_id}/{action}` 格式一致。

- **Client**
  - publish：`/jiayi/fsp/client/{client_id}/#`
  - subscribe：`/jiayi/fsp/server/{client_id}/#`
- **Server**
  - subscribe：`/jiayi/fsp/client/+/#`
  - publish：`/jiayi/fsp/server/{client_id}/#`

> v0.2 草案中 Server 的订阅写作 `/jiayi/fsp/client/+`（仅 4 层），
> 而消息 Topic 为 5 层，**该过滤器匹配不到任何消息**，已在 v1.0.0 更正为 `+/+`。
>
> 更严格的实现可按 action 逐个订阅（如 `/jiayi/fsp/client/+/login`、
> `/jiayi/fsp/client/+/upload_start`），语义等价且权限更小；
> 参考实现即采用此方式。

完整的 Mosquitto 配置示例见 [`examples/mosquitto/acl.conf`](../../examples/mosquitto/acl.conf)。

## 认证

- 用户名/密码
- TLS、客户端证书可与 client_id 绑定

## 强约束

- Broker 拒绝跨 client_id
- Server 不得向错误 client_id 发布
- Session 必须绑定唯一 client_id

## 审计

- 记录跨 client_id 尝试与 session_abort
