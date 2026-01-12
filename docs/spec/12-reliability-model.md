# 12 可靠性模型（Reliability）

- 控制面统计缺失并下发 missing_chunks；接收端乱序写盘与位图；crc32 包级完整性；文件级内容 hash 校验
- 重传：UDP 不即时重传；聚合缺失后统一下发；重传优先级高于新数据
- 中断与恢复：不回滚已写入；恢复后加载位图并补传缺失
- 完成与提交：bitmap 全真 → 校验 hash → 原子重命名 → upload_complete
