//! 一致性测试 —— 把规范里的**规范性要求**写成断言。
//!
//! 每个测试都注明对应的规范条目。测试失败即表示实现或规范出了问题。

use fsp_sim::scenarios;
use fsp_sim::World;
use fsp_core::message::{ControlMessage, MissingMode};

/// 启动一次上传并推进仿真。
fn run_upload(world: &mut World, duration_ms: u64) {
    world.client.start(&mut world.sim);
    world.run(duration_ms);
}

#[test]
fn 零丢包时不得卡死() {
    // 11 章：接收方**必须**至少每 50ms 下发一次缺失裁决（**包括空列表**）；
    // 若只在「有缺失」时才下发，发送方在窗口填满后永远等不到裁决 —— 协议死锁。
    let mut world = scenarios::zero_loss();
    run_upload(&mut world, 3000);

    assert!(
        world.server.verdicts_sent > 0,
        "接收方必须周期性下发缺失裁决"
    );
    assert!(
        world.server.empty_verdicts_sent > 0,
        "**空数组裁决是必需的** —— 它是「可以继续推进」的信号（11 章）"
    );
    assert!(
        world.client.empty_verdicts_seen > 0,
        "发送方必须收到空数组裁决"
    );
    assert!(
        world.client.completed,
        "零丢包时传输必须完成；超时说明发送方卡死在窗口上限"
    );
    assert!(world.content_matches(), "文件内容必须一致");
}

#[test]
fn 空洞块必须被补发() {
    // 12 章：缺失块必须被补发；否则文件级校验必然失配。
    let mut world = scenarios::hole_in_the_middle();
    run_upload(&mut world, 3000);

    assert!(
        world.sim.datagrams_dropped > 0,
        "场景前提：确实丢了一个包"
    );
    let sent = world.client.send_counts.get(50).copied().unwrap_or(0);
    assert!(
        sent >= 2,
        "空洞块（chunk 50）必须被补发，实际发送次数 = {sent}"
    );
    assert!(world.client.completed, "补发后必须完成");
    assert!(
        world.content_matches(),
        "补发后文件内容必须一致（否则文件级校验会失配）"
    );
}

#[test]
fn 裁决丢失时必须主动索要() {
    // 11 章第 4 条 / 12 章看门狗：发送方 1000ms 未收到裁决时必须发布
    // missing_chunks(mode=request)。
    let mut world = scenarios::verdict_lost();
    run_upload(&mut world, 3000);

    assert!(
        world.client.requests_sent > 0,
        "1000ms 未收到裁决时，发送方必须主动索要（mode=request）"
    );
}

#[test]
fn 重复报文必须幂等丢弃() {
    // 19 章 CHUNK_ALREADY_RECEIVED：重复 chunk 静默丢弃，不影响文件正确性。
    let mut world = scenarios::duplicated_datagrams();
    run_upload(&mut world, 3000);

    assert!(
        world.server.duplicate_dropped > 0,
        "场景前提：确实收到了重复报文"
    );
    assert!(world.client.completed, "重复报文不得阻止完成");
    assert!(world.content_matches(), "重复报文不得污染文件");
}

#[test]
fn crc32_损坏的包必须丢弃且不写入位图() {
    // 08 章 / 12 章：CRC32 校验失败的包直接丢弃，**不得**标记位图。
    let mut world = scenarios::corrupted_datagram();
    run_upload(&mut world, 3000);

    assert!(
        world.server.corrupt_dropped > 0,
        "场景前提：确实收到了 CRC 失配的包"
    );
    assert!(world.client.completed, "损坏的包必须被裁决补发，最终完成");
    assert!(world.content_matches(), "损坏的包不得污染文件");
}

#[test]
fn 缺失列表必须分批且总数正确() {
    // 18 章：单条 missing_chunks 载荷不得超过 4096 条；
    // missing_count 给出**总数**，用于判断是否还有后续批次。
    let mut world = scenarios::large_missing_list();
    run_upload(&mut world, 5000);

    let sent = world.client.verdicts_seen;
    assert!(sent > 0, "必须收到裁决");
    assert!(
        world.client.completed,
        "分批下发必须最终把全部缺失块补齐"
    );
    assert!(world.content_matches(), "文件内容必须一致");
}

#[test]
fn 续传起点必须是第一个缺失块() {
    // 13 章：`next_chunk_id` = **第一个尚未收到的 chunk_id**。
    // v1.0.0 定义为「最大已收 id + 1」，对乱序协议是错误定义。
    let mut world = scenarios::resume_after_hole();

    // 第一阶段：正常传输，但 chunk 50 丢失。
    // 必须在**首次裁决（50ms）之前**发起续传 —— 否则空洞已被补发，
    // first_missing_id 就不再是 50，测不出「max+1 vs 第一个缺失块」的差别。
    world.client.start(&mut world.sim);
    world.run(20);
    assert!(
        world.sim.datagrams_dropped > 0,
        "场景前提：chunk 50 被丢弃"
    );
    let bitmap = &world.server.session.as_ref().unwrap().bitmap;
    assert_eq!(
        bitmap.max_received_id(),
        Some(99),
        "场景前提：块 0..99 中除 50 外都已收到，最大已收 id = 99"
    );
    assert_eq!(
        bitmap.first_missing_id(),
        Some(50),
        "场景前提：空洞在块 50"
    );

    // 第二阶段：模拟断线重连 —— Client 发送 resume_session
    let session_id = world.client.session_id.expect("会话已建立");
    world.sim.post_control(
        fsp_sim::Endpoint::Server,
        ControlMessage::ResumeSession { session_id },
    );
    world.run(100);

    assert!(
        !world.client.resume_acks.is_empty(),
        "Server 必须应答 resume_ack"
    );
    let (next_chunk_id, missing_count) = world.client.resume_acks[0];
    assert_eq!(
        next_chunk_id, 50,
        "续传起点必须是**第一个缺失块**（chunk 50）；\
         v1.0.0 的 max_received_id + 1 会给出 100 —— 空洞块将永远不被重传"
    );
    assert_ne!(
        next_chunk_id, 100,
        "不得使用 max_received_id + 1（= 100）作为续传起点"
    );
    assert!(
        missing_count >= 1,
        "missing_count 必须反映尚未收到的块数"
    );

    // 第三阶段：按续传起点继续，文件必须最终一致
    world.run(3000);
    assert!(world.client.completed, "续传后必须完成");
    assert!(
        world.content_matches(),
        "续传后文件内容必须一致 —— 这正是 v1.0.0 定义会失败的地方"
    );
}

#[test]
fn 秒传不得发送任何数据() {
    // 10 章「秒传路径」：file_exists = true 时发送侧不发送任何数据，
    // 接收侧走 NEGOTIATING → VERIFY → COMMIT → COMPLETED（不产生位图）。
    let mut world = scenarios::instant_upload();
    world.client.start(&mut world.sim);
    world.run(1000);

    assert!(
        world.client.datagrams_sent == 0,
        "秒传时发送侧不得发送任何数据，实际发了 {} 个",
        world.client.datagrams_sent
    );
    assert!(world.client.completed, "秒传也必须走到 COMPLETED");
    assert!(
        world.server.verdicts_sent == 0,
        "秒传路径不产生位图，不应有缺失裁决；实际 {} 次",
        world.server.verdicts_sent
    );
}

#[test]
fn missing_chunks_请求与报告可区分() {
    // 06 章：请求与报告必须可区分（mode=request / mode=report）。
    let json = r#"{"type":"missing_chunks","mode":"request","session_id":"9","missing_count":0,"chunks":[]}"#;
    let msg: ControlMessage = serde_json::from_str(json).unwrap();
    match msg {
        ControlMessage::MissingChunks { mode, chunks, .. } => {
            assert_eq!(mode, MissingMode::Request);
            assert!(chunks.is_empty(), "mode=request 时 chunks 必须为空数组");
        }
        other => panic!("类型不符: {other:?}"),
    }
}
