# 13 中断与恢复（Resume & Recovery）

## 中断处理

- 可随时中断
- 保留已写入与位图
- 中断不回滚已写入数据

## 恢复流程

1. Client 发送 `resume_session`
2. Server 加载位图 → 计算缺失 → 下发 `missing_chunks` 或 `resume_ack`
3. Client 根据 `next_chunk_id` 继续发送

### resume_ack 语义

- `next_chunk_id`：Server 已接收的最大 chunk_id + 1，即续传起点
- `udp_port`：数据面端口，可能因会话重建而变化，**Client 必须使用回执中的端口**

## 失败与清理

- 长时间未恢复可清理，删除临时文件/位图/元数据
- 清理前应确认会话已进入 ABORTED 状态
