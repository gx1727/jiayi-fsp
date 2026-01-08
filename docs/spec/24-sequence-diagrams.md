# 24 端到端时序（MQTT + UDP）

本章描述一次完整文件传输的端到端流程，上传与下载仅数据方向相反，控制面一致。

## 24.1 参与方

```
Client App        MQTT Broker           Server
    |                  |                  |
```

## 24.2 会话建立（控制面）

```
Client               MQTT               Server
  |-- upload_start --->|------------------->|
  |<-- upload_accept --|<-------------------|
  |   (session_id, chunk_size, window)
```

会话参数在此阶段确定；后续 UDP 包必须绑定该 session_id。

## 24.3 UDP 可用性探测

```
Client                                   Server
  |---- UDP_HELLO ---------------------->|
  |<--- UDP_HELLO_ACK -------------------|

Client               MQTT               Server
  |<-- udp_ready -----|<------------------|
```

探测失败进入 TCP fallback。

## 24.4 数据传输（UDP）

```
Client                                   Server
  |-- UDP chunk #0 ---------------------->|
  |-- UDP chunk #1 ---------------------->|
  |      ...                               |
  |-- UDP chunk #N ---------------------->|
```

UDP 包可能乱序、丢失、重复；Server 立即按 chunk_id 随机写盘。

## 24.5 缺失统计与重传（控制面裁决）

```
Client               MQTT               Server
  |<-- missing_chunks -|<------------------|
       [3, 7, 9]

Client                                   Server
  |-- UDP chunk #3 ---------------------->|
  |-- UDP chunk #7 ---------------------->|
  |-- UDP chunk #9 ---------------------->|
```

UDP 层不直接触发重传；由控制面统一处理。

## 24.6 校验与提交

```
Server
  - 校验文件 hash
  - 原子重命名临时文件
  - 标记 session completed

Server               MQTT               Client
  |-- upload_complete |------------------->|
```

## 24.7 中断与恢复

```
(网络中断 / App 退后台)
--- 时间间隔 ---

Client               MQTT               Server
  |-- resume_session ->|------------------->|
  |<-- missing_chunks -|<-------------------|
```

已写入 chunk 保留，仅补传缺失部分。

## 24.8 下载流程对称性

```
Upload   : Client 发送 UDP → Server 接收
Download : Server 发送 UDP → Client 接收
```

控制面时序一致，仅 UDP 数据方向互换。
