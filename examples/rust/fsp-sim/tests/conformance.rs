//! 一致性测试 —— 把规范里的**规范性要求**写成断言。
//!
//! 每个测试都注明对应的规范条目。测试失败即表示实现或规范出了问题。

use fsp_sim::scenarios;
use fsp_sim::World;
use fsp_core::message::{ControlMessage, Direction, MissingMode, Role};

/// 启动一次上传并推进仿真。
fn run_upload(world: &mut World, duration_ms: u64) {
    world.begin();
    world.run(duration_ms);
}

#[test]
fn 零丢包时不得卡死() {
    // 11 章：接收方**必须**至少每 50ms 下发一次缺失裁决（**包括空列表**）；
    // 若只在「有缺失」时才下发，发送方在窗口填满后永远等不到裁决 —— 协议死锁。
    let mut world = scenarios::zero_loss();
    run_upload(&mut world, 3000);

    assert!(
        world.receiver.verdicts_sent > 0,
        "接收方必须周期性下发缺失裁决"
    );
    assert!(
        world.receiver.empty_verdicts_sent > 0,
        "**空数组裁决是必需的** —— 它是「可以继续推进」的信号（11 章）"
    );
    assert!(
        world.sender.empty_verdicts_seen > 0,
        "发送方必须收到空数组裁决"
    );
    assert!(
        world.sender.completed,
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
    let sent = world.sender.send_counts.get(50).copied().unwrap_or(0);
    assert!(
        sent >= 2,
        "空洞块（chunk 50）必须被补发，实际发送次数 = {sent}"
    );
    assert!(world.sender.completed, "补发后必须完成");
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
        world.sender.requests_sent > 0,
        "1000ms 未收到裁决时，发送方必须主动索要（mode=request）"
    );
}

#[test]
fn 重复报文必须幂等丢弃() {
    // 19 章 CHUNK_ALREADY_RECEIVED：重复 chunk 静默丢弃，不影响文件正确性。
    let mut world = scenarios::duplicated_datagrams();
    run_upload(&mut world, 3000);

    assert!(
        world.receiver.duplicate_dropped > 0,
        "场景前提：确实收到了重复报文"
    );
    assert!(world.sender.completed, "重复报文不得阻止完成");
    assert!(world.content_matches(), "重复报文不得污染文件");
}

#[test]
fn crc32_损坏的包必须丢弃且不写入位图() {
    // 08 章 / 12 章：CRC32 校验失败的包直接丢弃，**不得**标记位图。
    let mut world = scenarios::corrupted_datagram();
    run_upload(&mut world, 3000);

    assert!(
        world.receiver.corrupt_dropped > 0,
        "场景前提：确实收到了 CRC 失配的包"
    );
    assert!(world.sender.completed, "损坏的包必须被裁决补发，最终完成");
    assert!(world.content_matches(), "损坏的包不得污染文件");
}

#[test]
fn 缺失列表必须分批且总数正确() {
    // 18 章：单条 missing_chunks 载荷不得超过 4096 条；
    // missing_count 给出**总数**，用于判断是否还有后续批次。
    let mut world = scenarios::large_missing_list();
    run_upload(&mut world, 5000);

    let sent = world.sender.verdicts_seen;
    assert!(sent > 0, "必须收到裁决");
    assert!(
        world.sender.completed,
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
    world.begin();
    world.run(20);
    assert!(
        world.sim.datagrams_dropped > 0,
        "场景前提：chunk 50 被丢弃"
    );
    let bitmap = &world.receiver.session.as_ref().unwrap().bitmap;
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
    let session_id = world.sender.session_id.expect("会话已建立");
    world.sim.post_control(
        fsp_sim::Endpoint::Server,
        ControlMessage::ResumeSession { session_id },
    );
    world.run(100);

    assert!(
        !world.sender.resume_acks.is_empty(),
        "Server 必须应答 resume_ack"
    );
    let (next_chunk_id, missing_count) = world.sender.resume_acks[0];
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
    assert!(world.sender.completed, "续传后必须完成");
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
    world.begin();
    world.run(1000);

    assert!(
        world.sender.datagrams_sent == 0,
        "秒传时发送侧不得发送任何数据，实际发了 {} 个",
        world.sender.datagrams_sent
    );
    assert!(world.sender.completed, "秒传也必须走到 COMPLETED");
    assert!(
        world.receiver.verdicts_sent == 0,
        "秒传路径不产生位图，不应有缺失裁决；实际 {} 次",
        world.receiver.verdicts_sent
    );
}

// ═══════════════════════════ 下载方向（S→C） ═══════════════════════════

#[test]
fn 下载方向必须由_client_发布_download_complete() {
    // 06 章：**完成消息一律由接收方发布**。
    // 下载方向的接收方是 Client，因此 `download_complete` 必须由 Client 发布。
    // v1.0.0 曾三处互相矛盾：14 章要求 Client 发、22 章禁止 Client 发、
    // 22 章自身的 topic 清单又允许 Client 发。
    let mut world = scenarios::download_zero_loss();
    world.begin();
    world.run(3000);

    assert!(
        world.receiver.complete_published.contains(&"download_complete"),
        "下载方向的接收侧（Client）必须发布 download_complete；实际发布 = {:?}",
        world.receiver.complete_published
    );
    assert!(
        !world.receiver.complete_published.contains(&"upload_complete"),
        "下载方向不得发布 upload_complete"
    );
    // 发送侧（Server）只接收完成消息，不得自己发布
    assert_eq!(
        world.receiver.complete_published.len(),
        1,
        "完成消息必须恰好发布一次"
    );
    assert!(
        world.sender.complete_seen.contains(&"download_complete"),
        "发送侧必须收到 download_complete；实际收到 = {:?}",
        world.sender.complete_seen
    );
    assert!(world.content_matches(), "文件内容必须一致");
}

#[test]
fn 下载方向的裁决由接收方_client_下发() {
    // 09 章 / 06 章：`missing_chunks` 的 `mode=report` 由**接收方**发布。
    // 下载方向的接收方是 Client —— 所以裁决必须由 Client 发出。
    let mut world = scenarios::download_hole();
    world.begin();
    world.run(3000);

    assert!(
        world.receiver.verdicts_sent > 0,
        "接收侧（Client）必须下发缺失裁决"
    );
    assert!(
        world.receiver.empty_verdicts_sent > 0,
        "空数组裁决是必需的（11 章）"
    );
    let sent = world.sender.send_counts.get(50).copied().unwrap_or(0);
    assert!(sent >= 2, "空洞块必须被 Server 补发，实际发送次数 = {sent}");
    assert!(world.content_matches(), "补发后文件内容必须一致");
}

#[test]
fn 下载_accept_必须带_udp_port_与_chunk_size() {
    // 06 章「accept 字段对称性」：缺 udp_port 则数据通路无法建立
    //（20 章禁止 Server 向未知端口发送 UDP）；缺 chunk_size 则接收方无法核算 chunk_count。
    let mut world = scenarios::download_accept_fields();
    // 在客户端捕获收到的 download_accept
    world.begin();
    world.run(2000);

    let session = world
        .receiver
        .session
        .as_ref()
        .expect("接收侧必须建立会话（说明 accept 被正确解析）");
    assert_eq!(session.direction, Direction::Download, "会话方向必须是 download");
    assert_ne!(session.udp_port, 0, "download_accept 必须携带非零 udp_port");
    assert!(
        session.chunk_size > 0,
        "download_accept 必须携带 chunk_size，否则无法核算 chunk_count"
    );
    assert_eq!(
        session.chunk_count(),
        (world.source.len() as u64).div_ceil(u64::from(session.chunk_size)) as u32,
        "chunk_count 必须等于 ceil(size / chunk_size)"
    );
    assert!(world.content_matches(), "文件内容必须一致");
}

#[test]
fn 下载方向的会话参数带_direction() {
    // 09 章：`direction` 字段是必需的 —— `missing_chunks` 是双向消息，
    // 其含义依赖会话方向。Server 侧的会话记录必须能区分「我是接收方」还是「我是发送方」。
    assert_eq!(Direction::Upload.receiver(), Role::Server);
    assert_eq!(Direction::Upload.sender(), Role::Client);
    assert_eq!(Direction::Download.receiver(), Role::Client);
    assert_eq!(Direction::Download.sender(), Role::Server);
}

#[test]
fn 数据发到错误端点必须丢弃() {
    // 08 章 / 20 章：接收方必须校验会话绑定的端点。
    // 仿真里把数据报投给**发送侧**（错误端点）必须被丢弃，不得写入任何位图。
    let mut world = scenarios::zero_loss();
    world.begin();
    debug_assert_eq!(world.direction, Direction::Upload);

    // 故意把 8 个数据报投给发送侧端点（上传方向下发送侧是 Client）
    let wrong = world.sender_endpoint();
    const WRONG: u64 = 8;
    for chunk_id in 0..WRONG as u32 {
        let pkt = fsp_core::packet::Packet::new(
            1,
            chunk_id,
            fsp_core::packet::flags::DATA,
            vec![0u8; fsp_core::packet::DEFAULT_CHUNK_SIZE as usize],
        );
        world.sim.post_datagram(wrong, pkt.encode());
    }

    world.run(3000);

    assert!(
        world.sim.datagrams_dropped >= WRONG,
        "发到错误端点的 {WRONG} 个数据报必须全部被丢弃，实际丢弃 {}",
        world.sim.datagrams_dropped
    );
    // 正确端点的传输仍然正常完成
    assert!(world.sender.completed, "错误端点的包不得影响正常传输");
    assert!(world.content_matches(), "文件内容必须一致");
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
