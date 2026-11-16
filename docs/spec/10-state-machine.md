# 10 Session 状态机

定义传输会话的状态与合法状态转移。控制面（MQTT）负责裁决与同步。

## 状态定义

```
INIT → NEGOTIATING → TRANSFERRING → VERIFY → COMMIT → COMPLETED
          │              │  ↑                        │
          │              │  └────────────────────────┘
          │              ↓
          │           PAUSED  ←─────────────────────────┘
          ↓
       ABORTED （从任意状态可到达）
```

| 状态 | 说明 |
|------|------|
| INIT | 初始化状态 - 会话刚创建 |
| NEGOTIATING | 协商状态 - 正在协商传输参数 |
| TRANSFERRING | 传输状态 - 正在收发数据 |
| PAUSED | 暂停状态 - 会话被显式暂停，位图与临时文件保留 |
| VERIFY | 验证状态 - 正在做文件级完整性校验 |
| COMMIT | 提交状态 - 正在原子提交（重命名） |
| COMPLETED | 完成状态 - 传输成功完成 |
| ABORTED | 中止状态 - 传输被终止；终态，无出边 |

## 视角定义（规范性）

同一个会话在**发送侧**与**接收侧**各自持有一份状态视图：

| 视角 | 定义 | 上传方向 | 下载方向 |
|------|------|----------|----------|
| **发送侧** | 发送数据的一方 | Client | Server |
| **接收侧** | 持有位图与临时文件的一方 | Server | Client |

- **接收侧**使用全部状态（INIT / NEGOTIATING / TRANSFERRING / PAUSED / VERIFY / COMMIT / COMPLETED / ABORTED）。
- **发送侧**只使用 INIT / NEGOTIATING / TRANSFERRING / PAUSED / COMPLETED / ABORTED ——
  发送侧不持有位图，**不存在** VERIFY 与 COMMIT 状态：文件级校验与原子提交
  天然发生在接收侧。
- 「Server 为最终裁决者」指**终止权**：任何进入 ABORTED 的转换只能由 Server 通过
  `session_abort` 发起（发送侧收到 `session_abort` 后同样转入 ABORTED）。

## 合法状态转移

**每个状态对每个视角都要有合法转移**。下表逐视角列出：

| 从状态 | 到状态 | 视角 | 触发 |
|--------|--------|------|------|
| INIT | NEGOTIATING | 发送侧 | upload_start / download_start |
| INIT | NEGOTIATING | 接收侧 | upload_start / download_start（收到传输请求） |
| INIT | ABORTED | 双方 | session_abort |
| NEGOTIATING | TRANSFERRING | 发送侧 | upload_accept / download_accept |
| NEGOTIATING | TRANSFERRING | 接收侧 | upload_accept / download_accept（发送 accept 后进入常规传输） |
| NEGOTIATING | VERIFY | 接收侧 | 秒传：accept 携带 file_exists=true，跳过数据传输 |
| NEGOTIATING | COMPLETED | 发送侧 | 秒传：accept 携带 file_exists=true，不发送任何数据 |
| NEGOTIATING | ABORTED | 双方 | session_abort |
| TRANSFERRING | VERIFY | 接收侧 | 位图全真，进入文件级校验 |
| TRANSFERRING | PAUSED | 双方 | session_pause |
| TRANSFERRING | COMPLETED | 发送侧 | upload_complete / download_complete |
| TRANSFERRING | ABORTED | 双方 | session_abort |
| PAUSED | TRANSFERRING | 双方 | resume_session（发送侧发起）/ resume_ack（接收侧应答后） |
| PAUSED | ABORTED | 双方 | session_abort |
| VERIFY | COMMIT | 接收侧 | 文件级 Blake3 校验通过 |
| VERIFY | ABORTED | 双方 | session_abort（reason = FILE_HASH_MISMATCH） |
| COMMIT | COMPLETED | 接收侧 | 原子重命名完成，发布 upload_complete / download_complete |
| COMMIT | ABORTED | 双方 | session_abort |

**ABORTED 是终态（规范性）**：没有出边，也不存在 `ABORTED → ABORTED`。
清理动作（删除临时文件 / 位图 / 元数据）在进入 ABORTED 之后执行
（见 09 章「超时与回收」、13 章「失败与清理」）。

**每个视角都必须能从 INIT 走到 COMPLETED（规范性）**：转移表必须对**两个视角**
都给出完整路径。接收侧同样需要 `NEGOTIATING → TRANSFERRING` ——
否则它只能从 `NEGOTIATING` 直接跳到 `VERIFY`（秒传路径），
常规传输的「位图全真 → VERIFY」在接收侧**不可达**。

> v1.1.0 初稿的转移表只有发送侧的 `NEGOTIATING → TRANSFERRING`。
> 该缺口会让接收侧在正常传输期间停留在 `NEGOTIATING`，
> 使 `TRANSFERRING → VERIFY` 无法合法触发。
> `fsp-core` 的 `每个视角都能走到_completed` 断言固化了这条要求。

**两个视角的状态序列不同（规范性）**：接收侧经过 VERIFY / COMMIT，发送侧不经过
（发送侧没有位图，无法做文件级校验）。把两侧日志对齐时，
发送侧的 `TRANSFERRING → COMPLETED` 与接收侧的
`TRANSFERRING → VERIFY → COMMIT → COMPLETED` 对应**同一次完成**，
中间的 VERIFY / COMMIT 对发送侧不可见。

### 终止权

- **Server 是会话状态的唯一仲裁者**：会话参数、状态、超时的裁决都由 Server 做出。
- **`session_abort` 的发布方有两类**：
  - **Server** —— 可因任意原因终止会话（超时、ACL、状态非法、资源不足等）；
  - **接收侧** —— **仅限**检测到不可恢复的数据完整性失败
    （`reason` = `FILE_HASH_MISMATCH`）。这是接收侧唯一有权发起的终止。
- 发送侧**不得**主动发布 `session_abort`（除非它也是该会话的接收侧）。
  发送侧需要终止时，**必须**发布 `session_pause` 或等待 Server 的裁决。

## 推进规则

- **发送侧状态仅由收到的 MQTT 控制消息推进**。UDP 数据包不得改变发送侧状态。
- **接收侧状态由本地事件推进**（位图全真 / Blake3 校验通过 / 原子重命名完成），
  但接收侧**每一次推进必须紧接着通过 MQTT 向发送侧发布一条控制消息**：

  | 接收侧推进 | 必须发布的控制消息 |
  |------------|--------------------|
  | 位图出现缺失 | `missing_chunks`（`mode=report`），携带缺失 chunk 列表 |
  | 位图全真 → VERIFY | 无需单独消息；随后的 `upload_complete` / `download_complete` 隐含告知 |
  | VERIFY → COMMIT | 无需单独消息 |
  | COMMIT → COMPLETED | `upload_complete`（上传方向）/ `download_complete`（下载方向） |

- **数据面（UDP）只能更新位图**，不得直接触发状态转移。
  位图变化是接收侧推进的**输入**，不是状态转移本身。
- **状态转换必须符合上述转移规则**；任何实现都不得进入表外的状态组合。

## 秒传路径（file_exists）

当 Server 侧已存在相同 `file_hash` 的完整文件时：

1. Server **必须**在生成 `upload_accept` **之前**完成既有文件的校验：
   对已存在的同 `file_hash` 文件重新计算 Blake3 并与 `upload_start.file_hash` 比对。
2. 校验**通过**时，Server 在 `upload_accept` 中置 `file_exists = true`；
   校验**不通过**时**不得**置该标志，按常规传输路径下发 `file_exists = false`。
   （`file_exists` 是单向发布的消息字段，一旦上线即不可撤回 ——
   因此校验必须发生在发布之前。）
3. 发送侧：NEGOTIATING → COMPLETED（**不发送任何数据**，直接等待完成通知）。
4. 接收侧：NEGOTIATING → VERIFY → COMMIT → COMPLETED → 发布 `upload_complete`。

秒传路径**不产生位图**，因此「bitmap 全真」这一完成条件**不适用于**秒传路径；
它的完成条件是「既有文件校验通过并原子提交」。
这与常规路径的完成条件是同一件事，只是数据来源不同。

## 暂停与恢复

- `session_pause` 使会话进入 PAUSED；位图、临时文件与已传输数据**全部保留**。
- PAUSED 状态下双方**必须**停止发送数据面数据包。
- 恢复**只有一条路径**：发送侧发布 `resume_session`，对端回 `resume_ack`
  （含续传起点与数据面端口），双方回到 TRANSFERRING。
- `session_pause` **不是**切换开关：重复发布是幂等的
  （已处于 `PAUSED` 时不产生状态变化）。
- `session_pause` 与 `session_abort` 的区别：暂停**可恢复**，中止**不可恢复**。
- `session_pause` 与 `resume_session` 的区别：`session_pause` 是**显式暂停**，
  `resume_session` 是**异常中断后的恢复**。两者的恢复动作相同（回到 TRANSFERRING）。

## 典型触发消息

| 消息 | 视角 | 状态效果 |
|------|------|----------|
| upload_start / download_start | 发送侧 | INIT → NEGOTIATING |
| upload_accept / download_accept | 发送侧 | NEGOTIATING → TRANSFERRING（或秒传 → COMPLETED） |
| missing_chunks（mode=report） | 接收侧 | 维持 TRANSFERRING，触发补发 |
| session_pause | 双方 | TRANSFERRING ⇄ PAUSED |
| resume_session / resume_ack | 发送侧 | PAUSED → TRANSFERRING（或恢复中断的会话） |
| upload_complete / download_complete | 发送侧 | TRANSFERRING → COMPLETED |
| session_abort | 双方 | 任意状态 → ABORTED |
