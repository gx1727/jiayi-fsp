# 25 实现者指南（Implementation Guide）

## 设计哲学

- 控制慢而可靠，数据快而可丢；状态推进/裁决/重传在 MQTT；高吞吐在 UDP/TCP

## 推荐分层与模块划分（Client/Server 侧）

## Session 管理

- 每个 session_id 独立状态机；仅由 MQTT 推进；UDP 仅更新位图；状态可持久化

## UDP 数据面

- 发送端顺序发送、遵守 window、不等待 ACK；接收端校验后随机写盘、丢弃失败包；禁止整文件缓存

## 位图实现

- bitset 表示接收情况；常驻内存周期刷盘；避免每 chunk 同步刷盘

## 重传策略

- Server 周期统计缺失并合并下发；重传优先级低于新数据

## TCP fallback

- UDP 不可用时兜底，不并行；保持 chunk 与 bitmap 语义

## 移动端注意

- Android 前台服务；iOS 前台传输，后台中断，前台恢复

## 常见错误

- 在 UDP 层实现 ACK/拥塞；用 UDP 判定完成；整文件缓存；忽略 crc32

## MVP 顺序

- MQTT→单文件 UDP→重传→恢复→TCP fallback
