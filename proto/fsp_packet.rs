#[repr(C)]
pub struct FspChunkPacket {
    pub session_id: u64,
    pub file_id_hash: u64,
    pub chunk_id: u32,
    pub payload_len: u16,
    pub flags: u16,
}
