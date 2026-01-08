# 09 会话模型（Session Model）

定义会话的生命周期、参数与持久化内容。

## 9.1 会话生命周期

```
INIT → NEGOTIATING → TRANSFERRING → COMPLETED | ABORTED
```

可选扩展：TRANSFERRING → VERIFY → COMMIT → COMPLETED

## 9.2 会话参数

- session_id（uint64）
- client_id（string）
- file_id_hash（uint64）
- file_size（uint64）
- chunk_size（uint32）
- chunk_count（uint64）
- window（uint32）

## 9.3 持久化内容（Server）

- 临时文件（包含已写入数据）
- bitmap（接收进度）
- session 元数据

## 9.4 创建规则

- file_id 正在同步 → 返回现有 session
- file 已存在且完整 → 直接返回完成
- 否则创建新会话（INIT）

## 9.5 超时与回收

- last_activity 超过阈值 → ABORTED
- 清理包含临时文件、位图与元数据
