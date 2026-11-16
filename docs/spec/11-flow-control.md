# 11 流控（Flow Control）

## 基础机制：滑动窗口

- 固定滑动窗口（window），由 Server 在 upload_accept / download_accept 中下发
- 发送规则：顺序填充与发送；达上限暂停；等待 missing_chunks 重传
- 节奏：每发送 N 个 chunk 休眠 T ms（建议 N=window，T=2–5ms）
- 确认与重传：无逐包 ACK；仅通过 missing_chunks 裁决

> 窗口大小是**初始值**，Client 允许按下方 AIMD 算法动态调整。

## 动态流控：AIMD

Client 依据 Server 周期性下发的 `progress_update` 估算丢包率，据此动态调节发送速率与窗口。

### 丢包率估算

```
loss_rate = 1.0 - (delta_received / delta_sent)
```

- `delta_received`：相邻两次 `progress_update` 之间 Server 确认的 chunk 数增量
- `delta_sent`：同一区间内 Client 实际发出的 chunk 数增量

### 调整规则

| 条件 | 动作 |
|------|------|
| 拥塞（loss_rate > 5%） | `packet_gap_us *= 1.5`（发送间隔增大，减速）<br>`window_size *= 0.7`（窗口减小） |
| 通畅（loss_rate < 1%） | `packet_gap_us -= 10us`（发送间隔减小，加速）<br>`window_size += 2`（窗口线性增大） |
| 中间区间（1% ≤ loss_rate ≤ 5%） | 保持不变 |

- `packet_gap_us` 与 `window_size` 均需设置上下限，避免退化（间隔趋近 0 或窗口趋近 0）
- 每次调整应加随机抖动（jitter），避免多个 Client 同步震荡

### 实现位置参考

- 桌面端：`jiayi-client/pc/windows/src-tauri/src/fsp/transport.rs`
- 移动端：`jiayi-client/app/lib/protocol/flow_control.dart`

### 前提条件

- AIMD **只作用于发送节奏**，不改变协议语义；任何流控参数下协议行为都应一致
- AIMD 依赖 `progress_update` 的周期性下发（建议 Server 约 500ms 一次）
- 数据面**不做**拥塞控制，AIMD 属于客户端策略而非协议层机制
