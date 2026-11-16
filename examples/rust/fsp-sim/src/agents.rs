//! 协议代理：发送侧（Client）与接收侧（Server）。
//!
//! 这里落地的是规范 `06` / `10` / `11` / `12` / `13` 章的**行为**。
//! 每个断言场景都针对其中一条具体的规范性要求。

use crate::sim::{Endpoint, Event, Sim};
use fsp_core::flow::{FlowController, VERDICT_TIMEOUT_MS};
use fsp_core::message::{
    ControlMessage, Direction, MissingMode, ResultKind, SessionId, StatusValue,
};
use fsp_core::packet::{flags, DecodeCtx, Packet};
use fsp_core::session::{Session, SessionState, SessionView};
use fsp_core::verdict::{build_verdict, VerdictScheduler};

/// 简易内容摘要（FNV-1a 64）。
///
/// **注意**：生产实现使用 **Blake3**（06 章 / 12 章）。仿真用摘要只是为了
/// 表达「接收方在 VERIFY 阶段校验文件内容」这一**协议行为**，
/// 不试图复刻 Blake3。
pub fn digest(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// 发送侧代理（上传方向 = Client）。
#[derive(Debug)]
pub struct SenderAgent {
    /// 客户端设备 ID。
    pub client_id: String,
    /// 会话 ID（收到 accept 后确定）。
    pub session_id: Option<SessionId>,
    /// 发送侧状态。
    pub state: SessionState,
    /// 流控控制器。
    pub flow: FlowController,
    /// 协商的 chunk 大小。
    pub chunk_size: u32,
    /// 总 chunk 数。
    pub chunk_count: u32,
    /// 源文件内容。
    pub file: Vec<u8>,
    /// 顺序发送游标。
    pub next_to_send: u32,
    /// 待重传的 chunk（优先于新数据）。
    pub retransmit: Vec<u32>,
    /// 上次收到裁决的时刻（微秒）。
    pub last_verdict_at_us: u64,
    /// 是否已发送完全部数据。
    pub all_sent: bool,
    /// 会话是否已完成。
    pub completed: bool,
    /// 收到的裁决次数。
    pub verdicts_seen: u64,
    /// 其中「空数组」裁决的次数 —— 这是「可以继续推进」的信号。
    pub empty_verdicts_seen: u64,
    /// 发出的 `mode=request` 次数。
    pub requests_sent: u64,
    /// 因在途量达上限而暂停的次数。
    pub pause_count: u64,
    /// 已发出的数据报计数。
    pub datagrams_sent: u64,
    /// 每个 chunk 的发送次数（用于断言「空洞块确实被补发过」）。
    pub send_counts: Vec<u32>,
    /// 收到的 `resume_ack`（用于断言续传起点）。
    pub resume_acks: Vec<(u32, u32)>,
}

impl SenderAgent {
    /// 新建发送侧代理。
    pub fn new(client_id: impl Into<String>, file: Vec<u8>, chunk_size: u32, window: u32) -> Self {
        let chunk_count = if chunk_size == 0 {
            0
        } else {
            (file.len() as u64).div_ceil(u64::from(chunk_size)) as u32
        };
        Self {
            client_id: client_id.into(),
            session_id: None,
            state: SessionState::Init,
            flow: FlowController::new(window),
            chunk_size,
            chunk_count,
            file,
            next_to_send: 0,
            retransmit: Vec::new(),
            last_verdict_at_us: 0,
            all_sent: false,
            completed: false,
            verdicts_seen: 0,
            empty_verdicts_seen: 0,
            requests_sent: 0,
            pause_count: 0,
            datagrams_sent: 0,
            send_counts: Vec::new(),
            resume_acks: Vec::new(),
        }
    }

    /// 发起上传（`upload_start`）。
    pub fn start(&mut self, sim: &mut Sim) {
        let file_hash = format!("{:016x}", digest(&self.file));
        let msg = ControlMessage::UploadStart {
            session_id: None,
            file_hash,
            file_name: "sample.bin".into(),
            size: self.file.len() as u64,
            mtime: 0,
            chunk_size: self.chunk_size,
        };
        self.state = SessionState::Negotiating;
        sim.post_control(Endpoint::Server, msg);
    }

    /// 处理收到的控制消息。
    pub fn on_control(&mut self, msg: &ControlMessage, sim: &mut Sim) {
        match msg {
            ControlMessage::UploadAccept {
                session_id,
                window,
                chunk_size,
                file_exists,
                ..
            } => {
                self.session_id = Some(*session_id);
                self.chunk_size = *chunk_size;
                self.chunk_count = self.chunk_count_for(*chunk_size);
                self.flow = FlowController::new(*window);
                if *file_exists {
                    // 秒传：不发送任何数据，直接等待完成通知（10 章）
                    self.state = SessionState::Completed;
                    self.all_sent = true;
                    return;
                }
                self.state = SessionState::Transferring;
                sim.schedule_now(Event::ClientSendBatch);
            }
            ControlMessage::MissingChunks {
                mode,
                missing_count,
                chunks,
                ..
            } => match mode {
                MissingMode::Report => self.on_verdict(missing_count, chunks, sim),
                MissingMode::Request => {
                    // 发送方没有缺失块，回一个空报告
                    let reply = ControlMessage::MissingChunks {
                        mode: MissingMode::Report,
                        session_id: self.session_id.expect("会话已建立"),
                        missing_count: Some(0),
                        chunks: Vec::new(),
                    };
                    sim.post_control(Endpoint::Server, reply);
                }
            },
            ControlMessage::ResumeAck { next_chunk_id, missing_count, .. } => {
                // 续传：顺序发送流从 next_chunk_id 继续（13 章）
                self.resume_acks.push((*next_chunk_id, *missing_count));
                self.next_to_send = *next_chunk_id;
                self.retransmit.clear();
                self.state = SessionState::Transferring;
                sim.schedule_now(Event::ClientSendBatch);
            }
            ControlMessage::UploadComplete { .. } | ControlMessage::DownloadComplete { .. } => {
                self.completed = true;
                self.state = SessionState::Completed;
            }
            ControlMessage::SessionAbort { .. } => {
                self.state = SessionState::Aborted;
                self.completed = true;
            }
            _ => {}
        }
    }

    fn chunk_count_for(&self, chunk_size: u32) -> u32 {
        if chunk_size == 0 {
            0
        } else {
            (self.file.len() as u64).div_ceil(u64::from(chunk_size)) as u32
        }
    }

    fn on_verdict(&mut self, missing_count: &Option<u32>, chunks: &[u32], sim: &mut Sim) {
        self.verdicts_seen += 1;
        self.last_verdict_at_us = sim.now_us;
        if chunks.is_empty() {
            // 空数组裁决 = 「我这边没有缺失，可继续推进」
            self.empty_verdicts_seen += 1;
        }
        // 缺失块入重传队列（去重，且只处理已经发出过的块）
        for &id in chunks {
            if id < self.next_to_send && !self.retransmit.contains(&id) {
                self.retransmit.push(id);
            }
        }
        // 累加 AIMD 采样：已确认 = 总数 − 尚未补齐
        let pending = missing_count.unwrap_or(0);
        let acked = u64::from(self.chunk_count.saturating_sub(pending));
        let sent_in_interval = self.flow.sent_total.saturating_sub(self.flow.acked_total);
        let received_in_interval = acked.saturating_sub(self.flow.acked_total);
        self.flow.on_verdict(acked, sent_in_interval, received_in_interval);
        // 收到裁决后立即安排一次发送（有缺失要补发）
        sim.schedule_now(Event::ClientSendBatch);
    }

    /// 发送一批数据。
    pub fn on_tick(&mut self, sim: &mut Sim) {
        if std::env::var("FSP_TRACE").is_ok() {
            eprintln!(
                "  [client.on_tick] t={}ms state={:?} in_flight={}/{} next={} req={}",
                sim.now_ms(), self.state, self.flow.in_flight(), self.flow.window,
                self.next_to_send, self.requests_sent
            );
        }
        if self.state != SessionState::Transferring {
            return;
        }
        // 裁决超时：主动索要
        let elapsed_ms = (sim.now_us - self.last_verdict_at_us) / 1000;
        if elapsed_ms > VERDICT_TIMEOUT_MS {
            let msg = ControlMessage::MissingChunks {
                mode: MissingMode::Request,
                session_id: self.session_id.expect("会话已建立"),
                missing_count: Some(0),
                chunks: Vec::new(),
            };
            self.requests_sent += 1;
            self.last_verdict_at_us = sim.now_us;
            sim.post_control(Endpoint::Server, msg);
        }

        let burst = self.flow.burst_size as usize;

        // 1) 优先补发重传队列
        //
        // **重传不受在途上限约束**（11 章窗口推进判据第 3 条）：
        // 「达到上限时，发送方必须暂停发送**新数据**（重传不受此限）」。
        // 若把重传也挡在上限之外，在途量已满时缺失块永远补不回来 —— 会死锁。
        let take_resend = self.retransmit.len().min(burst);
        for _ in 0..take_resend {
            let id = self.retransmit.remove(0);
            self.send_chunk(id, true, sim);
        }

        // 2) 在途量达上限 → 暂停发送新数据
        if self.flow.should_pause() {
            self.pause_count += 1;
            sim.schedule(100, Event::ClientSendBatch);
            return;
        }

        // 3) 再发新数据
        let mut sent_new = 0u64;
        while sent_new < burst as u64 && self.next_to_send < self.chunk_count {
            let id = self.next_to_send;
            self.next_to_send += 1;
            self.send_chunk(id, false, sim);
            sent_new += 1;
        }

        self.flow.on_sent(sent_new);
        self.all_sent = self.next_to_send >= self.chunk_count && self.retransmit.is_empty();

        // 安排下一批
        let gap_ms = (self.flow.batch_gap_us / 1000).max(1);
        if !self.all_sent {
            sim.schedule(gap_ms, Event::ClientSendBatch);
        } else {
            // 数据已发完，等待完成通知；仍需周期 tick 以便触发裁决超时索要
            sim.schedule(gap_ms.max(100), Event::ClientSendBatch);
        }
    }

    fn send_chunk(&mut self, chunk_id: u32, resend: bool, sim: &mut Sim) {
        let offset = chunk_id as usize * self.chunk_size as usize;
        let end = (offset + self.chunk_size as usize).min(self.file.len());
        let payload = self.file[offset..end].to_vec();
        let mut f = flags::DATA;
        if resend {
            f |= flags::RESEND;
        }
        if chunk_id + 1 == self.chunk_count {
            f |= flags::LAST_CHUNK;
        }
        let pkt = Packet::new(self.session_id.expect("会话已建立").0, chunk_id, f, payload);
        self.datagrams_sent += 1;
        if self.send_counts.len() <= chunk_id as usize {
            self.send_counts.resize(chunk_id as usize + 1, 0);
        }
        self.send_counts[chunk_id as usize] += 1;
        sim.post_datagram(Endpoint::Server, pkt.encode());
    }
}

/// 接收侧代理（上传方向 = Server）。
#[derive(Debug)]
pub struct ReceiverAgent {
    /// 会话。
    pub session: Option<Session>,
    /// 接收缓冲区（按 chunk 偏移写入）。
    pub buffer: Vec<u8>,
    /// 期望内容摘要（用于 VERIFY 阶段）。
    pub expected_digest: u64,
    /// 上次下发裁决的时刻（微秒）。
    pub last_verdict_at_us: u64,
    /// 裁决调度器。
    pub scheduler: VerdictScheduler,
    /// 已下发的裁决次数。
    pub verdicts_sent: u64,
    /// 其中空数组裁决的次数。
    pub empty_verdicts_sent: u64,
    /// 收到的 `mode=request` 次数。
    pub requests_seen: u64,
    /// 因 CRC / 会话不匹配而丢弃的报文数。
    pub corrupt_dropped: u64,
    /// 重复收到的报文数（幂等丢弃）。
    pub duplicate_dropped: u64,
    /// 是否已完成。
    pub completed: bool,
    /// Server 生成会话 ID 的计数器。
    next_session_id: u32,
    /// 下发的滑动窗口（**在途上限**）。
    pub window: u32,
    /// 是否走秒传路径（`file_exists = true`）。
    pub instant_upload: bool,
}

impl ReceiverAgent {
    /// 新建接收侧代理；`expected` 是期望收到的文件内容。
    pub fn new(expected: Vec<u8>) -> Self {
        Self {
            session: None,
            buffer: vec![0u8; expected.len()],
            expected_digest: digest(&expected),
            last_verdict_at_us: 0,
            scheduler: VerdictScheduler::new(0),
            verdicts_sent: 0,
            empty_verdicts_sent: 0,
            requests_seen: 0,
            corrupt_dropped: 0,
            duplicate_dropped: 0,
            completed: false,
            next_session_id: 1,
            window: fsp_core::packet::DEFAULT_WINDOW,
            instant_upload: false,
        }
    }

    /// 处理收到的控制消息。
    pub fn on_control(&mut self, msg: &ControlMessage, sim: &mut Sim) {
        match msg {
            ControlMessage::UploadStart {
                file_hash,
                file_name,
                size,
                chunk_size,
                ..
            } => {
                let session_id = SessionId(self.next_session_id);
                self.next_session_id += 1;
                let mut session = Session::new(
                    session_id,
                    "client1",
                    Direction::Upload,
                    file_hash.clone(),
                    file_name.clone(),
                    *size,
                    0,
                    *chunk_size,
                    self.window,
                    5000,
                    0,
                );
                session.transition(SessionState::Negotiating, SessionView::Receiver).expect("INIT→NEGOTIATING 合法");
                let chunk_count = session.chunk_count();
                let window = session.window;
                let chunk_size = session.chunk_size;
                self.session = Some(session);
                let accept = ControlMessage::UploadAccept {
                    file_hash: file_hash.clone(),
                    session_id,
                    chunk_count,
                    window,
                    udp_port: 5000,
                    tcp_port: 0,
                    chunk_size,
                    file_exists: self.instant_upload,
                };
                sim.post_control(Endpoint::Client, accept);
                if self.instant_upload {
                    // 秒传路径（10 章）：不产生位图，
                    // 完成条件是「既有文件校验通过并原子提交」
                    self.complete_instant(sim);
                    return;
                }
                // 首次裁决延后一个周期 —— 接收方不会在会话刚建立、位图还全空时
                // 就下发「你什么都缺」的裁决（11 章：裁决周期 50ms）
                sim.schedule(fsp_core::flow::VERDICT_PERIOD_MS, Event::ServerVerdict);
            }
            ControlMessage::ResumeSession { session_id } => {
                // 续传起点 = **第一个尚未收到的 chunk_id**（13 章 / 18 章）
                let Some(session) = &self.session else { return };
                let next_chunk_id = session.bitmap.first_missing_id().unwrap_or(session.chunk_count());
                let missing_count = session.chunk_count() - session.bitmap.received_count();
                let ack = ControlMessage::ResumeAck {
                    session_id: *session_id,
                    next_chunk_id,
                    missing_count,
                    udp_port: session.udp_port,
                };
                sim.post_control(Endpoint::Client, ack);
            }
            ControlMessage::MissingChunks { mode: MissingMode::Request, .. } => {
                self.requests_seen += 1;
                // 必须在 50ms 内应答
                self.publish_verdict(sim);
            }
            _ => {}
        }
    }

    /// 处理收到的数据报。
    pub fn on_datagram(&mut self, bytes: &[u8], sim: &mut Sim) {
        let Some(session) = &self.session else {
            return;
        };
        let ctx = DecodeCtx::new(session.session_id.0, session.chunk_size, session.size);
        let pkt = match Packet::decode(bytes, &ctx) {
            Ok(p) => p,
            Err(_) => {
                // CRC32_INVALID / SESSION_ID_MISMATCH / INVALID_CHUNK → 直接丢弃，
                // 不得写入位图（08 章 / 12 章）
                self.corrupt_dropped += 1;
                return;
            }
        };
        let chunk_id = pkt.chunk_id;
        let was_new = session.bitmap.mark_received(chunk_id);
        if !was_new {
            // 幂等：重复 chunk 静默丢弃（19 章 CHUNK_ALREADY_RECEIVED）
            self.duplicate_dropped += 1;
            return;
        }
        let offset = chunk_id as usize * session.chunk_size as usize;
        let end = offset + pkt.payload.len();
        if end <= self.buffer.len() {
            self.buffer[offset..end].copy_from_slice(&pkt.payload);
        }
        // 位图全真 → VERIFY → COMMIT → COMPLETED
        if session.bitmap.is_complete() && !self.completed {
            self.complete(sim);
        }
    }

    /// 秒传路径：校验既有文件 → COMMIT → COMPLETED → 发布完成消息。
    fn complete_instant(&mut self, sim: &mut Sim) {
        let Some(session) = &mut self.session else { return };
        session.transition(SessionState::Verify, SessionView::Receiver).expect("NEGOTIATING→VERIFY 合法");
        // 仿真中「既有文件」就是期望内容
        session.transition(SessionState::Commit, SessionView::Receiver).expect("VERIFY→COMMIT 合法");
        session.transition(SessionState::Completed, SessionView::Receiver).expect("COMMIT→COMPLETED 合法");
        let complete = ControlMessage::UploadComplete {
            session_id: session.session_id,
            result: ResultKind::Success,
            code: None,
            file_hash: Some(session.file_hash.clone()),
            asset_id: Some("asset-1".into()),
        };
        self.completed = true;
        sim.post_control(Endpoint::Client, complete);
    }

    fn complete(&mut self, sim: &mut Sim) {
        let Some(session) = &mut self.session else { return };
        session.transition(SessionState::Verify, SessionView::Receiver).expect("TRANSFERRING→VERIFY 合法");
        let actual = digest(&self.buffer);
        if actual != self.expected_digest {
            // 文件级校验失败 → 必须以 session_abort 终止（12 章）
            let abort = ControlMessage::SessionAbort {
                session_id: session.session_id,
                reason: "FILE_HASH_MISMATCH".into(),
                detail: None,
            };
            session.transition(SessionState::Aborted, SessionView::Receiver).ok();
            sim.post_control(Endpoint::Client, abort);
            return;
        }
        session.transition(SessionState::Commit, SessionView::Receiver).expect("VERIFY→COMMIT 合法");
        session.transition(SessionState::Completed, SessionView::Receiver).expect("COMMIT→COMPLETED 合法");
        let complete = ControlMessage::UploadComplete {
            session_id: session.session_id,
            result: ResultKind::Success,
            code: None,
            file_hash: Some(session.file_hash.clone()),
            asset_id: Some("asset-1".into()),
        };
        self.completed = true;
        sim.post_control(Endpoint::Client, complete);
    }

    /// 周期裁决：下发 `missing_chunks`（`mode=report`）。
    ///
    /// **即使缺失列表为空也必须下发** —— 空数组是「可以继续推进」的裁决。
    pub fn on_verdict_tick(&mut self, sim: &mut Sim) {
        if self.session.is_none() {
            return;
        }
        self.publish_verdict(sim);
        // 安排下一次裁决（50ms 周期）
        sim.schedule(fsp_core::flow::VERDICT_PERIOD_MS, Event::ServerVerdict);
    }

    fn publish_verdict(&mut self, sim: &mut Sim) {
        let Some(session) = &self.session else { return };
        let verdict = build_verdict(&session.bitmap);
        self.verdicts_sent += 1;
        if verdict.is_clear() {
            self.empty_verdicts_sent += 1;
        }
        self.last_verdict_at_us = sim.now_us;
        self.scheduler.on_verdict_sent(sim.now_ms());
        let msg = ControlMessage::MissingChunks {
            mode: MissingMode::Report,
            session_id: session.session_id,
            missing_count: Some(verdict.missing_count),
            chunks: verdict.chunks,
        };
        sim.post_control(Endpoint::Client, msg);
    }
}

/// 便捷构造：Client 侧的状态消息。
pub fn status_online(device_id: &str) -> ControlMessage {
    ControlMessage::Status { device_id: device_id.into(), status: StatusValue::Online }
}
