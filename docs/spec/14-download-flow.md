# 14 下载流程（Server → Client）

在不改变现有上传协议的前提下，支持 Client 从 Server 拉取文件，控制面与可靠性规则保持一致。

## 14.1 原则

- Client 仍为主动方
- Server 不主动向未知端口推送 UDP
- 控制面由 Client 发起
- 可靠性完全复用上传逻辑

## 14.2 会话类型扩展

```
Upload   : Client → Server
Download : Server → Client
```

在会话中引入方向标识（实现层）。

## 14.3 控制消息

- download_start（Client → Server）
- download_accept（Server → Client）
- missing_chunks（Client → Server）
- upload_complete（Server → Client）对称为 download_complete（可选）

## 14.4 UDP 行为（NAT 关键点）

- Client 必须先向 Server 发送 UDP HELLO
- Server 只能向该源端口回包
- 之后 Server 按 chunk 顺序发送，Client 记录 bitmap
