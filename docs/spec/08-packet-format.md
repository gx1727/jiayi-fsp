# 08 UDP 数据包格式规范（Packet Format）

## 数据包结构（小端序 Little-Endian）

> **端序约定**：本协议所有多字节整数字段一律采用 **小端序（Little-Endian）**。
> v0.2 草案中"网络字节序，小端"的表述自相矛盾，已在 v1.0.0 中统一为小端；
> 参考实现 `jiayi-fsp-core` 采用 `to_le_bytes` / `from_le_bytes`，与本节一致。

| 字段 | 偏移 | 大小 | 类型 | 说明 |
|------|------|------|------|------|
| session_id | 0-3 | 4 | u32 | 会话唯一标识，由 Server 生成 |
| chunk_id | 4-7 | 4 | u32 | 逻辑位置（从 0 开始） |
| payload_len | 8-9 | 2 | u16 | 载荷长度（不含头部与 CRC） |
| flags | 10-11 | 2 | u16 | 标志位 |
| payload | 12.. | N | bytes | 文件数据 |
| crc32 | 12+N | 4 | u32 | CRC32 校验码（覆盖 header+payload） |

## 头部大小

```
HEADER_SIZE = 4 + 4 + 2 + 2 = 12 字节
CRC_SIZE = 4 字节
```

## 载荷长度上限

- `payload_len ≤ chunk_size`
- 建议 `chunk_size` 取 1024–1350 字节，**上限 1400 字节**，避免 IP 分片
- 参考实现默认 `chunk_size = 1200`
- **接收端必须校验**：`payload_len ≤ 1400` 且 `payload_len ≤ 本会话协商的 chunk_size`；
  越界即按 `INVALID_PACKET` 丢弃，**不得**写入位图
- 最后一个 chunk 的 `payload_len` **必须**等于 `size - chunk_id × chunk_size`；
  不等即按 `INVALID_PACKET` 丢弃

## CRC32 变体（规范性）

本协议的 `crc32` 字段使用 **CRC-32/ISO-HDLC**（又称 CRC-32、zlib `crc32`）：

| 参数 | 取值 |
|------|------|
| 宽度 | 32 |
| 多项式 | 0x04C11DB7（反射形式 0xEDB88320） |
| 初值 | 0xFFFFFFFF |
| 输入反射 | 是 |
| 输出反射 | 是 |
| 末尾异或 | 0xFFFFFFFF |
| 校验值 | `"123456789"` → 0xCBF43926 |

**覆盖区间（规范性）**：被校验的字节是数据包的 **第 0 字节到第 `12 + payload_len - 1` 字节**，
即 **12 字节头部 + payload**，**不含**尾部 4 字节 `crc32` 字段本身。

> v1.0.0 草案只写「CRC32」而未指定变体。CRC-32/ISO-HDLC 与 CRC-32C（Castagnoli，
> 多项式 0x1EDC6F41 / 反射 0x82F63B78）互不兼容：两侧各选一种会导致
> **所有包校验失败**，而按 12 章「校验失败直接丢弃」的策略，
> 该故障表现为「传输永远无法完成」，极难定位。本版本显式固定变体。

## Flags 位定义

| 位 | 常量 | 值 | 说明 |
|----|------|-----|------|
| bit0 | FLAG_DATA | 0x0001 | 数据包标志 |
| bit1 | FLAG_RESEND | 0x0002 | 重传包标志；接收方**无需**区别处理（重传是幂等的） |
| bit2 | FLAG_LAST_CHUNK | 0x0004 | 最后一个 chunk 标志，置位时该包的 `chunk_id` = `chunk_count - 1`；**可选**，接收方**不得**依赖它判断完成（完成以位图为准） |
| bit3 | FLAG_PROBE | 0x0008 | 探测包标志；PROBE 与 PROBE_ACK 共用，以 `payload_len` 区分（4 / 5） |
| bit4-15 | RESERVED | 0x0000 | 保留位；接收方**必须**忽略未知标志位 |

## 数据包示例

**普通数据包**:
```
session_id: 0x00000001
chunk_id: 0x00000010
payload_len: 1024
flags: 0x0001 (FLAG_DATA)
payload: [1024 bytes data]
crc32: [4 bytes checksum]
```

**探测包（PROBE）**:
```
session_id: 0x00000001
chunk_id: 0x00000000
payload_len: 4
flags: 0x0008 (FLAG_PROBE)
payload: [session_id in little-endian, 4 bytes]
crc32: [4 bytes checksum]
```

> 探测包 payload 为 `session_id`（u32）的 **4 字节小端编码**。
> v0.2 草案的 07 章曾写作"8 字节"，已在本版本更正为 4 字节。

**探测应答包（PROBE_ACK）**:
```
session_id: 0x00000001
chunk_id: 0x00000000
payload_len: 5
flags: 0x0008 (FLAG_PROBE)
payload: [session_id in little-endian, 4 bytes][status: 0x00]
crc32: [4 bytes checksum]
```

PROBE_ACK 的 payload 为 5 字节：4 字节 session_id + 1 字节状态码
（0 = 成功，1 = 失败）。
**PROBE 与 PROBE_ACK 共用 `flags.bit3`，以 `payload_len` 区分**（4 / 5）。

## 错误处理

任一关键字段不匹配或校验失败均丢弃。校验失败的数据包**不得**标记位图。

- `payload_len` 越界（> 1400 或 > 协商 chunk_size）→ `INVALID_PACKET`，丢弃
- `chunk_id` 越界（≥ chunk_count）→ `INVALID_CHUNK`，丢弃
- `session_id` 不匹配 → `SESSION_ID_MISMATCH`，丢弃
- CRC32 校验失败 → `CRC32_INVALID`，丢弃

## 扩展

- flags 预留位（bit4–15）；接收方**必须**忽略未知标志位
- 不兼容扩展通过 MQTT 协议版本协商：`login` 携带 `proto_version`，
  Server 在 `login_ack` 中回填选定的版本；Client 不支持该版本时**必须**中止会话
  （见 06 章）
- **版本比较规则（规范性）**：版本号形如 `"major.minor.patch"`，
  按 major → minor → patch 逐段**数值**比较。
  - Client 声明其**最高**支持的版本；Server **应当**回填两者都支持的**最高**版本。
  - Server 回填的版本其 major **必须**等于 Client 声明的 major
    —— **major 不同即不兼容**，Client **必须**中止会话，
    并以 `login_ack.code` = `PROTO_VERSION_UNSUPPORTED` 报告。
  - major 相同、minor 或 patch 不同即视为兼容：新增字段一律可选，
    接收方**必须**忽略不认识的字段。
