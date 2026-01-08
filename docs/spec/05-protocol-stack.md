# 05 协议栈（Protocol Stack）

```
应用层（Client 应用 / Server）
控制层（MQTT）         — 状态机、重传裁决、会话管理
数据层（UDP / TCP）    — chunk 数据传输与兜底
网络层（LAN）          — Ethernet / Wi‑Fi
```

- 控制面拥有最终裁决权
- 数据面仅负责搬运与落盘
