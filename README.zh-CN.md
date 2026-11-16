# JIAYI-FSP 中文文档

JIAYI File Sync Protocol（JIAYI-FSP）是一种为可信局域网设计的混合文件同步协议。

- 控制平面：MQTT
- 数据平面：UDP

状态：**v1.1.0（草案，规范修订中）**

本版本补全了控制面节奏语义，并修正了 v1.0.0 遗留的若干定义错误。
**规范尚未冻结**，线路格式仍可能变动。

v1.0.0 与参考实现 `jiayi-fsp-core` 0.2.0 / `jiayi-client` 1.0.0 对齐。

## 项目目标

- 在局域网环境下高效进行大文件同步
- 盘后备的 UDP 接收，避免大内存缓冲
- 通过 `client_id` 将 MQTT 主题按客户端隔离

## 非目标

- 互联网规模的跨广域网传输
- 移动端的后台长时间传输
- 不可信的点对点网络

## 仓库结构

本仓库专注于**协议本身**——规范、参考实现与开发者工具，不包含面向终端用户的产品应用。

```
proto/        协议数据结构定义（C / Rust）
docs/spec/    规范性文档——唯一事实来源
docs/         GitHub Pages 对外提供的规范源文件（构建产物不入库）
doc/          单文件版规范合集（自动生成，勿手改）
examples/     Broker ACL 与参考实现示例
tools/        规范工具（合集生成器 / 一致性检查器）+ 调试工具集
site/         协议官网源码
```

### 规范工作流

`docs/spec/` 是唯一事实来源；`doc/` 下的合集是**自动生成产物**，请勿手动编辑。

```bash
python3 tools/build-spec-compilation.py           # 重新生成单文件合集
python3 tools/check-spec-consistency.py           # 端序 / 类型 / Topic / proto 对齐检查
python3 tools/build-spec-compilation.py --check   # CI 校验模式
```

## 官网部署

官网发布到 **`gh-pages` 分支**；构建产物**不提交**到 `main`
（带 hash 的资源名会让每次改动产生上百个文件的 diff 噪音）。

```bash
cd site
npm run build        # tsc -b && vite build
npm run copy:spec    # 把 docs/spec + docs/diagrams 复制进 dist/（否则 /spec/*.md 会 404）
npm run deploy       # 发布 dist/ 到 gh-pages 分支
```

CI（`.github/workflows/deploy-pages.yml`）会在 push 到 main 时自动执行。

> **一次性仓库设置**：Settings → Pages → Source ＝ *Deploy from a branch*，
> 分支选 `gh-pages`，目录选 `/(root)`。

## 参考实现

`examples/rust/` 把规范里的**规范性要求**变成可执行断言 ——
确定性仿真（虚拟时钟 + 虚拟 Broker + 可注入故障的虚拟 UDP）加真实 UDP 回环示例。

```bash
cd examples/rust
cargo test            # 27 个核心单测 + 9 个一致性场景
cargo run -p fsp-sim  # 跑一遍全部场景并打印观测结果
```

每个场景钉住一条规范条款：零丢包不得死锁、空洞块必须补发、
`next_chunk_id` 必须是**第一个缺失块**，等等。

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
