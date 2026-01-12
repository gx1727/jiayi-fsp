# jiayi-fsp 协议规范合集

本文整合 docs/spec 目录下全部章节内容为单文件，便于整体阅读与归档。若存在内容差异，以分章节原文件为准。

## 00 概览（Overview）

jiayi-fsp 是面向可信局域网的混合文件同步协议。通过“控制面 MQTT + 数据面 UDP”的双通道协作，在保证高吞吐的同时，实现可中断、可恢复、可裁决的文件同步能力。

- 局域网内大文件（照片/视频）高效同步
- 不中断可恢复、乱序可写、内存与文件大小解耦
- 控制与数据分离，可靠性由控制面裁决
- 控制面：MQTT；数据面：UDP；位图进度；会话恢复；NAT 探测
- 角色：Client 发起与发送；Server 裁决与提交
- 草案版本：v0.1（实验性）

## 01 架构（Architecture）

分层架构：应用层 → 控制层（MQTT） → 数据层（UDP/TCP） → 网络层（LAN）。
- 控制面：会话建立/协商、缺失统计与重传、udp_ready、完成与终止
- 数据面：chunk 传输、crc32 校验、随机写盘与位图
- 原则：状态机仅由控制面推进，数据面不得改变会话状态，Server 为最终裁决者

## 02 术语（Terminology）

- session_id（uint64）、file_id_hash（uint64）、chunk_size（bytes）、chunk_id（uint32）
- payload_len（uint16）、flags（uint16）、bitmap、window（uint32）
- client_id（string）、missing_chunks（uint32[]）、udp_ready、resume_session

## 03 设计目标（Design Goals）

- 高效大文件同步、可恢复、乱序写盘、重传裁决清晰
- 内存与文件大小解耦；分层清晰；发送端策略简化与稳定

## 04 非目标（Non-Goals）

- 公网穿透与互联网规模传输
- 复杂拥塞控制与逐包 ACK
- 多租户不可信网络
- 移动端长期后台无 UI 传输

## 05 协议栈（Protocol Stack）

应用层 → 控制层（MQTT：状态机/重传裁决/会话管理） → 数据层（UDP/TCP：chunk 传输与兜底） → 网络层（LAN）。
- 控制面拥有最终裁决权
- 数据面仅负责搬运与落盘

## 06 控制面（MQTT）

控制面是会话状态推进与可靠性裁决的唯一通道。Topic 规范详见 22-topic-spec。
- 职责（完整列表）：
  - 会话建立：upload_start（Client→Server）、upload_accept（Server→Client）
  - UDP 探测：udp_ready（Server→Client）
  - 传输控制：missing_chunks（Server→Client）
  - 会话终止：upload_complete、session_abort（Server→Client）
  - 恢复：resume_session（Client→Server）
  - 下载（可选）：download_start、download_accept、download_complete
- 载荷约束：
  - 所有消息必须携带 session_id（String 类型）
  - upload_start：file_id（String）、file_name（String）、size（u64）、mtime（u64）、chunk_size（u32）
  - upload_accept：chunk_count（u32）、window（u32）、udp_port（u16）
  - missing_chunks：chunks（Vec<u32>）
  - 未识别消息必须忽略，保持幂等性
- 订阅发布约束：Client 仅 client/{client_id}/... 发布，server/{client_id}/... 订阅；Server 订阅 client/+ 发布 server/{client_id}/...
- 与状态机关系：映射以 10-state-machine 为准；Topic 语义与方向详见 22-topic-spec

## 07 数据面（UDP / TCP Fallback）

- UDP 原则：无连接、无逐包 ACK、允许丢包与乱序；payload_len ≤ chunk_size；crc32 包级校验
- 发送端：按 window 连续发送；间隔 2–5ms；不做拥塞控制；仅对 missing_chunks 重传
- 接收端：校验头与 crc32；随机写盘；标记位图；不直接推进状态机
- UDP 可用性探测：
  - 交互流程：Client 发送 UDP_PROBE → Server 返回 UDP_PROBE_ACK → Server 通过 MQTT 发送 udp_ready
  - PROBE 数据包：flags.bit3=1，payload 为 session_id 的 8 字节小端编码
  - PROBE_ACK 数据包：flags.bit3=1，payload 为 session_id + 1 字节状态码（0=成功，1=失败）
  - 超时：Client 等待 PROBE_ACK 超时（建议 2s）则判定 UDP 不可用，回退到 TCP
- TCP 降级：UDP 不可用或异常时启用；保持 chunk 与 bitmap 语义

## 08 UDP 数据包格式规范（Packet Format）

整体结构（网络字节序）：
| session_id(64) | file_id_hash(64) | chunk_id(32) | payload_len(16) | flags(16) | payload(N) | crc32(32) |
- session_id：会话唯一标识；file_id_hash：防跨文件误写；chunk_id：逻辑位置；payload_len ≤ chunk_size；flags 位掩码：
  - bit0 DATA、bit1 RESEND、bit2 LAST_CHUNK、bit3 PROBE、4–15 RESERVED=0
- payload：文件数据；crc32 覆盖 header+payload
- 错误处理：任一关键字段不匹配或校验失败均丢弃
- 扩展：flags 预留位；不兼容扩展通过 MQTT 协议版本协商

## 09 会话模型（Session Model）

- 生命周期与合法状态转移以 10-state-machine 为准
- 会话参数：
  - 核心字段：session_id、client_id、file_id_hash、file_size、chunk_size、chunk_count、window
  - 扩展字段：mtime（文件修改时间）、udp_port、tcp_port、temp_file_path、last_activity、state
- 持久化：临时文件、bitmap、元数据
- 创建规则：同文件正在同步 → 返回现有会话；已存在完整文件 → 直接完成；否则新建
- 超时与回收：长时间无活动 → ABORTED 并清理临时文件/位图/元数据

## 10 Session 状态机

状态定义：
INIT → NEGOTIATING → TRANSFERRING → COMPLETED | ABORTED
可选扩展：TRANSFERRING → VERIFY → COMMIT → COMPLETED
- 推进规则：仅 MQTT 控制消息；UDP 不得改变状态
- 合法转移：INIT→NEGOTIATING；NEGOTIATING→TRANSFERRING；TRANSFERRING→COMPLETED/ABORTED/VERIFY；VERIFY→COMMIT/TRANSFERRING；COMMIT→COMPLETED；ANY→ABORTED
- 典型触发：upload_start、upload_accept、missing_chunks、upload_complete、session_abort、resume_session

## 11 流控（Flow Control）

- 固定滑动窗口（window），由 Server 下发
- 发送规则：顺序填充与发送；达上限暂停；等待 missing_chunks 重传
- 节奏：每发送 N 个 chunk 休眠 T ms（建议 N=window，T=2–5ms）
- 确认与重传：无逐包 ACK；仅通过 missing_chunks 裁决

## 12 可靠性模型（Reliability）

- 控制面统计缺失并下发 missing_chunks；接收端乱序写盘与位图；crc32 包级完整性；文件级内容 hash 校验
- 重传：UDP 不即时重传；聚合缺失后统一下发；重传优先级高于新数据
- 中断与恢复：不回滚已写入；恢复后加载位图并补传缺失
- 完成与提交：bitmap 全真 → 校验 hash → 原子重命名 → upload_complete

## 13 中断与恢复（Resume & Recovery）

- 中断处理：可随时中断；保留已写入与位图
- 恢复流程：Client 发送 resume_session；Server 加载位图→计算缺失→下发 missing_chunks
- 失败与清理：长时间未恢复可清理，删除临时文件/位图/元数据

## 14 下载流程（Server → Client）

- 原则：Client 主动发起；Server 不向未知端口推送 UDP；控制面与可靠性机制与上传一致
- 会话类型：Upload（C→S）、Download（S→C）
- 下载控制消息：
  - download_start（Client→Server）：请求下载文件
  - download_accept（Server→Client）：接受下载请求，携带会话参数
  - missing_chunks（Server→Client）：下发缺失 chunk 列表
  - download_complete（Server→Client）：可选，下载完成通知

### 下载流程时序

1. **会话建立**
   - Client 发送 download_start（Topic: jiayi/fsp/client/{client_id}/download_start）
   - Server 验证文件权限，返回 download_accept（Topic: jiayi/fsp/server/{client_id}/download_accept）
   - 参数：session_id、chunk_count、window、tcp_port

2. **UDP 探测（NAT 场景）**
   - Client 发送 UDP_PROBE 到 Server 的 udp_port
   - Server 记录 Client 源端口，返回 UDP_PROBE_ACK
   - Server 通过 MQTT 发送 udp_ready

3. **数据传输**
   - Server 按 window 连续发送 UDP chunk（与上传方向相反）
   - Client 接收、校验、随机写盘、更新位图
   - Server 周期统计缺失，下发 missing_chunks

4. **缺失重传**
   - Client 发送 missing_chunks 请求（或 Server 主动下发）
   - Server 优先重传缺失 chunk

5. **完成与终止**
   - Client 检测位图全真 → 校验文件 hash
   - 校验通过 → 原子重命名 → 发送 download_complete
   - 失败 → 发送 session_abort

### UDP 行为（NAT）

- Client 主动发送 UDP_PROBE 建立 NAT 映射
- Server 仅向 Client 源端口回包
- Server 发送 chunk 时使用 Client 记录的端口

### 与上传对称性

| 阶段 | 上传（C→S） | 下载（S→C） |
|------|-------------|-------------|
| 会话建立 | upload_start | download_start |
| 参数下发 | upload_accept | download_accept |
| 数据方向 | Client→Server | Server→Client |
| 缺失请求 | missing_chunks | missing_chunks |
| 完成通知 | upload_complete | download_complete |

## 15 NAT 与网络限制

- 支持：单一局域网；二级路由（条件支持）；跨公网 NAT/蜂窝网络不支持
- 假设：同一局域网；Client 可主动向 Server 发起 TCP/UDP；不提供穿透
- 建议：Client 先发 UDP 建 NAT 映射；mDNS 可能失效需手动配置

## 16 移动端后台限制

- Android：Foreground Service；展示同步状态
- iOS：仅前台；后台视为中断；回前台需 resume_session

## 17 存储与内存模型

- 目标：GB 级文件；内存与文件大小解耦；乱序写/中断恢复
- 临时文件：初始化创建；标记 incomplete；可预分配（稀疏）
- 写入：offset=chunk_id*chunk_size；允许重复覆盖
- 内存：仅 payload、bitmap、Session 状态；与 chunk_size 成正比
- 完成与提交：校验 hash、原子重命名、标记 completed

## 18 位图与块图（Bitmap & Chunk Map）

- 位图：每 chunk 1bit；大小=total_chunks/8；必须持久化
- 刷盘：每 1s 或每 100 chunk 刷盘（建议）
- 缺失：expected-[received] → missing，通过 missing_chunks 下发与重传

## 19 错误码（Error Codes）

- 分类：SESSION、TRANSPORT、INTEGRITY、AUTH、TIMEOUT
- 建议：SESSION_NOT_FOUND、SESSION_ABORTED、UDP_UNAVAILABLE、CRC32_INVALID、FILE_HASH_MISMATCH、ACL_DENY、CLIENT_ID_INVALID、TIMEOUT_NO_ACTIVITY
- 行为：错误记录日志；INTEGRITY 错误包直接丢弃；SESSION 错误触发 session_abort

## 20 安全模型（Security Model）

- 假设：可信局域网；Broker 与 Server 受控；不穿透公网
- 控制面：建议启用 TLS；用户名/密码或客户端证书；ACL 强制隔离 client_id 发布/订阅
- 数据面：UDP 使用 CRC32（完整性非安全性）；文件级内容 hash；数据加密由实现层自选
- 状态与权限：UDP 会话必须绑定 MQTT Session；Server 不向未知端口发送 UDP；TCP 降级需重校验权限
- 审计：跨 client_id 访问与 session_abort 原因

## 21 ACL 与认证

- 目标：多 Client 隔离；防止跨 client_id；服务端裁决边界明确
- ACL 推荐：Client publish jiayi/fsp/client/{client_id}/#、subscribe jiayi/fsp/server/{client_id}/#；Server subscribe client/+、publish server/{client_id}/#
- 认证：用户名/密码、TLS、客户端证书可与 client_id 绑定
- 强约束：Broker 拒绝跨 client_id；Server 不得向错误 client_id 发布；Session 必须绑定唯一 client_id
- 审计：记录跨 client_id 尝试与 session_abort

## 22 MQTT Topic 规范（Control Plane Specification）

- 设计原则：单向职责、方向可判定、幂等可重复、client_id 隔离、session 强绑定
- 命名：jiayi/fsp/{role}/{client_id}/{action}（role=client|server）
- 订阅与发布：Client 订阅 server/{client_id}/#，发布 client/{client_id}/upload_start 与 resume_session；Server 订阅 client/+，发布 server/{client_id}/...
- 会话建立：
  - upload_start（Client→Server）Topic: jiayi/fsp/client/{client_id}/upload_start，Payload: session_id、file_id_hash、file_size、chunk_size
  - upload_accept（Server→Client）Topic: jiayi/fsp/server/{client_id}/upload_accept，Payload: session_id、chunk_count、window
- UDP 可用性：udp_ready（Server→Client）Topic: jiayi/fsp/server/{client_id}/udp_ready，Payload: session_id
- 传输阶段：missing_chunks（Server→Client）Topic: jiayi/fsp/server/{client_id}/missing_chunks，Payload: session_id、missing_chunks[]
- 完成与终止：upload_complete（Server→Client）；session_abort（Server→Client）
- 恢复：resume_session（Client→Server）
- Topic 与状态机映射：详见原表（06 控制面与 10 状态机一致）
- 非法行为：跨 client_id 订阅/发布、Client 发布 server/*、Client 主动发布 upload_complete、COMPLETED 状态处理 missing_chunks
- MQTT ACL 建议：Client publish client/{client_id}/#；subscribe server/{client_id}/#

## 23 Client Identity 与设备绑定

- 目标：全局唯一、稳定可复用、设备级粒度、低耦合、安全可控
- 生成：base32(UUIDv4) 或 hash(device_uuid+salt)，持久化安全存储，不得重启重生成
- 生命周期与存储：Android EncryptedSharedPreferences/Keystore；iOS Keychain；桌面加密文件
- 多设备：1 Device ⇔ 1 client_id；1 Account ⇔ N client_id（协议不感知账号）
- 与 Session：client_id 可并发多 session；不得跨 client_id 使用；Server 校验一致性
- 与 MQTT ACL：基于 client_id 的发布/订阅隔离
- 隐私：不得包含敏感信息；推荐随机或哈希派生
- 非法行为：client_id 非法；跨 client_id Topic；跨 client_id session 操作

## 24 端到端时序（MQTT + UDP）

- 会话建立：upload_start → upload_accept（含会话参数）
- UDP 探测：UDP_HELLO → UDP_HELLO_ACK → udp_ready
- 数据传输：Client 连续发送 UDP chunk；Server 乱序随机写盘
- 缺失统计与重传：Server 下发 missing_chunks；Client 按指令重传
- 校验与提交：Server 校验 hash、原子重命名、upload_complete
- 中断与恢复：resume_session → missing_chunks
- 下载对称：控制面一致，仅 UDP 方向互换
- 状态机与合法转移以 10-state-machine 为准

## 25 实现者指南（Implementation Guide）

- 设计哲学：控制慢而可靠，数据快而可丢；状态推进/裁决/重传在 MQTT；高吞吐在 UDP/TCP
- 推荐分层与模块划分（Client/Server 侧）
- Session 管理：每个 session_id 独立状态机；仅由 MQTT 推进；UDP 仅更新位图；状态可持久化
- UDP 数据面：发送端顺序发送、遵守 window、不等待 ACK；接收端校验后随机写盘、丢弃失败包；禁止整文件缓存
- 位图实现：bitset 表示接收情况；常驻内存周期刷盘；避免每 chunk 同步刷盘
- 重传策略：Server 周期统计缺失并合并下发；重传优先级低于新数据
- TCP fallback：UDP 不可用时兜底，不并行；保持 chunk 与 bitmap 语义
- 移动端注意：Android 前台服务；iOS 前台传输，后台中断，前台恢复
- 常见错误：在 UDP 层实现 ACK/拥塞；用 UDP 判定完成；整文件缓存；忽略 crc32
- MVP 顺序：MQTT→单文件 UDP→重传→恢复→TCP fallback

## 26 最小参考实现（Reference MVP）

- 目标：单文件上传；控制面 MQTT；数据面 UDP；随机写盘与恢复
- 状态：INIT→NEGOTIATING→TRANSFERRING→COMPLETED|ABORTED（仅由 MQTT 推进）
- Chunk：chunk_size=1200；chunk_count=ceil(file_size/chunk_size)
- UDP 接收：verify_crc→match_session→pwrite(offset=chunk_id*chunk_size)→bitmap.mark
- 位图：周期刷盘
- 缺失计算与下发：遍历位图→missing[]→MQTT 下发
- 重传：Client 接收 missing_chunks 并重传
- 恢复：Server 持久化临时文件/位图/元数据；恢复时加载与下发缺失
- 完成：bitmap 全真→校验 hash→rename→upload_complete
- 验收清单：乱序可完成；中断可继续；内存不随文件增长；位图丢失可恢复；无 UDP ACK
- 下一阶段：下载、TCP fallback、并发 session、加密与认证

## 99 术语表（Glossary）

- jiayi-fsp：协议名称
- MQTT：控制面协议；UDP：数据面主通道；TCP fallback：降级通道
- session、session_id、client_id、chunk、chunk_id、chunk_size、payload_len、flags、bitmap、missing_chunks、udp_ready、resume_session

