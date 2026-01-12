# 09 会话模型（Session Model）

## 生命周期

生命周期与合法状态转移以 10-state-machine 为准。

## 会话参数

### 核心字段

- session_id
- client_id
- file_id_hash
- file_size
- chunk_size
- chunk_count
- window

### 扩展字段

- mtime（文件修改时间）
- udp_port
- tcp_port
- temp_file_path
- last_activity
- state

## 持久化

- 临时文件
- bitmap
- 元数据

## 创建规则

- 同文件正在同步 → 返回现有会话
- 已存在完整文件 → 直接完成
- 否则新建

## 超时与回收

长时间无活动 → ABORTED 并清理临时文件/位图/元数据
