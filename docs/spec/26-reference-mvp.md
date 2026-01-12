# 26 最小参考实现（Reference MVP）

## 目标

- 单文件上传；控制面 MQTT；数据面 UDP；随机写盘与恢复

## 状态

- INIT→NEGOTIATING→TRANSFERRING→COMPLETED|ABORTED（仅由 MQTT 推进）

## Chunk

- chunk_size=1200；chunk_count=ceil(file_size/chunk_size)

## UDP 接收

- verify_crc→match_session→pwrite(offset=chunk_id*chunk_size)→bitmap.mark

## 位图

- 周期刷盘

## 缺失计算与下发

- 遍历位图→missing[]→MQTT 下发

## 重传

- Client 接收 missing_chunks 并重传

## 恢复

- Server 持久化临时文件/位图/元数据；恢复时加载与下发缺失

## 完成

- bitmap 全真→校验 hash→rename→upload_complete

## 验收清单

- 乱序可完成；中断可继续；内存不随文件增长；位图丢失可恢复；无 UDP ACK

## 下一阶段

- 下载、TCP fallback、并发 session、加密与认证
