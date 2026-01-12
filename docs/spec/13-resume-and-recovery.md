# 13 中断与恢复（Resume & Recovery）

## 中断处理

- 可随时中断
- 保留已写入与位图

## 恢复流程

Client 发送 resume_session → Server 加载位图→计算缺失→下发 missing_chunks

## 失败与清理

长时间未恢复可清理，删除临时文件/位图/元数据
