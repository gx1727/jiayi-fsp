# 11 流控（Flow Control）

定义发送端窗口与发送节奏控制策略，目标为“足够可靠、足够简单、稳定运行”。

## 11.1 窗口模型

- 固定滑动窗口（Fixed Sliding Window）
- 窗口大小由 Server 下发（window）
- 不动态调整，不实现复杂拥塞算法

## 11.2 发送规则（Client）

1. 顺序填充窗口
2. 窗口未满持续发送
3. 达到上限暂停发送
4. 等待 missing_chunks 指令进行重传

## 11.3 发送节奏

- 每发送 N 个 chunk 休眠 T ms
- 推荐初始值：N = window，T = 2–5 ms
- 目的：避免打满 CPU/Wi‑Fi，降低丢包概率

## 11.4 确认与重传

- 无逐包 ACK
- 仅通过 missing_chunks 进行确认与重传
- 未被点名的 chunk 视为成功
