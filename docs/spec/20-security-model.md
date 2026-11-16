# 20 安全模型（Security Model）

## 假设

- 可信局域网
- Broker 与 Server 受控
- 不穿透公网

## 控制面

- 建议启用 TLS
- 用户名/密码或客户端证书
- ACL 强制隔离 client_id 发布/订阅

## 数据面

- UDP 使用 CRC32（完整性非安全性）
- 文件级内容 hash
- 数据加密由实现层自选

## 状态与权限

- UDP 会话必须绑定 MQTT Session
- Server 不向未知端口发送 UDP
- TCP 降级需重校验权限

## 审计

- 跨 client_id 访问与 session_abort 原因
