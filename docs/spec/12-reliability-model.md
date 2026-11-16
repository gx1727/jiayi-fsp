# 12 可靠性模型（Reliability）

## 分层保障

- 控制面统计缺失并下发 `missing_chunks`（`mode=report`）；接收端乱序写盘与位图
- crc32 包级完整性；文件级内容 hash 校验（Blake3）
- 重传：UDP 不即时重传；聚合缺失后统一下发；重传优先级高于新数据
- 中断与恢复：不回滚已写入；恢复后加载位图并补传缺失
- 完成与提交：bitmap 全真 → 校验 hash → 原子重命名 → 发布完成消息

**裁决是唯一的推进依据（规范性）**：接收方**必须**至少每 50ms 下发一次 `missing_chunks`
（`mode=report`），**包括缺失列表为空的情况**；发送方据此推进窗口。
详见 11 章「窗口推进判据」。

## 保底轮询（Keep-Alive Polling）

- Client 发送完全部数据后，若 Server 持续返回 `missing_chunks`（`mode=report`）
  且 `chunks` 为空（缺失列表为空），但**始终未下发** `upload_complete`，
  Client 进入保底轮询状态。
- 轮询间隔 **2 秒**：Client **必须**发布 `missing_chunks`（`mode=request`）
  持续索要缺失列表，直至收到 `upload_complete` 或触发超时。
- 目的：防止因 MQTT 消息丢失导致的状态机死锁。

> **v1.1.0 修正**：v1.0.0 未定义「请求缺失列表」的消息，轮询只能靠重复发送
> 语义不明的 `missing_chunks` 实现。本版本以 `mode=request` 显式区分
> 「请求」与「报告」，两者载荷结构相同但 `chunks` 的取值约束不同（见 06 章）。

## 看门狗（Watchdog）

- Client 发出 `missing_chunks`（`mode=request`）后，若 **3 秒**内未收到任何
  `mode=report` 的回复，**必须**重发该请求。
- 重发不改变协议语义，与幂等要求兼容（见 06 章「未识别消息必须忽略，保持幂等性」）。

## 完整性校验

- **包级**：每个 UDP 包尾部携带 CRC32，校验失败直接丢弃，**不得**写入位图
- **文件级**：传输完成后，接收方计算全文件 Blake3 哈希，与请求中的 `file_hash` 比对；
  一致才进入 COMMIT 并发布完成消息（上传方向 `upload_complete`，
  下载方向 `download_complete`）
- **文件级校验失败**：接收方**必须**发布 `session_abort`（`reason` = `FILE_HASH_MISMATCH`）
  终止会话。
  这是**接收侧唯一有权发起的终止**（见 10 章「终止权」）；
  其余终止原因一律由 Server 发起。
  由于包级 CRC32 已经过校验，文件级失配通常意味着**会话参数错误**
  （如 `chunk_size` 协商不一致）而非个别数据损坏，因此**不作为可重传事件处理**。
- **秒传路径**：跳过数据传输时没有位图，完成条件为「既有文件校验通过并原子提交」
  （见 10 章「秒传路径」）。
