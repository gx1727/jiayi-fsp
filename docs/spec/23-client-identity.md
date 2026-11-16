# 23 Client Identity 与设备绑定

## 目标

- 全局唯一、稳定可复用、设备级粒度、低耦合、安全可控

## 生成

- base32(UUIDv4) 或 hash(device_uuid+salt)，持久化安全存储，不得重启重生成
- client_id 与文件哈希（Blake3）**无关**，两者不可互相推导
- client_id 一旦分配即固定，不得因设备重装、换网或升级而改变

## 生命周期与存储

- Android EncryptedSharedPreferences/Keystore
- iOS Keychain
- 桌面加密文件

## 与登录消息的一致性

- `login` 载荷中的 `device_id` **必须**与 Topic 中的 `{client_id}` 一致。
- 不一致时 Server **必须**以 `CLIENT_ID_INVALID` 拒绝登录（见 19 章）。
- 该等价关系是 ACL 隔离（21 章）与跨 client_id 审计（20 章）的基础，
  两者都依赖「Topic 中的 client_id 即为已认证的设备身份」。

## 多设备

- 1 Device ⇔ 1 client_id；1 Account ⇔ N client_id（协议不感知账号）

## 与 Session

- client_id 可并发多 session；不得跨 client_id 使用；Server 校验一致性

## 与 MQTT ACL

- 基于 client_id 的发布/订阅隔离

## 隐私

- 不得包含敏感信息；推荐随机或哈希派生

## 非法行为

- client_id 非法；跨 client_id Topic；跨 client_id session 操作
