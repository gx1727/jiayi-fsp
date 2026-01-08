# 18 位图与块图（Bitmap & Chunk Map）

定义接收端进度追踪的数据结构与持久化策略。

## 18.1 位图结构

- 每个 chunk 使用 1 bit 表示接收状态
- 位图大小 = total_chunks / 8
- 位图必须持久化到磁盘

## 18.2 刷盘策略

- 每 1s 或每 100 个 chunk 刷盘一次（建议）
- 避免每个 chunk 都同步写位图文件

## 18.3 计算缺失

```
expected = [0..total_chunks)
missing  = expected - received_chunks
```

缺失列表通过 missing_chunks 下发并重传。
