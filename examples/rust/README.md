# JIAYI-FSP 参考实现（Rust）

规范 **v1.1.0** 的可执行断言。

> 本目录把规范里的**规范性要求**变成可运行的代码与断言 ——
> 而不是「跑通了顺便发现」。v1.0.0 的致命缺陷（零丢包时协议死锁、
> 空洞块永不补发、`next_chunk_id` 定义错误）在这里都有对应的回归场景。

## 结构

| crate | 作用 |
|-------|------|
| `fsp-core` | 协议核心库（**无 I/O**）：包编解码、位图、消息模型、状态机、流控、裁决 |
| `fsp-sim` | **确定性仿真**：虚拟时钟 + 虚拟 Broker + 可注入故障的虚拟 UDP + 一致性场景 |
| `server` | 真实 UDP 接收端示例（随机写盘、位图完成判定） |
| `client` | 真实 UDP 发送端示例（批量发送、CRC32 封装） |

`fsp-core` 只含协议语义，不做任何 I/O —— MQTT 与 UDP 由调用方提供。
这样协议规则可以被**确定性地**测试，不受网络时序干扰。

## 快速开始

```bash
cd examples/rust

cargo test              # 27 个核心单测 + 9 个一致性场景 + 1 个文档测试
cargo run -p fsp-sim    # 跑一遍全部场景并打印观测结果
cargo run -p fsp-sim 空洞   # 只跑名字含「空洞」的场景
```

### 真实 UDP 回环

```bash
cargo build
head -c 300000 /dev/urandom > /tmp/in.bin

./target/debug/fsp-server-example 127.0.0.1:19999 42 1200 /tmp/out.bin 300000 &
sleep 0.4
./target/debug/fsp-client-example 127.0.0.1:19999 42 1200 /tmp/in.bin

cmp /tmp/in.bin /tmp/out.bin && echo "逐字节一致"
```

## 一致性场景

每个场景对应规范里的一条**规范性要求**：

| 场景 | 规范条目 | 断言 |
|------|----------|------|
| 零丢包不卡死 | 11 章「窗口推进判据」 | 接收方**必须**每 50ms 下发裁决（**空数组也算**）；发送方不得在窗口填满后停滞 |
| 空洞块补发 | 12 / 18 章 | 低于最大已收 id 的缺失块**必须**被补发 |
| 裁决丢失时主动索要 | 11 章第 4 条 / 12 章看门狗 | 1000ms 未收到裁决时**必须**发布 `missing_chunks(mode=request)` |
| 重复报文幂等 | 19 章 | 重复 chunk **必须**静默丢弃，不污染文件 |
| CRC32 损坏丢弃 | 08 / 12 章 | CRC 失配的包**必须**丢弃且**不得**写入位图 |
| 大文件缺失分批 | 18 章 | 单条载荷**不得超过 4096 条**，`missing_count` 给出总数 |
| 断点续传起点 | 13 章 | `next_chunk_id` **必须**是第一个缺失块，**不得**是 `max_received_id + 1` |
| 秒传不发数据 | 10 章 | `file_exists = true` 时**不得**发送任何数据 |
| **下载方向端到端** | 06 / 14 章 | 接收方是 Client，**必须由 Client 发布 `download_complete`** |
| **下载方向空洞补发** | 12 / 18 章 | 下载方向的缺失裁决由 Client 下发，空洞块由 Server 补发 |
| **下载 accept 字段完整** | 06 章 | `download_accept` 必须带 `udp_port` 与 `chunk_size` |

## 双向仿真

`World` 区分**会话方向**，方向决定角色（09 章 `direction`）：

| direction | 发送侧 | 接收侧 | 完成消息 |
|-----------|--------|--------|----------|
| `upload`   | Client | Server | `upload_complete`（Server 发布） |
| `download` | Server | Client | `download_complete`（Client 发布） |

无论哪个方向，**发起会话的都是 Client**（`upload_start` / `download_start`），
**发布完成消息的都是接收侧**。

```bash
cargo run -p fsp-sim 下载   # 只跑下载方向的场景
```

## 值得注意的实现细节

- **CRC32 自实现**：08 章固定了变体（CRC-32/ISO-HDLC），而 CRC-32 与 CRC-32C
  互不兼容 —— 两侧各选一种会导致**所有包**校验失败，故障表现为
  「传输永远无法完成」，极难定位。`fsp-core::crc32` 自带校验值断言
  （`"123456789"` → `0xCBF43926`），让这条定义成为可执行断言。
- **转移表即数据**：10 章的合法状态转移表被原样编码为
  `fsp_core::session::LEGAL_TRANSITIONS`，改表只需改一处。
- **发送侧 / 接收侧双视角**：接收侧有 VERIFY / COMMIT，发送侧没有 ——
  发送侧没有位图，无法做文件级校验。状态机对两个视角分别校验。
- **重传不受在途上限约束**：11 章规定「达到上限时暂停发送**新数据**
  （重传不受此限）」。若把重传也挡在上限外，在途量已满时缺失块永远补不回来。
- **确定性**：丢包 / 乱序 / 重复由带种子的 PRNG 决定，同一场景永远同一结果。
  故障注入只作用于**首次**出现 —— 否则被丢的块永远补不回来。
- **每个视角都必须可达**：`fsp-core` 的 `每个视角都能走到_completed` 断言检查
  转移表对发送侧 / 接收侧**各自**是否连通。该断言曾暴露一个真实缺口 ——
  转移表只有发送侧的 `NEGOTIATING → TRANSFERRING`，导致
  `TRANSFERRING → VERIFY`（位图全真）在接收侧不可达。

## 未覆盖的部分

诚实起见，明确列出本参考实现**没有**覆盖的部分：

- **MQTT 传输层**：控制面语义（消息模型、Topic、载荷）已实现并测试，
  但**没有**实现 MQTT 客户端/服务端协议栈，也没有接入 Broker。
  真实部署需要自行接入（ACL 示例见 `examples/mosquitto/acl.conf`）。
- **Blake3**：仿真用 FNV-1a 摘要代替（只表达「在 VERIFY 阶段校验文件内容」
  这一**协议行为**）。生产实现必须用 Blake3。
- **TCP 降级**：线路格式在规范 v1.1.0 中仍未定义（见 ROADMAP v1.2）。
- **文件级持久化**：位图落盘 API 已实现（`save_to_file` / `load_from_file`），
  但会话元数据的持久化未实现。
- **并发**：单会话、单文件。

## 依赖

- `fsp-core`：`serde` + `serde_json`（控制面消息的 JSON 编解码）
- 其余 crate：无第三方依赖

CRC32 是**自实现**的，不依赖 `crc32fast` —— 让规范里那条「变体」定义
在代码里可见且可断言。
