# 13 中断与恢复（Resume & Recovery）

定义会话在任意阶段的中断处理与恢复流程。

## 13.1 中断处理

- 接收端可随时中断数据接收
- 已写入 chunk 不得回滚或删除
- 保留临时文件与位图

## 13.2 恢复流程（Server）

1. 接收 resume_session（Client → Server）
2. 加载位图与会话元数据
3. 计算 missing_chunks
4. 下发 missing_chunks（Server → Client）

## 13.3 失败与清理

- 长时间未恢复的会话可清理
- 清理需同时删除临时文件、位图与元数据
