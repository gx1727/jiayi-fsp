# 23 Client Identity 与设备绑定

## 目标

- 全局唯一、稳定可复用、设备级粒度、低耦合、安全可控

## 生成

- base32(UUIDv4) 或 hash(device_uuid+salt)，持久化安全存储，不得重启重生成

## 生命周期与存储

- Android EncryptedSharedPreferences/Keystore
- iOS Keychain
- 桌面加密文件

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
