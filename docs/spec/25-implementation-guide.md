# 25 实现者指南（Implementation Guide）

> 本文档为 **非规范性文档（Non-Normative）**，用于指导工程实现。
> 规范内容以 `docs/spec/` 各章为准。

## 设计哲学

- 控制慢而可靠，数据快而可丢
- 状态推进/裁决/重传在 MQTT；高吞吐在 UDP/TCP

## 推荐分层与模块划分（Client/Server 侧）

### 协议核心（jiayi-fsp-core）

协议无关的纯逻辑核心，建议独立为库（如 `jiayi-fsp-core`），供服务端与各端客户端复用：

| 模块 | 职责 |
|------|------|
| `protocol` | MQTT 消息定义、UDP 数据包编解码、Topic 构造与解析 |
| `session` | 会话状态、状态机与转移校验 |
| `bitmap` | 位图管理（AtomicU64 位集、缺失计算、持久化） |
| `storage` | SQLite 持久化（临时文件、位图、会话元数据） |
| `transport` | UDP / TCP 数据面抽象 |
| `control` | 控制面抽象（MQTT 客户端封装、消息编解码） |
| `util` | 文件哈希（Blake3）、CRC32、定位写盘 |

### 端侧实现

- 桌面端：Rust 实现（`jiayi-client/pc/windows/src-tauri`）
- 移动端：Dart 实现（`jiayi-client/app`），流控逻辑独立成 `protocol/flow_control.dart`
- 服务端：Rust 实现（`jiayi-server/windows/src-tauri`）

## Session 管理

- 每个 session_id 独立状态机；仅由 MQTT 推进；UDP 仅更新位图
- 状态可持久化
- 状态机转移必须通过 10 章定义的合法转移表校验

## UDP 数据面

- 发送端顺序发送、遵守 window、不等待 ACK
- 接收端校验后随机写盘、丢弃失败包
- **禁止整文件缓存**

## 位图实现

- bitset 表示接收情况；常驻内存周期刷盘
- 使用 `AtomicU64` 数组支持并发安全的状态更新
- 避免每 chunk 同步刷盘

## 重传策略

- Server 周期统计缺失并合并下发
- 重传优先级高于新数据

## 流控

- 基础为固定滑动窗口；Client 可按 11 章的 AIMD 动态调整发送节奏

## TCP fallback

- UDP 不可用时兜底
- 不与 UDP 并行
- 保持 chunk 与 bitmap 语义

## 移动端注意

- Android 前台服务；iOS 前台传输，后台中断，前台恢复

## 常见错误

- 在 UDP 层实现 ACK/拥塞
- 用 UDP 判定完成
- 整文件缓存
- 忽略 crc32
- 把 `file_hash` 当作 u64 传递（应为 Blake3 十六进制串）
- 把 `session_id` 序列化为 JSON 数字（应为字符串）

## MVP 顺序

1. MQTT 认证与连接
2. 单文件 UDP 上传
3. 重传机制
4. 断点续传
5. TCP fallback
6. 下载功能
