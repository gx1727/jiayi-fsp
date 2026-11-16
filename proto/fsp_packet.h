#pragma once
#include <stdint.h>

/*
 * JIAYI-FSP UDP 数据包格式
 * 规范来源: docs/spec/08-packet-format.md  (协议版本 v1.1.0)
 *
 * 全部多字节整数采用小端序 (Little-Endian)。
 * 头部 12 字节 + payload + 尾部 4 字节 CRC32。
 *
 * 偏移  字段          类型     说明
 * 0-3   session_id   uint32_t 会话唯一标识，由 Server 生成
 * 4-7   chunk_id     uint32_t 逻辑位置（从 0 开始）
 * 8-9   payload_len  uint16_t 载荷长度（不含头部与 CRC）
 * 10-11 flags        uint16_t 标志位
 * 12..  payload      uint8_t[] 文件数据
 * End-4 crc32        uint32_t CRC32 校验码（覆盖 header+payload）
 *
 * CRC32 变体: CRC-32/ISO-HDLC (zlib crc32)
 *   多项式 0x04C11DB7 (反射 0xEDB88320), 初值 0xFFFFFFFF,
 *   输入输出均反射, 末尾异或 0xFFFFFFFF。
 *   校验值: "123456789" -> 0xCBF43926
 *   覆盖区间: 第 0 字节到第 (12 + payload_len - 1) 字节, 即 header+payload,
 *             不含尾部 crc32 字段本身。
 */

#define FSP_HEADER_SIZE 12
#define FSP_CRC_SIZE 4
#define FSP_MAX_PAYLOAD_LEN 1400

/* Flags 位定义 */
#define FSP_FLAG_DATA       0x0001
#define FSP_FLAG_RESEND     0x0002
#define FSP_FLAG_LAST_CHUNK 0x0004
#define FSP_FLAG_PROBE      0x0008

struct fsp_chunk_packet {
    uint32_t session_id;
    uint32_t chunk_id;
    uint16_t payload_len;
    uint16_t flags;
    uint8_t  payload[];
    uint32_t crc32;
};
