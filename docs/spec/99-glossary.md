# 99 术语表（Glossary）

## 协议相关

| 术语 | 说明 |
|------|------|
| jiayi-fsp | 协议名称（JIAYI File Sync Protocol） |
| MQTT | 控制面协议（信令 / 裁决） |
| UDP | 数据面主通道 |
| TCP fallback | 降级通道 |
| 小端序 | 本协议所有多字节整数的字节序（Little-Endian） |

## 会话与数据

| 术语 | 说明 |
|------|------|
| session | 一次文件传输会话 |
| session_id | 会话唯一标识（u32，由 Server 生成，JSON 中以字符串传输） |
| client_id | 设备唯一标识（设备级粒度） |
| chunk | 文件分块 |
| chunk_id | 分块序号（u32，从 0 开始） |
| chunk_size | 分块大小（字节，建议 1024–1350，上限 1400） |
| chunk_count | 总块数 = ceil(size / chunk_size) |
| payload_len | 载荷长度（u16） |
| flags | 数据包标志位（u16） |
| bitmap | 位图，记录各 chunk 的接收状态 |
| missing_chunks | 缺失块列表 |
| udp_ready | UDP 可用性就绪通知 |
| resume_session | 恢复会话请求 |
| missing_chunks | 缺失块列表（双向：请求 / 返回） |
| file_hash | 文件哈希，Blake3 的 64 字符十六进制串 |
| window | 滑动窗口大小 |
| AIMD | 加性增乘性减动态流控算法 |

## 控制面消息

| 消息 | 方向 | 说明 |
|------|------|------|
| login / login_ack | C→S / S→C | 设备登录认证与响应 |
| status | 双向 | 设备在线状态报告 |
| upload_start / upload_accept | C→S / S→C | 上传会话建立 |
| download_start / download_accept | C→S / S→C | 下载会话建立 |
| udp_ready | S→C | UDP 可用性通知 |
| missing_chunks | 双向 | 缺失块列表请求 / 返回 |
| progress_update | 双向 | 传输进度更新（单位：chunk 数） |
| session_pause / session_abort | 双向 | 会话暂停 / 中止 |
| resume_session / resume_ack | C→S / S→C | 会话恢复 |
| upload_complete / download_complete | S→C / 双向 | 传输完成 |
| asset_recall / asset_recall_complete | C→S / S→C | 资产撤回请求与回执 |
