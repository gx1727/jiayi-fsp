# 15 NAT 与网络限制

## 支持

- 单一局域网
- 二级路由（条件支持）
- 跨公网 NAT/蜂窝网络不支持

## 假设

- 同一局域网
- Client 可主动向 Server 发起 TCP/UDP
- 不提供穿透

## 建议

- Client 先发 UDP 建 NAT 映射
- mDNS 可能失效需手动配置
