# Changelog

## v1.1.0

协议版本：v1.1.0

**规范仍在修订中，尚未冻结。** 本版本针对 v1.0.0 的一轮系统性审计结果做修正，
补全控制面的「何时做、在什么条件下做」，并修复若干会导致实现跑不通的定义错误。

### 致命缺陷修复

- **06/11/12/14 控制面节奏**：v1.0.0 从未规定接收方下发 `missing_chunks` 的周期。
  发送方在窗口填满后「等待 missing_chunks 重传」，若接收方只在有缺失时才下发，
  **零丢包时协议死锁**。本版本规定：接收方**必须**至少每 **50ms** 下发一次
  `missing_chunks`（`mode=report`），**包括空列表**；发送方 1000ms 未收到裁决时
  **必须**发布 `mode=request` 索要。
- **06 `missing_chunks` 语义**：`chunks` 字段的含义此前从未定义
  （「我缺的块」还是「我有的块」），且请求与应答共用同一消息类型、无区分字段，
  导致上传方向的发送方**无法表达**「我还缺什么」。本版本明确
  `chunks` 恒为「发送方自己尚未收到的块」，并引入 `mode` 判别字段
  （`report` / `request`），同时新增 `missing_count`。
- **13 `resume_ack.next_chunk_id`**：v1.0.0 定义为「最大已收 chunk_id + 1」，
  对显式支持乱序写的协议是**错误定义** —— 低于最大 id 的空洞块将永远不会被重传，
  文件级校验必然失配。改为**第一个尚未收到的 chunk_id**。
- **06/22 下载方向数据面端口缺失**：`download_accept` 只带 `tcp_port` 不带 `udp_port`，
  `udp_ready` 也不带端口，而规范又禁止 Server 向未知端口发送 UDP
  —— 下载数据通路在规范层不可实现。本版本为 `download_accept` 补 `udp_port`、
  为 `upload_accept` 补 `tcp_port`、为 `udp_ready` 补 `udp_port`。
- **06/22/99/14 `download_complete` 方向三方冲突**：14 章要求 Client 发布，
  22 章禁止 Client 发布，22 章自身的 Topic 清单又允许 Client 发布。
  本版本统一为**完成消息一律由接收方发布**
  （上传方向 Server 发布 `upload_complete`，下载方向 Client 发布 `download_complete`）。
- **10 状态机归属方未定义**：v1.0.0 的转移表混合了发送侧事件与接收侧事件，
  且 4 条转移没有对应的触发消息，直接违反「仅 MQTT 控制消息可推进状态」。
  本版本定义**发送侧 / 接收侧**两个视角，明确每条转移的驱动方与触发，
  并补充 `PAUSED` 状态。

- **06 / 18 控制面**：`missing_chunks.missing_count` 改为**可空**（`u32|null`）。
  v1.0.0 的报文不带此字段，若把它设为必填则 v1.0.0 对端的报文无法反序列化 ——
  与 06 章「缺省兼容」承诺相矛盾。`null` 表示「未知」，此时发送方**必须**
  按「可能还有后续批次」处理。`chunks` 始终是权威的重传依据。

### 文档结构修正

- **规范性内容不得藏在引用块中**：v1.1.0 初稿把大量「必须」写在了 `>` 引用块内。
  但按仓库约定，引用块是**历史变更说明**，且检查器的 `normative()` 会跳过引用块 ——
  这意味着藏在引用块里的规范既不受矛盾性检查保护，读者也无法识别其效力。
  本版本把全部规范性内容迁回正文，引用块只保留历史说明；
  并新增「引用块规范性守卫」自动检查，防止回退。
- **00 章**新增「文档约定（规范性）」：明确引用块的用途、规范性用语的强度分级。

### 规范补全

- **10 秒传路径**：`file_exists` 此前无状态转移可承载，且与「bitmap 全真」的完成条件冲突。
  新增 `NEGOTIATING → VERIFY`（接收侧）与 `NEGOTIATING → COMPLETED`（发送侧），
  并明确秒传路径的完成条件是「既有文件校验通过并原子提交」。
- **10/12 移除绕过校验的转移**：`TRANSFERRING → COMPLETED（小文件一次性传输完成）`
  与 12/17/26 章要求的「校验 hash → 原子重命名」冲突，已删除。
- **06 MQTT 服务属性**：此前**完全没有**规定 QoS / retain / 遗嘱 / 持久会话 /
  MQTT 版本 / 最大报文长度，导致 `status: offline` 没有生产者、
  断线重连后无续传机制。本版本要求控制面 QoS 1、`status` retain、
  遗嘱发布 `offline`、持久会话、最大报文 ≥ 1 MiB。
- **08 CRC32 变体**：只写「CRC32」而未指定多项式 / 初值 / 反射 / 末尾异或。
  两侧各选一种会导致**所有包校验失败**，故障表现为「传输永远无法完成」。
  本版本固定为 **CRC-32/ISO-HDLC**，并明确覆盖区间为「12 字节头 + payload」。
- **11 节奏口径统一**：07 章「间隔 2–5ms」（逐包读法）与 11 章
  「每发送 N 个 chunk 休眠 T ms」（批量读法）相差约 `window` 倍。
  本版本统一为批量读法，并给出 `window` 推荐默认值 **4096** 与
  `throughput ≈ window × chunk_size / 裁决周期` 的换算关系。
- **11 AIMD 修正**：v1.0.0 用单 tick 差值估算丢包率，小窗口下量化噪声主导
  （每 tick 16 块时丢 1 块即 6.25%，越过 5% 阈值并收缩窗口，形成正反馈）。
  本版本改用滑动窗口累计值、设置最小样本量、加入滞回，
  并把调整对象从协议层 `window` 改为本地 `burst_size` / `batch_gap_us`
  —— v1.0.0 让 AIMD 直接收缩协议层窗口，与「任何流控参数下协议行为都应一致」冲突。
- **19 错误码上线路**：30 个错误码此前只存在于 19 章自身，任何消息都不携带。
  本版本规定通过 `session_abort.reason` 与完成类消息的 `code` 字段上线路。
- **09 会话模型**：新增 `direction` 字段（`upload` / `download`）——
  `missing_chunks` 是双向消息，其含义依赖会话方向，Server 侧记录必须能区分。
- **09 超时阈值**：会话活动超时固定为 **60 秒**（此前「长时间」无定义）。
- **09 session_id 唯一性**：禁止用裸时间戳低 32 位生成 u32 session_id。
- **18 缺失列表上限**：单条 `missing_chunks` 不超过 **4096 条**，超出分批下发；
  此前 `missing_chunks_limit` 的 limit 无取值、无分片协议。
- **08 包格式校验**：明确接收端**必须**校验 `payload_len` 越界、
  末块长度不符、`chunk_id` 越界；`FLAG_LAST_CHUNK` 语义明确为「可选，接收方不得依赖」。
- **07 探测包**：统一 `UDP_PROBE` / `UDP_PROBE_ACK` 命名（24 章原写作 UDP_HELLO），
  补充 PROBE 与 PROBE_ACK 以 `payload_len`（4 / 5）区分的判别规则，
  补充 Server 以 PROBE 源地址作为会话对端地址的绑定规则。
- **06 版本协商**：`login` / `login_ack` 新增 `proto_version` 字段
  （此前 08 章声称「通过 MQTT 协议版本协商」，但线上没有任何字段承载版本号）。
- **06 `download_start`**：`session_id` 改为可空（此前强制要求一个
  「由 Server 生成」却尚未生成的 ID）。
- **22 非法行为**：移除「COMPLETED 状态处理 missing_chunks」——
  该条会抵消 12 章保底轮询「防止因 MQTT 消息丢失导致状态机死锁」的目的；
  改为明确「COMPLETED 状态下接收方**必须**应答 `mode=request`」。
- **10 状态机双视角可达性**：转移表原先只有**发送侧**的 `NEGOTIATING → TRANSFERRING`。
  接收侧因此只能从 `NEGOTIATING` 直接跳到 `VERIFY`（秒传路径），
  常规传输的 `TRANSFERRING → VERIFY`（位图全真）在接收侧**不可达** ——
  接收侧会在整个传输期间停留在 `NEGOTIATING`。
  本版本补充接收侧的 `NEGOTIATING → TRANSFERRING`，并新增
  「每个视角都必须能从 INIT 走到 COMPLETED」的规范要求与自动检查。
  该缺口由参考实现的 `每个视角都能走到_completed` 断言发现。
- **10 `session_pause`**：此前在状态机中完全缺席（无状态、无转移、无 ack），
  新增 `PAUSED` 状态与 `TRANSFERRING ⇄ PAUSED` 转移。
- **20 数据面安全**：补充源地址校验、加密不得改变 chunk 边界与偏移映射的约束。
- **23 身份一致性**：`login.device_id` **必须**与 Topic 中的 `{client_id}` 一致。

### 工程

- **tools/check-spec-consistency.py**：由 6 项检查扩展到 17 项，新增
  「消息清单三方交叉（06 / 22 / 99 的名称与方向）」、
  「消息载荷 type 与消息表对齐」、
  「状态机转移引用的消息必须存在 + 会话类消息必须出现在状态机中」、
  「Topic 过滤器规范形式」、「非法行为与 Topic 清单互斥」、
  「Markdown 代码围栏必须成对」、
  「关键载荷字段守卫（accept 必须带 udp_port/tcp_port/chunk_size；
  missing_chunks 必须带 mode；resume_ack 必须带 udp_port 等）」、
  「next_chunk_id 必须是『第一个尚未收到的块』」、
  「window 默认值在各章一致」、
  「引用块规范性守卫：规范性『必须/不得/应当』必须写在正文，
  引用块仅用于历史变更说明」、
  「状态机双视角可达性：每个视角都必须能从 INIT 走到 COMPLETED」。
  - 新增的每一项守卫都经过**负向测试**：人为改坏规范后必须报错，改回后必须通过。
- **21 ACL**：修正 `+/+` 笔误为规范写法 `/jiayi/fsp/client/+/#`；补充 QoS / 持久会话 / 遗嘱说明。
- **proto/**：补注 CRC32 变体（CRC-32/ISO-HDLC）与覆盖区间。
- **site/**：官网的状态机图、时序图、消息载荷示例、可靠性说明、CRC32 说明
  同步至 v1.1.0。
- **AGENTS.md**：修正部署命令 —— `rsync --delete` 会删掉不在 `dist/` 中的
  `docs/spec/`（规范源文件，GitHub Pages 直接服务它）与 `docs/diagrams/`，
  已加 `--exclude` 保护。
- **examples/rust/**：新增**可执行的一致性套件** ——
  把规范里的**规范性要求**变成断言，而不是「跑通了顺便发现」。
  - `fsp-core`：协议核心库（无 I/O）。CRC32 **自实现**并断言规范校验值
    （`"123456789"` → `0xCBF43926`）；10 章的合法状态转移表被原样编码为
    `LEGAL_TRANSITIONS`（转移表即数据）；发送侧 / 接收侧双视角分别校验。
  - `fsp-sim`：**确定性仿真** —— 虚拟时钟 + 虚拟 Broker + 可注入故障的虚拟 UDP。
    8 个规范场景：零丢包不卡死、空洞块补发、裁决丢失时主动索要、
    重复报文幂等、CRC32 损坏丢弃、大文件缺失分批、续传起点、秒传不发数据。
  - `server` / `client`：真实 UDP 回环示例，已验证传输结果与源文件**逐字节一致**。
  - **双向仿真**：`World` 区分会话方向，方向决定角色 ——
    上传方向 Client 发送 / Server 接收；下载方向 Server 发送 / Client 接收。
    无论哪个方向，发起会话的都是 Client，**发布完成消息的都是接收侧**。
  - 共 29 个核心单测 + 14 个一致性场景；CI 新增 `conformance` 作业。
  - 新增的下载方向场景：端到端完成（钉住「Client 发布 `download_complete`」）、
    空洞补发（接收方为 Client）、`download_accept` 字段完整性、
    数据发到错误端点必须丢弃。
  - 未覆盖部分（诚实列出）：MQTT 传输层、Blake3、TCP 降级。
- **官网部署改为 gh-pages 分支**：构建产物（`docs/assets/` 等带 hash 的文件）
  不再提交到 `main` —— 否则每次改官网都产生上百个文件的 diff 噪音。
  - 新增 `site/scripts/copy-spec.mjs`：把 `docs/spec/` 与 `docs/diagrams/`
    复制进构建产物 —— 规范源文件不在 `site/dist/` 里，不复制的话
    `/jiayi-fsp/spec/00-overview.md` 会 404。
  - 新增 `.github/workflows/deploy-pages.yml`：push 到 main 时自动构建并发布到
    `gh-pages` 分支；发布前校验产物完整性（index.html / assets / spec / diagrams）。
  - `.gitignore` 新增 `docs/assets/`、`docs/index.html`、`docs/404.html`、
    `docs/favicon.svg`、`docs/logo.png`，并从 git 索引中移除（工作区文件保留）。
  - `docs/spec/`、`docs/diagrams/` **仍然入库** —— 它们是规范源文件，
    由 Pages 对外提供，不是构建产物。
  - **一次性前置设置**：Settings → Pages → Source ＝ *Deploy from a branch*，
    分支选 `gh-pages`，目录选 `/(root)`。
- **site/**：修正两处过期的版本号文案（`v1.0.0` → `v1.1.0`），
  并把「生产可用」改为「规范草案」，与 README 的状态声明一致。
- 协议版本由 v1.0.0（标注 Production Ready）回退为 **v1.1.0（草案）**：
  v1.0.0 的下载方向与恢复路径存在上述致命缺陷，不具备冻结条件。

## v1.0.0

协议版本：v1.0.0

与参考实现 `jiayi-fsp-core` 0.2.0 / `jiayi-client` 1.0.0 对齐，协议冻结。

### 规范修正

- **08 数据包格式**：端序表述由"网络字节序，小端"（自相矛盾）统一为**小端序**；补充 payload ≤ 1400 字节上限
- **06 控制面**：`session_id` 明确为 u32 **字符串**传输；`file_hash` 统一为 **Blake3 十六进制串**；`upload_accept` 增加 `file_exists` 秒传标志；`progress_update` 明确 `received`/`total` 单位为 chunk 数
- **07 数据面**：UDP 探测包 payload 由"8 字节"更正为 session_id(u32) 的 **4 字节**小端编码；移除"不做拥塞控制"的绝对表述（改为由 AIMD 承担）
- **21/22 ACL 与 Topic**：Server 订阅路径由 `/jiayi/fsp/client/+`（4 层，匹配不到任何消息）更正为 `/jiayi/fsp/client/+/#`（5 层）
- **11 流控**：新增 AIMD 动态流控（丢包率 >5% 减速 / <1% 加速）
- **12 可靠性**：新增保活轮询（2s）与看门狗（3s）机制
- **19 错误码**：按 SESSION / TRANSPORT / INTEGRITY / AUTH / TIMEOUT / STORAGE / PROTOCOL 七类补全
- **20/21 安全模型与 ACL**：Topic 格式统一带前导斜杠

### 规范新增

- **资产撤回扩展**：`asset_recall` / `asset_recall_complete` 写入 06、22 章
- **25 实现者指南**：回填 `jiayi-fsp-core` 七模块划分（protocol / session / bitmap / storage / transport / control / util）
- **26 参考 MVP**：回填状态机流转、位图与缺失计算细节
- **99 术语表**：回填完整术语与控制面消息对照表

### 工程

- `proto/` 三件套与 08 章对齐：session_id 改 u32、移除 file_id_hash、补 crc32、补充常量与端序说明
- `examples/mosquitto/acl.conf`：修正前导斜杠、补 Server 用户、说明 deny 规则的作用域陷阱
- README / ROADMAP 版本号统一为 v1.0.0

## Unreleased

- 仓库聚焦 FSP 协议本身：移除产品官网 `jiayi-web/` 与旧稿备份 `jiayi-fsp_bak/`
- 清理 `site/` 中未使用的 Vite 模板残留（`Empty.tsx`、`pages/Home.tsx`、`assets/react.svg`）
- 统一仓库换行符为 LF，新增 `.gitattributes`
- `.gitignore` 补全构建产物与依赖忽略规则
- README（中/英）与 AGENTS.md 补充协议仓库结构说明与部署流程
- 修复 `site/` 的 5 处 ESLint 报错，并补全 MQTT `status` 消息示例

## v0.1
- Initial public draft
- MQTT control plane
- UDP data plane
