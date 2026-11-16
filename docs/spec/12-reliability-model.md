# 12 可靠性模型（Reliability）

## 分层保障

- 控制面统计缺失并下发 missing_chunks；接收端乱序写盘与位图
- crc32 包级完整性；文件级内容 hash 校验（Blake3）
- 重传：UDP 不即时重传；聚合缺失后统一下发；重传优先级高于新数据
- 中断与恢复：不回滚已写入；恢复后加载位图并补传缺失
- 完成与提交：bitmap 全真 → 校验 hash → 原子重命名 → upload_complete

## 保活轮询（Keep-Alive Polling）

- Client 发送完全部数据后，若 Server 返回 `missing_chunks: []`（缺失列表为空）但**始终未下发** `upload_complete`，Client 进入保底轮询状态
- 轮询间隔 **2 秒**，持续请求缺失列表，直至收到 `upload_complete` 或触发超时
- 目的：防止因 MQTT 消息丢失导致的状态机死锁

## 看门狗（Watchdog）

- Client 发出 `missing_chunks` 请求后，若 **3 秒**内未收到任何回复，强制重发该请求
- 重发不改变协议语义，与 12 章的幂等要求兼容（见 06 章"未识别消息必须忽略，保持幂等性"）

## 完整性校验

- **包级**：每个 UDP 包尾部携带 CRC32，校验失败直接丢弃，**不得**写入位图
- **文件级**：传输完成后，Server 计算全文件 Blake3 哈希，与 `upload_start` 中的 `file_hash` 比对；一致才发送 `upload_complete`
