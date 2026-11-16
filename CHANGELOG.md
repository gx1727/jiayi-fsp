# Changelog

## v1.0.0

与参考实现 `jiayi-fsp-core` 0.2.0 / `jiayi-client` 1.0.0 对齐，协议冻结。

### 规范修正

- **08 数据包格式**：端序表述由"网络字节序，小端"（自相矛盾）统一为**小端序**；补充 payload ≤ 1400 字节上限
- **06 控制面**：`session_id` 明确为 u32 **字符串**传输；`file_hash` 统一为 **Blake3 十六进制串**；`upload_accept` 增加 `file_exists` 秒传标志；`progress_update` 明确 `received`/`total` 单位为 chunk 数
- **07 数据面**：UDP 探测包 payload 由"8 字节"更正为 session_id(u32) 的 **4 字节**小端编码；移除"不做拥塞控制"的绝对表述（改为由 AIMD 承担）
- **21/22 ACL 与 Topic**：Server 订阅路径由 `/jiayi/fsp/client/+`（4 层，匹配不到任何消息）更正为 `/jiayi/fsp/client/+/#`（5 层）
- **11 流控**：新增 AIMD 动态流控（丢包率 >5% 减速 / <1% 加速）
- **12 可靠性**：新增保活轮询（2s）与看门狗（3s）机制
- **19 错误码**：按 SESSION / TRANSPORT / INTEGRITY / AUTH / TIMEOUT / STORAGE / PROTOCOL 七类补全
- **20/21 安全模型与 ACL**：Topic 格式统一带前导斜杠

### 规范新增

- **资产撤回扩展**：`asset_recall` / `asset_recall_complete` 写入 06、22 章
- **25 实现者指南**：回填 `jiayi-fsp-core` 七模块划分（protocol / session / bitmap / storage / transport / control / util）
- **26 参考 MVP**：回填状态机流转、位图与缺失计算细节
- **99 术语表**：回填完整术语与控制面消息对照表

### 工程

- `proto/` 三件套与 08 章对齐：session_id 改 u32、移除 file_id_hash、补 crc32、补充常量与端序说明
- `examples/mosquitto/acl.conf`：修正前导斜杠、补 Server 用户、说明 deny 规则的作用域陷阱
- README / ROADMAP 版本号统一为 v1.0.0

## Unreleased

- 仓库聚焦 FSP 协议本身：移除产品官网 `jiayi-web/` 与旧稿备份 `jiayi-fsp_bak/`
- 清理 `site/` 中未使用的 Vite 模板残留（`Empty.tsx`、`pages/Home.tsx`、`assets/react.svg`）
- 统一仓库换行符为 LF，新增 `.gitattributes`
- `.gitignore` 补全构建产物与依赖忽略规则
- README（中/英）与 AGENTS.md 补充协议仓库结构说明与部署流程
- 修复 `site/` 的 5 处 ESLint 报错，并补全 MQTT `status` 消息示例

## v0.1
- Initial public draft
- MQTT control plane
- UDP data plane
