# 19 错误码（Error Codes）

规范错误码语义，便于实现与调试。错误码用于控制面与日志，不用于逐包数据面。

## 19.1 分类

- SESSION：会话级错误
- TRANSPORT：传输层错误（UDP/TCP）
- INTEGRITY：完整性校验错误
- AUTH：认证与 ACL 错误
- TIMEOUT：超时

## 19.2 建议错误码

| Code                   | 类别       | 说明                          |
| ---------------------- | ---------- | ----------------------------- |
| SESSION_NOT_FOUND      | SESSION    | 会话不存在或已终止            |
| SESSION_ABORTED        | SESSION    | 会话被服务端终止              |
| UDP_UNAVAILABLE        | TRANSPORT  | UDP 探测失败                  |
| CRC32_INVALID          | INTEGRITY  | 包级校验失败                  |
| FILE_HASH_MISMATCH     | INTEGRITY  | 文件级内容哈希不一致          |
| ACL_DENY               | AUTH       | ACL 拒绝访问                  |
| CLIENT_ID_INVALID      | AUTH       | client_id 非法                |
| TIMEOUT_NO_ACTIVITY    | TIMEOUT    | 长时间无活动导致超时          |

## 19.3 行为建议

- 错误必须记录日志
- INTEGRITY 错误的包直接丢弃，不即时重传
- SESSION 级错误触发 session_abort
