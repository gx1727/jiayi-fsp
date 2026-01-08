# 17 存储与内存模型

定义接收端在大文件传输过程中的存储与内存使用规则（强制）。

## 17.1 目标

- 支持 GB 级及以上文件传输
- 内存使用与文件大小解耦
- 乱序写入与中断恢复

## 17.2 临时文件

- 会话初始化阶段创建临时文件，包含 session_id
- 状态标记为 incomplete
- 已知 total_size 时可预分配（稀疏文件）

## 17.3 写入规则

```
offset = chunk_id * chunk_size
write(payload, offset)
```

- 写入不要求顺序，允许重复覆盖

## 17.4 内存占用

- 仅允许存在：当前 UDP payload、received bitmap、Session 状态
- 内存占用与 chunk_size 成正比，不与 total_size 成正比

## 17.5 完成与提交

- bitmap 完整后校验 hash
- 原子重命名临时文件为最终文件
- 标记 session completed
