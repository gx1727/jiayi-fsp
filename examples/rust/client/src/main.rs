//! JIAYI-FSP 发送端示例（真实 UDP）。
//!
//! 演示 07 / 08 章的数据面：
//! - 按**批次**发送（`burst_size` 块一批，批间隔 `batch_gap_us`）—— 见 11 章；
//! - 每个包是 12 字节头部 + payload + 4 字节 CRC32，全部小端；
//! - **不等待逐包 ACK**：控制面的缺失裁决由 MQTT 提供（本示例省略）。
//!
//! 用法：
//!
//! ```text
//! cargo run -p fsp-client-example -- <addr:port> <session_id> <chunk_size> <file>
//! ```
//!
//! 配套接收端：`cargo run -p fsp-server-example -- <bind_addr:port> <session_id> <chunk_size> <out_file> [size]`

use fsp_core::packet::{flags, Packet};
use std::env;
use std::net::UdpSocket;
use std::time::Duration;

/// 每批连续发送的块数（11 章 `burst_size`）。
const BURST_SIZE: u32 = 64;
/// 批间隔（微秒，11 章 `batch_gap_us`）。
const BATCH_GAP_US: u64 = 2000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("用法: {} <addr:port> <session_id> <chunk_size> <file>", args[0]);
        return Err("参数不足".into());
    }
    let addr = &args[1];
    let session_id: u32 = args[2].parse()?;
    let chunk_size: usize = args[3].parse()?;
    let path = &args[4];

    let data = std::fs::read(path)?;
    let chunk_count = data.len().div_ceil(chunk_size);
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let target: std::net::SocketAddr = addr.parse()?;

    println!(
        "发送 {path}（{} 字节，{chunk_count} 块）→ {addr}",
        data.len()
    );

    let mut sent = 0u64;
    let mut chunk_id = 0usize;
    while chunk_id < chunk_count {
        let end = (chunk_id + BURST_SIZE as usize).min(chunk_count);
        for id in chunk_id..end {
            let offset = id * chunk_size;
            let payload = &data[offset..(offset + chunk_size).min(data.len())];
            let mut f = flags::DATA;
            if id + 1 == chunk_count {
                f |= flags::LAST_CHUNK;
            }
            let pkt = Packet::new(session_id, id as u32, f, payload.to_vec());
            socket.send_to(&pkt.encode(), target)?;
            sent += 1;
        }
        chunk_id = end;
        // 批间隔（11 章「发送节奏」）
        std::thread::sleep(Duration::from_micros(BATCH_GAP_US));
    }
    println!("已发送 {sent} 个数据包（{chunk_count} 块）");
    Ok(())
}
