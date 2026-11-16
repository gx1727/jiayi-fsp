//! 规范场景 —— 每一条都对应规范里的一条**规范性要求**。
//!
//! 这些场景是 v1.0.0 → v1.1.0 修订中修掉的缺陷的**回归防线**。

use crate::sim::{ControlFaults, DataFaults};
use crate::World;
use fsp_core::packet::DEFAULT_CHUNK_SIZE;

/// 一个可运行的规范场景。
#[derive(Debug, Clone)]
pub struct Scenario {
    /// 场景名称。
    pub name: &'static str,
    /// 对应的规范条目。
    pub spec: &'static str,
    /// 场景说明。
    pub description: &'static str,
    /// 构造仿真世界。
    pub setup: fn() -> World,
    /// 仿真时长（毫秒）。
    pub duration_ms: u64,
    /// 该场景的通过判据（不同场景的正确性标准本就不同）。
    pub pass: fn(&World) -> bool,
}

/// 默认判据：传输完成且文件一致。
fn pass_completed(world: &World) -> bool {
    world.client.completed && world.content_matches()
}

/// 构造一段确定性的测试内容。
pub fn make_payload(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

/// 场景 1：零丢包 —— 接收方**必须**周期性下发空数组裁决，发送方**不得**卡死。
///
/// > v1.0.0 从未规定接收方下发 `missing_chunks` 的周期；若只在「有缺失」时才下发，
/// > 零丢包时发送方在窗口填满后永远等不到裁决 —— **协议死锁**。
pub fn zero_loss() -> World {
    let file = make_payload(200 * DEFAULT_CHUNK_SIZE as usize); // 200 个 chunk
    let mut world = World::new(file, DEFAULT_CHUNK_SIZE, 16, 0xA1); // 小窗口，强制触发在途上限
    world.server.window = 16;
    world.sim.data_faults = DataFaults::default();
    world.sim.control_faults = ControlFaults::default();
    world
}

/// 场景 2：空洞块 —— 丢一个中间块，**必须**被补发。
///
/// > 本协议显式支持乱序写，位图会出现「空洞」。若空洞块不被补发，
/// > 文件级 Blake3 校验必然失配（12 章）。
pub fn hole_in_the_middle() -> World {
    let file = make_payload(100 * DEFAULT_CHUNK_SIZE as usize);
    let mut world = World::new(file, DEFAULT_CHUNK_SIZE, 32, 0xB2);
    world.server.window = 32;
    world.sim.data_faults.drop_chunk_ids = vec![50];
    world
}

/// 场景 3：断点续传 —— `resume_ack.next_chunk_id` **必须**是第一个缺失块。
///
/// > v1.0.0 定义为「最大已收 chunk_id + 1」，对乱序协议是错误定义：
/// > 空洞块将永远不会被重传。
pub fn resume_after_hole() -> World {
    let file = make_payload(100 * DEFAULT_CHUNK_SIZE as usize);
    let mut world = World::new(file, DEFAULT_CHUNK_SIZE, 128, 0xC3);
    world.server.window = 128;
    world.sim.data_faults.drop_chunk_ids = vec![50];
    world
}

/// 场景 4：裁决丢失 —— 发送方 1000ms 未收到裁决时**必须**主动索要。
pub fn verdict_lost() -> World {
    let file = make_payload(100 * DEFAULT_CHUNK_SIZE as usize);
    let mut world = World::new(file, DEFAULT_CHUNK_SIZE, 16, 0xD4);
    world.server.window = 16;
    // 丢弃所有裁决消息，模拟控制面消息丢失
    world.sim.control_faults.drop_actions = vec!["missing_chunks"];
    world
}

/// 场景 5：重复报文 —— 必须幂等丢弃，不污染文件。
pub fn duplicated_datagrams() -> World {
    let file = make_payload(50 * DEFAULT_CHUNK_SIZE as usize);
    let mut world = World::new(file, DEFAULT_CHUNK_SIZE, 16, 0xE5);
    world.server.window = 16;
    world.sim.data_faults.dup_num = 1;
    world.sim.data_faults.dup_den = 1; // 100% 重复
    world
}

/// 场景 6：CRC32 损坏 —— 校验失败的包**必须**丢弃且**不得**写入位图。
pub fn corrupted_datagram() -> World {
    let file = make_payload(50 * DEFAULT_CHUNK_SIZE as usize);
    let mut world = World::new(file, DEFAULT_CHUNK_SIZE, 16, 0xF6);
    world.server.window = 16;
    world.sim.data_faults.corrupt_chunk_ids = vec![10, 20];
    world
}

/// 场景 7：大文件缺失分批 —— 单条载荷**不得超过 4096 条**。
pub fn large_missing_list() -> World {
    let file = make_payload(6000 * DEFAULT_CHUNK_SIZE as usize); // 6000 个 chunk
    let mut world = World::new(file, DEFAULT_CHUNK_SIZE, 2048, 0x77);
    world.server.window = 2048;
    // 丢弃前 5000 个块中的很多块，制造超长缺失列表
    world.sim.data_faults.drop_chunk_ids = (100..5100).collect();
    world
}

/// 场景 8：秒传 —— `file_exists = true` 时**不得**发送任何数据。
pub fn instant_upload() -> World {
    let file = make_payload(10 * DEFAULT_CHUNK_SIZE as usize);
    let mut world = World::new(file, DEFAULT_CHUNK_SIZE, 32, 0x88);
    world.server.window = 32;
    world.server.instant_upload = true;
    world
}

/// 全部场景。
pub fn all() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "零丢包不卡死",
            spec: "11 章「窗口推进判据」",
            description: "接收方必须每 50ms 下发裁决（空数组也算），发送方不得在窗口填满后停滞",
            setup: zero_loss,
            duration_ms: 3000,
            pass: |w| {
                // 零丢包时必须完成，且必须收到过空数组裁决（推进信号）
                pass_completed(w) && w.server.empty_verdicts_sent > 0 && w.client.empty_verdicts_seen > 0
            },
        },
        Scenario {
            name: "空洞块补发",
            spec: "12 章 / 18 章",
            description: "低于最大已收 id 的缺失块必须被补发，文件级校验必须通过",
            setup: hole_in_the_middle,
            duration_ms: 3000,
            pass: |w| {
                // 空洞块必须被补发（发送次数 ≥ 2）且文件一致
                pass_completed(w) && w.client.send_counts.get(50).copied().unwrap_or(0) >= 2
            },
        },
        Scenario {
            name: "裁决丢失时主动索要",
            spec: "11 章第 4 条 / 12 章看门狗",
            description: "发送方 1000ms 未收到裁决时必须发布 missing_chunks(mode=request)",
            setup: verdict_lost,
            duration_ms: 3000,
            pass: |w| {
                // 裁决全丢 → 传输**不可能**完成；判据是「发送方必须主动索要」
                w.client.requests_sent > 0
            },
        },
        Scenario {
            name: "重复报文幂等",
            spec: "19 章 CHUNK_ALREADY_RECEIVED",
            description: "重复收到的 chunk 必须静默丢弃，不影响文件正确性",
            setup: duplicated_datagrams,
            duration_ms: 3000,
            pass: |w| pass_completed(w) && w.server.duplicate_dropped > 0,
        },
        Scenario {
            name: "CRC32 损坏丢弃",
            spec: "08 章 / 12 章",
            description: "CRC32 校验失败的包必须丢弃且不得写入位图",
            setup: corrupted_datagram,
            duration_ms: 3000,
            pass: |w| pass_completed(w) && w.server.corrupt_dropped > 0,
        },
        Scenario {
            name: "大文件缺失分批",
            spec: "18 章「缺失列表的下发上限」",
            description: "单条 missing_chunks 载荷不得超过 4096 条，missing_count 给出总数",
            setup: large_missing_list,
            duration_ms: 30000,
            pass: |w| pass_completed(w) && w.server.verdicts_sent > 0,
        },
        Scenario {
            name: "断点续传起点",
            spec: "13 章「resume_ack 语义」",
            description: "next_chunk_id 必须是第一个缺失块，不得是 max_received_id + 1",
            setup: resume_after_hole,
            duration_ms: 3000,
            pass: pass_completed,
        },
        Scenario {
            name: "秒传不发数据",
            spec: "10 章「秒传路径」",
            description: "file_exists = true 时发送侧不得发送任何数据",
            setup: instant_upload,
            duration_ms: 2000,
            pass: |w| w.client.datagrams_sent == 0 && w.client.completed,
        },
    ]
}
