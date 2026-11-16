# 11 流控（Flow Control）

## 基础机制：滑动窗口

- **滑动窗口（`window`）是「在途上限」**：已发送但尚未被对端确认接收的 chunk 数上限。
- `window` 由 Server 在 `upload_accept` / `download_accept` 中下发。
- **协议层参数**（互操作相关）：

  | 参数 | 归属 | 约束 |
  |------|------|------|
  | `window` | Server 下发 | **Client 不得擅自修改**；**规范默认值 4096** |
  | `chunk_size` | 协商确定 | 1024–1350，上限 1400（见 08 章） |

- **无逐包 ACK**：可靠性完全由 `missing_chunks` 裁决提供（见 12 章）。

### 窗口与吞吐的关系（规范性）

稳态吞吐上限约为

```
throughput ≈ window × chunk_size / 裁决周期
```

以 `chunk_size = 1200`、裁决周期 50ms 计：

| window | 在途数据量 | 吞吐上限 |
|--------|-----------|----------|
| 1024 | 1.2 MB | 约 24 MB/s（≈195 Mbps） |
| 4096 | 4.9 MB | 约 98 MB/s（≈786 Mbps） |
| 8192 | 9.8 MB | 约 196 MB/s（≈1.5 Gbps） |

Server **应当**依据目标链路速率与 `chunk_size` 选取 `window`。
在 1 Gbps 局域网上的推荐取值为 **4096**。

## 发送节奏

- 发送方**必须**按**批次**发送：连续发送 `burst_size` 个 chunk（批内**不**逐包停顿），
  批间隔 `batch_gap_us`。
- `burst_size` 初值 = Server 下发的 `window`；`batch_gap_us` 初值 = **2000 µs**。
- `burst_size` **必须** ≤ `window`。
- 发送方**必须**持续推进批次，直到 `chunk_count` 全部发出；
  **唯一**的停顿条件是在途量触及 `window` 上限（见「窗口推进判据」第 3 条）。

> **口径统一**：v1.0.0 草案中，07 章写作「间隔 2–5ms」（逐包读法），
> 11 章写作「每发送 N 个 chunk 休眠 T ms」（批量读法），二者相差约 `window` 倍。
> 本版本统一为**批量读法**：间隔施加在**批**与**批**之间，而非包与包之间。

## 窗口推进判据（规范性）

这是发送方在「无逐包 ACK」下继续推进的**唯一**依据：

1. 发送方持续发送新 chunk，直到 `chunk_count` 全部发出。
2. 发送方**必须**以最近一次 `missing_chunks`（`mode=report`）裁决中报告的缺失列表
   作为重传依据，**重传优先级高于新数据**。
3. 发送方**必须**维护「已发送但尚未被对端确认接收」的 chunk 数估计
   （= 已发送数 − 最近一次裁决中对方已接收数），记为**在途量**。
   在途量**不得超过** `window`；达到上限时，发送方**必须**暂停发送**新数据**
   （重传不受此限），直至收到下一次裁决后重新计算。
   批次内部**不**因在途量触顶而中断 —— 一旦开始发送一个批次，**必须**发完该批次。
4. **裁决超时**：发送方距上一次裁决已超过 **1000 ms** 且仍在等待时，
   **必须**发布 `missing_chunks`（`mode=request`）主动索要裁决。
5. **裁决节奏**（接收侧义务）：
   - 接收方**必须**至少每 **50 ms** 下发一次 `missing_chunks`（`mode=report`），
     **即使缺失列表为空也必须下发**（空数组是「可以继续推进」的裁决）。
   - 接收方收到 `missing_chunks`（`mode=request`）时，**必须**在 **50 ms** 内
     以 `mode=report` 应答。

**空数组裁决是必需的（规范性）**：若接收方只在「有缺失」时才下发，发送方将在窗口填满后
永远等不到裁决，协议死锁。因此**周期性下发空数组裁决**是规范性要求，
而非优化建议。

## 动态流控：AIMD（推荐性）

本节描述**发送端本地策略**，**不属于互操作要求**。
两端实现**可以**采用完全不同的策略，协议行为**必须**保持一致。

**发送侧**依据对端周期性下发的 `progress_update` 估算链路状况，
据此调节 `burst_size` 与 `batch_gap_us`。

上传方向的发送侧是 Client，下载方向的发送侧是 Server ——
本节对**两个方向同等适用**。

### 丢包率估算

```
loss_rate = 1.0 - (delta_received / delta_sent)
```

- `delta_received`：相邻两次 `progress_update` 之间 Server 确认的 chunk 数增量
- `delta_sent`：同一区间内 Client 实际发出的 chunk 数增量

**估算约束（规范性）**：

- 估算**必须**基于**滑动窗口内累计值**（窗口长度**建议**为最近 20 个 tick），而非单 tick 差值；
- 样本量不足时（`delta_sent < 16`）**不得**据此调整；
- `delta_sent` 为 0 时**不得**触发除法。

> **v1.1.0 修正**：v1.0.0 直接用单次 tick 的差值估算丢包率，在小窗口下会被
> 量化噪声主导 —— 例如每 tick 仅发送 16 块时，丢 1 块即为 6.25%，
> 直接越过 5% 拥塞阈值并触发窗口收缩，形成正反馈。

### 调整规则

| 条件 | 动作 |
|------|------|
| 拥塞（loss_rate > 5%） | `batch_gap_us = min(max(batch_gap_us × 1.5, 200), 100000)`<br>`burst_size = max(burst_size × 0.7, min(64, window))` |
| 通畅（loss_rate < 1%） | `batch_gap_us = max(batch_gap_us - 10, 0)`<br>`burst_size = min(burst_size + 2, window)` |
| 中间区间（1% ≤ loss_rate ≤ 5%） | 保持不变 |

- `batch_gap_us` 范围 **[0, 100000] µs**；
  `burst_size` 范围 **[min(64, window), window]** —— 下界取 `min(64, window)`，
  以免 `window < 64` 时下界越过上界。
- 每次调整**应当**加入随机抖动（jitter），避免多个 Client 同步震荡。

> **v1.1.0 修正**：v1.0.0 的 AIMD 直接收缩协议层的 `window`，
> 这与「任何流控参数下协议行为都应一致」相冲突 ——
> 窗口收缩会改变在途上限，进而改变发送方的阻塞行为。
> 本版本改为调整**发送端本地**的 `burst_size` / `batch_gap_us`，
> 协议层 `window` **保持不变**。

### 前提条件

- AIMD **只作用于发送节奏**，不改变协议语义；任何流控参数下协议行为都应一致
- AIMD 依赖 `progress_update` 的周期性下发（发送侧约每 500ms 收到一次）
- 数据面**不做**拥塞控制，AIMD 属于客户端策略而非协议层机制

### 实现位置参考

- 桌面端：`jiayi-client/pc/windows/src-tauri/src/fsp/transport.rs`
- 移动端：`jiayi-client/app/lib/protocol/flow_control.dart`
