# 19 错误码（Error Codes）

## 分类

- SESSION：会话级错误
- TRANSPORT：传输层错误（UDP/TCP）
- INTEGRITY：完整性校验错误
- AUTH：认证与 ACL 错误
- TIMEOUT：超时

## 建议错误码

- SESSION_NOT_FOUND：会话不存在或已终止
- SESSION_ABORTED：会话被服务端终止
- UDP_UNAVAILABLE：UDP 探测失败
- CRC32_INVALID：包级校验失败
- FILE_HASH_MISMATCH：文件级内容哈希不一致
- ACL_DENY：ACL 拒绝访问
- CLIENT_ID_INVALID：client_id 非法
- TIMEOUT_NO_ACTIVITY：长时间无活动导致超时

## 行为

- 错误记录日志
- INTEGRITY 错误包直接丢弃
- SESSION 错误触发 session_abort
