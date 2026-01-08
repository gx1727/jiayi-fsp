# JIAYI-FSP 中文文档

JIAYI File Sync Protocol（JIAYI-FSP）是一种为可信局域网设计的混合文件同步协议。

- 控制平面：MQTT
- 数据平面：UDP

状态：实验性（v0.1）

## 项目目标

- 在局域网环境下高效进行大文件同步
- 盘后备的 UDP 接收，避免大内存缓冲
- 通过 `client_id` 将 MQTT 主题按客户端隔离

## 非目标

- 互联网规模的跨广域网传输
- 移动端的后台长时间传输
- 不可信的点对点网络

## 架构概览

- 混合协议设计：控制平面使用 MQTT 负责会话、身份与授权；数据平面使用 UDP 进行块传输与流控。
- 协议栈与包格式：采用位图/块图管理重传与进度，支持断点续传。
- 关键文档：
  - [架构](docs/spec/01-architecture.md)
  - [控制平面（MQTT）](docs/spec/06-control-plane-mqtt.md)
  - [数据平面（UDP）](docs/spec/07-data-plane-udp.md)
  - [包格式](docs/spec/08-packet-format.md)
  - [状态机](docs/spec/10-state-machine.md)
  - [流控](docs/spec/11-flow-control.md)
  - [可靠性模型](docs/spec/12-reliability-model.md)
  - [断点续传与恢复](docs/spec/13-resume-and-recovery.md)

## 快速上手

- 准备环境：在可信局域网内部署 MQTT Broker（如 Mosquitto）。
- ACL 与身份：参考 [ACL 示例](examples/mosquitto/acl.conf) 配置主题访问与 `client_id`。
- 参考实现：阅读 [实现指南](docs/spec/25-implementation-guide.md) 与 [参考 MVP](docs/spec/26-reference-mvp.md)。
- 数据报结构：查看 [fsp_packet.md](proto/fsp_packet.md) 与 [fsp_packet.rs](proto/fsp_packet.rs)。

## 安全与身份

- 信任模型：面向可信局域网，防御目标与边界详见 [安全模型](docs/spec/20-security-model.md)。
- 授权与 ACL：详见 [ACL 与认证](docs/spec/21-acl-and-auth.md)。
- 客户端身份：详见 [Client Identity](docs/spec/23-client-identity.md)。

## 会话与可靠性

- 状态机定义：[State Machine](docs/spec/10-state-machine.md)。
- 位图与块图：[Bitmap/Chunk Map](docs/spec/18-bitmap-and-chunk-map.md)。
- 错误码与恢复策略：[Error Codes](docs/spec/19-error-codes.md)、[Resume & Recovery](docs/spec/13-resume-and-recovery.md)。

## 序列图

- 上传流程序列图：参考 [sequence-upload.puml](docs/diagrams/sequence-upload.puml)。

## 术语

- 统一术语定义见 [Glossary](docs/spec/99-glossary.md)。
- 协议总览见 [Overview](docs/spec/00-overview.md)。

## 路线图与变更

- 项目规划见 [ROADMAP.md](ROADMAP.md)。
- 版本变更见 [CHANGELOG.md](CHANGELOG.md)。

## 贡献

- 贡献流程与规范见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

- 本项目采用 Apache License 2.0，详见 [LICENSE](LICENSE)。

---

English version: [README.md](README.md)
