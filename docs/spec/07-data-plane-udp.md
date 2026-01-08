# 07 数据面（UDP / TCP Fallback）

数据面承载高吞吐文件 chunk 传输。UDP 为主、TCP 为降级；控制面始终拥有最终裁决权。

## 7.1 UDP 原则

- 无连接、无逐包 ACK、允许丢包与乱序
- 固定 chunk 语义，payload_len ≤ chunk_size
- 包级完整性使用 crc32 校验

## 7.2 发送端行为（Client）

- 按 window 连续发送 chunk
- 发送间隔建议 2–5ms（可调）
- 不做拥塞控制，不等待 ACK
- 重传仅针对 missing_chunks 指定的 chunk

## 7.3 接收端行为（Server）

- 校验 session_id / file_id_hash / crc32
- 随机写盘（offset = chunk_id * chunk_size）
- 标记 bitmap 接收进度
- 不直接推进状态机

## 7.4 UDP 可用性探测

```
Client → Server : UDP_HELLO
Server → Client : UDP_HELLO_ACK
Server → Client : udp_ready (MQTT)
```

三步成功方视为 UDP 可用。

## 7.5 TCP 降级

- 启用条件：UDP 探测失败、丢包异常比例高、明确网络错误
- 行为约束：单 session、保持 chunk 与 bitmap 机制、不追求极限性能
