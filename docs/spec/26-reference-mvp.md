# 26 最小参考实现（Reference MVP）

本文用于指导实现一个可跑通、可验证协议正确性的最小实现（MVP），不追求性能与完整功能。

## 26.1 目标

- 单文件上传（Client → Server）
- 控制面：MQTT
- 数据面：UDP
- 支持随机写盘与中断恢复

## 26.2 Session 状态机

状态：
```
INIT → NEGOTIATING → TRANSFERRING → COMPLETED | ABORTED
```

约束：
- 只有 MQTT 消息可以推进状态
- UDP 收包不能改变 Session 状态

## 26.3 Chunk 规则

```
chunk_size = 1200 bytes
chunk_count = ceil(file_size / chunk_size)
chunk_id ∈ [0, chunk_count)
```

## 26.4 UDP 接收处理

伪代码：
```
on_udp_packet(pkt):
    if !verify_crc(pkt): return
    if !match_session(pkt.session_id): return
    offset = pkt.chunk_id * chunk_size
    pwrite(file_fd, pkt.payload, offset)
    bitmap.mark(pkt.chunk_id)
```

## 26.5 Bitmap

- BitSet 表示是否接收
- 周期刷盘（如每 1s 或每 100 chunks）

## 26.6 missing_chunks

```
missing = []
for i in 0..chunk_count:
    if bitmap[i] == false:
        missing.append(i)
```

MQTT 下发：
```
{
  "session_id": "...",
  "missing_chunks": [3,7,9]
}
```

## 26.7 重传（Client 侧）

```
on_mqtt(missing_chunks):
    for chunk_id in missing_chunks:
        resend_udp(chunk_id)
```

## 26.8 中断与恢复

Server 持久化：临时文件、bitmap、session 元数据

恢复：
```
on_mqtt(resume_session):
    load bitmap
    calc missing_chunks
    send missing_chunks
```

## 26.9 完成与提交

```
if bitmap.all_true():
    verify_file_hash()
    rename(temp_file, final_file)
    mqtt(upload_complete)
```

## 26.10 验收清单

- UDP 包乱序仍可完成
- 中断后可继续
- 内存占用与文件大小无关
- bitmap 丢失可恢复
- 不存在 UDP ACK

## 26.11 下一阶段建议

- 下载（Server → Client）
- TCP fallback
- 并发 session
- 加密与认证
