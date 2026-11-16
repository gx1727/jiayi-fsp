//! JIAYI-FSP 接收端示例（真实 UDP）。
//!
//! 演示 08 章的数据包格式与 18 章的位图写入：
//! - 12 字节头部 + payload + 4 字节 CRC32，全部小端；
//! - 按 `offset = chunk_id × chunk_size` **随机写盘**（乱序到达也能正确落盘）；
//! - 用位图判定完成，**不依赖**任何逐包 ACK；
//! - CRC32 / 会话 ID / chunk_id 越界的包一律丢弃，**不得**写入位图。
//!
//! 用法：
//!
//! ```text
//! cargo run -p fsp-server-example -- <bind_addr:port> <session_id> <chunk_size> <out_file> [size]
//! ```
//!
//! 控制面（MQTT）在本示例中由命令行参数代替 —— 协议的控制面语义见
//! `fsp-core` 与规范 06 章；真实部署请接入 MQTT Broker。
//!
//! 配套发送端：`cargo run -p fsp-client-example -- <addr:port> <session_id> <chunk_size> <file>`

use fsp_core::bitmap::Bitmap;
use fsp_core::packet::{DecodeCtx, Packet};
use std::env;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::net::UdpSocket;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("用法: {} <bind_addr:port> <session_id> <chunk_size> <out_file> [size]", args[0]);
        return Err("参数不足".into());
    }
    let bind = &args[1];
    let session_id: u32 = args[2].parse()?;
    let chunk_size: u32 = args[3].parse()?;
    let out_path = &args[4];
    let size: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);

    let socket = UdpSocket::bind(bind)?;
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;
    let chunk_count = if chunk_size == 0 { 0 } else { size.div_ceil(u64::from(chunk_size)) } as u32;
    println!("监听 {bind}，会话 {session_id}，chunk_size {chunk_size}，chunk 数 {chunk_count}");

    let mut out = File::create(out_path)?;
    // 预分配（稀疏），接收方**不缓存整文件**（17 章）
    out.set_len(size)?;
    let bitmap = Bitmap::new(chunk_count);
    let ctx = DecodeCtx::new(session_id, chunk_size, size);
    let mut buf = vec![0u8; 2048];
    let mut dropped_crc = 0u64;
    let mut dropped_duplicate = 0u64;
    let mut dropped_other = 0u64;

    loop {
        let (n, _peer) = match socket.recv_from(&mut buf) {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if bitmap.is_complete() {
                    break;
                }
                continue;
            }
            Err(e) => return Err(e.into()),
        };
        let pkt = match Packet::decode(&buf[..n], &ctx) {
            Ok(p) => p,
            Err(fsp_core::packet::PacketError::CrcMismatch) => {
                dropped_crc += 1;
                continue;
            }
            Err(_) => {
                dropped_other += 1;
                continue;
            }
        };
        // 幂等：重复 chunk 静默丢弃（19 章 CHUNK_ALREADY_RECEIVED）
        if !bitmap.mark_received(pkt.chunk_id) {
            dropped_duplicate += 1;
            continue;
        }
        // 随机写盘：位置由 chunk_id 决定，与到达顺序无关
        let offset = u64::from(pkt.chunk_id) * u64::from(chunk_size);
        out.seek(SeekFrom::Start(offset))?;
        out.write_all(&pkt.payload)?;
        let done = bitmap.received_count();
        if done % 100 == 0 || bitmap.is_complete() {
            println!("已接收 {done}/{chunk_count} 块");
        }
        if bitmap.is_complete() {
            break;
        }
    }
    out.flush()?;
    println!(
        "完成：{}/{} 块，CRC 失配丢弃 {dropped_crc}，重复丢弃 {dropped_duplicate}，其他丢弃 {dropped_other}",
        bitmap.received_count(),
        chunk_count
    );
    println!("已写入 {out_path}");
    Ok(())
}
