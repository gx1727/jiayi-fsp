# 01 架构（Architecture）

jiayi-fsp 采用分层架构：控制面与数据面解耦，Server 为最终裁决者。

## 1.1 分层结构

```
应用层（Client 应用 / Server）
控制层（MQTT）         — 会话管理 / 状态同步 / 重传调度
数据层（UDP / TCP）    — 文件 Chunk 数据流
网络层（LAN）          — Ethernet / Wi‑Fi
```

## 1.2 控制面职责

- 会话建立与参数协商（chunk_size / window / total_chunks）
- 缺失统计与重传指令（missing_chunks）
- UDP 可用性确认（udp_ready）
- 会话完成与终止（upload_complete / session_abort）

## 1.3 数据面职责

- 传输 chunk 数据（乱序、可丢、可重复）
- 包级完整性校验（crc32）
- 随机写盘与进度标记（bitmap）

## 1.4 关键原则

- 状态机仅由控制面推进
- 数据面不得直接改变会话状态
- Server 为 Source of Truth（最终一致性）
