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
