# 18 位图与块图（Bitmap & Chunk Map）

## 位图

- 每 chunk 1bit
- 大小=total_chunks/8
- 必须持久化

## 刷盘

- 每 1s 或每 100 chunk 刷盘（建议）

## 缺失

- expected-[received] → missing
- 通过 missing_chunks 下发与重传
