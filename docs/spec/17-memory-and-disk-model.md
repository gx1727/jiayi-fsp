# 17 存储与内存模型

## 目标

- GB 级文件
- 内存与文件大小解耦
- 乱序写/中断恢复

## 临时文件

- 初始化创建
- 标记 incomplete
- 可预分配（稀疏）

## 写入

- offset=chunk_id*chunk_size
- 允许重复覆盖

## 内存

- 仅 payload、bitmap、Session 状态
- 与 chunk_size 成正比

## 完成与提交

- 校验 hash
- 原子重命名
- 标记 completed
