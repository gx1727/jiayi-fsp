# 22 MQTT Topic 规范（Control Plane Specification）

## 设计原则

- 单向职责
- 方向可判定
- 幂等可重复
- client_id 隔离
- session 强绑定

## Topic 格式

```
/jiayi/fsp/{role}/{client_id}/{action}
```

## 消息列表

### Client → Server

- `/jiayi/fsp/client/{client_id}/login`
- `/jiayi/fsp/client/{client_id}/upload_start`
- `/jiayi/fsp/client/{client_id}/download_start`
- `/jiayi/fsp/client/{client_id}/resume_session`
- `/jiayi/fsp/client/{client_id}/missing_chunks`
- `/jiayi/fsp/client/{client_id}/progress_update`
- `/jiayi/fsp/client/{client_id}/session_pause`
- `/jiayi/fsp/client/{client_id}/session_abort`
- `/jiayi/fsp/client/{client_id}/status`
- `/jiayi/fsp/client/{client_id}/download_complete`
- `/jiayi/fsp/client/{client_id}/asset_recall`

### Server → Client

- `/jiayi/fsp/server/{client_id}/login_ack`
- `/jiayi/fsp/server/{client_id}/upload_accept`
- `/jiayi/fsp/server/{client_id}/download_accept`
- `/jiayi/fsp/server/{client_id}/resume_ack`
- `/jiayi/fsp/server/{client_id}/missing_chunks`
- `/jiayi/fsp/server/{client_id}/progress_update`
- `/jiayi/fsp/server/{client_id}/udp_ready`
- `/jiayi/fsp/server/{client_id}/upload_complete`
- `/jiayi/fsp/server/{client_id}/session_pause`
- `/jiayi/fsp/server/{client_id}/session_abort`
- `/jiayi/fsp/server/{client_id}/status`
- `/jiayi/fsp/server/{client_id}/asset_recall_complete`

## 订阅与发布约束

- Client 仅订阅 `/jiayi/fsp/server/{client_id}/#`，发布到 `/jiayi/fsp/client/{client_id}/{action}`
- Server 订阅 `/jiayi/fsp/client/+/#`，发布到 `/jiayi/fsp/server/{client_id}/{action}`

> **层级说明**：Topic 固定为 5 层（`/jiayi/fsp/{role}/{client_id}/{action}`）。
> Server 侧的订阅过滤器必须匹配 5 层，`/jiayi/fsp/client/+` 仅 4 层，**匹配不到任何消息**——
> v0.2 草案此处有误，已在 v1.0.0 更正。
>
> 等价的更严格写法是按 action 逐个订阅（如 `/jiayi/fsp/client/+/login`），参考实现即采用此方式。

## 非法行为

- 跨 client_id 订阅/发布
- Client 发布 `server/*`
- **Client 发布 `upload_complete`** —— 上传方向的接收方是 Server，
  完成消息只能由 Server 发布
- **Server 发布 `download_complete`** —— 下载方向的接收方是 Client，
  完成消息只能由 Client 发布

**COMPLETED 状态下的应答义务（规范性）**：会话处于 COMPLETED 状态时，
接收方**必须**应答 `missing_chunks`（`mode=request`），
应答内容为**空数组**的 `mode=report`。

> v1.0.0 草案把「COMPLETED 状态处理 missing_chunks」列为非法行为，
> 这会抵消 12 章保底轮询「防止因 MQTT 消息丢失导致的状态机死锁」的目的：
> 若 `upload_complete` 在途中丢失而接收方已进入 COMPLETED，
> 接收方不得应答轮询 → 死锁依旧。本版本明确该义务。

**完成消息的发布方（规范性）**：

| 消息 | 方向 | 发布方 |
|------|------|--------|
| `upload_complete` | Server→Client | Server（上传方向的接收方） |
| `download_complete` | Client→Server | Client（下载方向的接收方） |

原因：只有接收方持有位图与临时文件，能完成「校验 hash → 原子重命名」
并判定成败。发送方通过收到该消息得知会话已提交。

> v1.0.0 草案把 `download_complete` 标为「双向」，同时又在「非法行为」中禁止
> Client 发布它，并在 14 章要求 Client 发布它 —— 三处互相矛盾，
> 已在 v1.1.0 统一。

## MQTT ACL 推荐

- **Client**: 发布 `/jiayi/fsp/client/{client_id}/#`，订阅 `/jiayi/fsp/server/{client_id}/#`
- **Server**: 订阅 `/jiayi/fsp/client/+/#`，发布 `/jiayi/fsp/server/{client_id}/#`
