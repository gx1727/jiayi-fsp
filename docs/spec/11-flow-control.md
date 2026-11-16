# 11 流控（Flow Control）

- 固定滑动窗口（window），由 Server 下发
- 发送规则：顺序填充与发送；达上限暂停；等待 missing_chunks 重传
- 节奏：每发送 N 个 chunk 休眠 T ms（建议 N=window，T=2–5ms）
- 确认与重传：无逐包 ACK；仅通过 missing_chunks 裁决
