# 23 Client Identity 与设备绑定

本章为规范性章节，定义 JIAYI-FSP 中 client_id 的生成规则、生命周期、唯一性约束、与设备 / 用户的绑定关系，以及其在安全与扩展中的作用。

## 23.1 设计目标

- 全局唯一：在同一域内不可冲突
- 稳定可复用：设备与 App 重启后保持不变
- 设备级粒度：一个物理设备对应一个 client_id
- 低耦合：不强依赖账号系统
- 安全可控：用于 ACL 与会话隔离

## 23.2 定义

```
client_id := string
```

- 推荐长度：16 ~ 64 字符
- 字符集：a-zA-Z0-9_-
- 区分大小写
- 协议不解析其内部语义

## 23.3 生成规则（建议）

首次安装生成：
```
client_id = base32(UUIDv4)
```
或：
```
client_id = hash(device_uuid + random_salt)
```

- 生成后持久化到本地安全存储
- 不得每次启动重新生成

## 23.4 生命周期

持久化位置：
- Android：EncryptedSharedPreferences / Keystore
- iOS：Keychain
- 桌面：加密配置文件

失效与重置：
- 用户主动“重置设备身份”
- 本地存储被系统清除
- 重置后视为全新设备，旧会话失效

## 23.5 多设备关系

默认模型：
```
1 Device ⇔ 1 client_id
```

存在账号系统时：
```
1 Account ⇔ N client_id
```
协议不感知账号，仅处理 client_id。

## 23.6 与 Session 的关系

- 一个 client_id 可同时存在多个 session
- session_id 不得跨 client_id 使用
- Server 必须校验：
```
session.client_id == topic.client_id
```

## 23.7 与 MQTT ACL 的关系

Broker ACL 推荐：
```
Client(client_id=X):
  publish   jiayi/fsp/client/X/#
  subscribe jiayi/fsp/server/X/#
```

## 23.8 隐私

- client_id 不应包含手机号、邮箱、明文设备序列号
- 推荐使用随机或哈希派生

## 23.9 非法行为（必须拒绝）

- client_id 为空或格式非法
- Client 订阅 / 发布他人的 client_id Topic
- Server 接受跨 client_id 的 session 操作

## 23.10 总结

- client_id 是多 Client 隔离的根基
- Topic、Session、ACL 必须围绕 client_id 构建
- 未实现本章要求的系统，视为不符合协议
