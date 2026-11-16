# 13 中断与恢复（Resume & Recovery）

## 中断处理

- 可随时中断
- 保留已写入与位图
- 中断不回滚已写入数据

## 恢复流程

### 发送侧发起

1. Client 发送 `resume_session {session_id}`
2. Server 加载位图 → 计算缺失 → **下发 `resume_ack`**
3. Client 按 `resume_ack` 继续发送

### resume_ack 语义

| 字段 | 语义 |
|------|------|
| `next_chunk_id` | **续传顺序起点**：Server 位图中**第一个尚未收到的 chunk_id**；位图已全真时为 `chunk_count` |
| `missing_count` | Server 尚未收到的 chunk 总数 |
| `udp_port` | 数据面端口，可能因会话重建而变化，**Client 必须使用回执中的端口** |

> **v1.1.0 修正**：`next_chunk_id` 在 v1.0.0 中定义为「Server 已接收的**最大** chunk_id + 1」。
> 本协议显式支持乱序写与随机访问（见 01、17 章），位图可能出现**空洞** ——
> 存在低于最大已收 id 的缺失块。
> 若以 `max_received_id + 1` 作为续传起点，空洞块将**永远不会被重传**，
> 文件级 Blake3 校验必然失配并导致会话中止。
> 正确的续传起点是**第一个缺失块**。

### 空洞块的补发

`next_chunk_id` 只决定**顺序发送流**从哪里继续；位图中低于当前发送位置的空洞
由周期下发的 `missing_chunks`（`mode=report`）负责指认：

- 接收方**必须**在裁决中报告**全部**未收到的 chunk_id，而不只是从
  `next_chunk_id` 开始的连续区间。
- 发送方**必须**优先重传裁决报告的缺失块，优先级高于新数据。

### 接收侧发起

下载方向的接收方是 Client，中断恢复的对称流程为：

1. Client 重连 → 重新 `login`（见 06 章「持久会话与断线重连」）
2. Client 发送 `resume_session {session_id}`，Server 回 `resume_ack`
3. Server 按 `next_chunk_id` 起继续向 Client 推送 chunk；
   Client 以 `missing_chunks`（`mode=report`）报告自己的缺失块
4. 双方回到 TRANSFERRING

**ABORTED 是终态（规范性）**：会话一旦进入 ABORTED，`resume_session` **不构成**合法的恢复请求
（见 10 章）。Client **必须**发起新会话。

## 失败与清理

- 长时间未恢复可清理，删除临时文件/位图/元数据
- 清理前应确认会话已进入 ABORTED 状态
- 会话超时时长见 09 章「超时与回收」
