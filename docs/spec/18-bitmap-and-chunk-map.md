# 18 位图与块图（Bitmap & Chunk Map）

### 位图实现

- **存储**: 使用 `AtomicU64` 数组，每 64 个 bit 存储在一个 AtomicU64 中
- **大小**: `word_count = (chunk_count + 63) / 64`
- **持久化**: 支持保存到文件和从文件加载
- **原子操作**: 支持并发安全的状态更新

### 位图操作

| 操作 | 说明 |
|------|------|
| mark_received(chunk_id) | 标记 chunk 为已接收，返回是否为新标记 |
| is_received(chunk_id) | 检查 chunk 是否已接收 |
| is_complete() | 检查所有 chunk 是否都已接收 |
| missing_chunks() | 获取缺失的 chunk 列表 |
| missing_chunks_limit(limit) | 获取缺失的 chunk 列表（限制数量） |
| max_received_id() | 获取已接收的最大 chunk ID |
| received_count() | 获取已接收的 chunk 数量 |
| to_bytes() | 将位图转换为字节数组 |
| from_bytes(bytes) | 从字节数组恢复位图 |
| save_to_file(path) | 将位图保存到文件 |
| load_from_file(path) | 从文件加载位图 |

### 刷盘策略

- 建议每 1s 或每 100 chunk 刷盘一次
- 避免每 chunk 同步刷盘

### 缺失计算优化

- 以 u64 为单位批量跳过全 1 的块
- 只有当 word != u64::MAX 时才逐位检查

### 缺失列表的下发上限（规范性）

- 单条 `missing_chunks` 载荷中的 chunk 条目**不得超过 4096 条**
  （受 Broker 最大报文长度 ≥ 1 MiB 的约束，见 06 章）。
- 接收方**必须**按位图顺序（chunk_id 升序）**分批下发**缺失列表：
  一批被补齐后，下一批在随后的裁决周期中下发。
- 载荷中的 `missing_count` 字段给出**尚未补齐的缺失块总数**，
  用于发送方判断是否还有后续批次；**null 表示未知**（v1.0.0 对端不带此字段），
  此时发送方**必须**按「可能还有后续批次」处理。
- 缺口规模估算：100 GB 文件、`chunk_size = 1200` 约 8.9×10⁷ 块，
  即使缺失 1% 也达 8.9×10⁵ 条，**必须**依赖上述分批机制。

> v1.0.0 草案只提供了 `missing_chunks_limit(limit)` 这一 API，
> 但未规定 limit 取值，也未规定「列表过长时如何告知还有更多」。
> 本版本固定上限为 4096 并引入 `missing_count`，
> 使发送方能够确定缺失列表是否已下发完毕。
