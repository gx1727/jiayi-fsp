# 10 Session 状态机

定义传输会话的状态与合法状态转移，控制面（MQTT）为唯一裁决者。

## 状态定义

```
INIT → NEGOTIATING → TRANSFERRING → VERIFY → COMMIT → COMPLETED
                          ↓
                      ABORTED (从任意状态可到达)
```

## 状态说明

| 状态 | 说明 |
|------|------|
| INIT | 初始化状态 - 会话刚创建 |
| NEGOTIATING | 协商状态 - 正在协商传输参数 |
| TRANSFERRING | 传输状态 - 正在进行文件传输 |
| VERIFY | 验证状态 - 验证传输完整性 |
| COMMIT | 提交状态 - 提交传输结果 |
| COMPLETED | 完成状态 - 传输成功完成 |
| ABORTED | 中止状态 - 传输被中止 |

## 合法状态转移

| 从状态 | 到状态 | 触发消息 |
|--------|--------|----------|
| INIT | NEGOTIATING | upload_start / download_start |
| INIT | ABORTED | session_abort |
| NEGOTIATING | TRANSFERRING | upload_accept / download_accept |
| NEGOTIATING | ABORTED | session_abort |
| TRANSFERRING | VERIFY | 接收完成，开始验证 |
| TRANSFERRING | COMPLETED | 小文件一次性传输完成 |
| TRANSFERRING | ABORTED | session_abort |
| VERIFY | COMMIT | 验证通过 |
| VERIFY | TRANSFERRING | 发现缺失 chunk，需要重传 |
| VERIFY | ABORTED | session_abort |
| COMMIT | COMPLETED | 提交成功 |
| COMMIT | ABORTED | session_abort |
| 任意状态 | ABORTED | session_abort |

## 推进规则

- **仅 MQTT 控制消息可以推进状态**
- **UDP 不得改变状态**
- **状态转换必须符合上述转移规则**

## 典型触发消息

- upload_start：INIT → NEGOTIATING
- upload_accept：NEGOTIATING → TRANSFERRING
- missing_chunks：维持 TRANSFERRING，触发重传
- upload_complete：COMMIT → COMPLETED
- session_abort：任意状态 → ABORTED
- resume_session：根据位图状态继续
