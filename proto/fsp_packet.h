#pragma once
#include <stdint.h>

struct fsp_chunk_packet {
    uint64_t session_id;
    uint64_t file_id_hash;
    uint32_t chunk_id;
    uint16_t payload_len;
    uint16_t flags;
    uint8_t  payload[];
    uint32_t crc32;
};
