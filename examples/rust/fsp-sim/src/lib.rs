//! # fsp-sim
//!
//! JIAYI-FSP 的**确定性仿真**：虚拟时钟 + 虚拟 Broker（控制面）+ 可注入故障的
//! 虚拟 UDP（数据面）。
//!
//! 用途：把规范里的**规范性要求**写成可执行的断言。
//! 典型例子：
//!
//! - 零丢包时接收方**必须**周期性下发空数组裁决，发送方**不得**卡死（11 章）；
//! - 空洞块（低于最大已收 id 的缺失块）**必须**被补发（13 章 / 18 章）；
//! - `resume_ack.next_chunk_id` **必须**是第一个缺失块，**不得**是 `max+1`（13 章）；
//! - CRC32 校验失败的包**必须**丢弃且**不得**写入位图（08 章 / 12 章）；
//! - 重复 chunk **必须**幂等丢弃（19 章）；
//! - **完成消息一律由接收方发布**（06 章）：上传方向是 `upload_complete`（Server），
//!   下载方向是 `download_complete`（Client）。
//!
//! 仿真完全离线，不需要 mosquitto，也不需要真实网卡。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod agents;
pub mod rng;
pub mod scenarios;
pub mod sim;

pub use agents::{ReceiverAgent, SenderAgent, digest};
pub use scenarios::Scenario;
pub use sim::{ControlFaults, DataFaults, Endpoint, Event, Sim};

use fsp_core::message::Direction;
use std::collections::BTreeMap;

/// 仿真世界：虚拟引擎 + 发送侧 + 接收侧 + 会话方向。
///
/// **方向决定角色**（09 章 `direction`）：
///
/// | direction | 发送侧（sender） | 接收侧（receiver） |
/// |-----------|------------------|--------------------|
/// | `upload`  | Client           | Server             |
/// | `download`| Server           | Client             |
///
/// 无论哪个方向，**发起会话的都是 Client**（`upload_start` / `download_start`），
/// **发布完成消息的都是接收侧**。
#[derive(Debug)]
pub struct World {
    /// 仿真引擎。
    pub sim: Sim,
    /// 会话方向。
    pub direction: Direction,
    /// 发送侧代理。
    pub sender: SenderAgent,
    /// 接收侧代理。
    pub receiver: ReceiverAgent,
    /// 源文件内容（用于校验接收结果）。
    pub source: Vec<u8>,
}

impl Default for SenderAgent {
    fn default() -> Self {
        SenderAgent::new(
            "client1",
            Vec::new(),
            fsp_core::packet::DEFAULT_CHUNK_SIZE,
            fsp_core::packet::DEFAULT_WINDOW,
            Direction::Upload,
        )
    }
}

impl Default for ReceiverAgent {
    fn default() -> Self {
        ReceiverAgent::new(Vec::new(), Direction::Upload)
    }
}

impl World {
    /// 新建**上传方向**的仿真世界（C→S）：`file` 是要传输的内容。
    pub fn new(file: Vec<u8>, chunk_size: u32, window: u32, seed: u64) -> Self {
        Self::with_direction(Direction::Upload, file, chunk_size, window, seed)
    }

    /// 新建**下载方向**的仿真世界（S→C）。
    pub fn new_download(file: Vec<u8>, chunk_size: u32, window: u32, seed: u64) -> Self {
        Self::with_direction(Direction::Download, file, chunk_size, window, seed)
    }

    fn with_direction(
        direction: Direction,
        file: Vec<u8>,
        chunk_size: u32,
        window: u32,
        seed: u64,
    ) -> Self {
        World {
            sim: Sim::new(seed),
            direction,
            sender: SenderAgent::new("client1", file.clone(), chunk_size, window, direction),
            receiver: ReceiverAgent::new(file.clone(), direction),
            source: file,
        }
    }

    /// 发送侧所在的端点。
    pub fn sender_endpoint(&self) -> Endpoint {
        match self.direction {
            Direction::Upload => Endpoint::Client,
            Direction::Download => Endpoint::Server,
        }
    }

    /// 接收侧所在的端点。
    pub fn receiver_endpoint(&self) -> Endpoint {
        match self.direction {
            Direction::Upload => Endpoint::Server,
            Direction::Download => Endpoint::Client,
        }
    }

    /// 发起会话。**两个方向都由 Client 发起**。
    pub fn begin(&mut self) {
        match self.direction {
            // 上传：Client 是发送侧
            Direction::Upload => self.sender.start_upload(&mut self.sim),
            // 下载：Client 是接收侧
            Direction::Download => self.receiver.start_download(&mut self.sim),
        }
    }

    /// 驱动事件循环直到 `until_ms` 毫秒或没有更多事件。
    pub fn run(&mut self, until_ms: u64) {
        let deadline = until_ms * 1000;
        loop {
            let Some(next_at) = self.sim.next_at_us() else { break };
            if next_at > deadline {
                break;
            }
            self.sim.now_us = next_at;
            let Some((_, event)) = self.sim.pop_due() else { break };
            self.dispatch(event);
        }
    }

    fn dispatch(&mut self, event: Event) {
        if std::env::var("FSP_TRACE").is_ok() {
            match &event {
                Event::Control { to, msg } => {
                    eprintln!("[t={}] 控制面 → {:?}: {}", self.sim.now_ms(), to, msg.action());
                }
                Event::Datagram { to, .. } => {
                    eprintln!("[t={}] 数据面 → {:?}", self.sim.now_ms(), to);
                }
                Event::ClientSendBatch => eprintln!("[t={}] 发送批次", self.sim.now_ms()),
                Event::ServerVerdict => eprintln!("[t={}] 裁决时刻", self.sim.now_ms()),
            }
        }
        match event {
            Event::Control { to, msg } => {
                self.sim.control_delivered += 1;
                // 按端点身份路由：发送侧 / 接收侧由 direction 决定
                if to == self.sender_endpoint() {
                    let mut sender = std::mem::take(&mut self.sender);
                    sender.on_control(&msg, &mut self.sim);
                    self.sender = sender;
                } else {
                    let mut receiver = std::mem::take(&mut self.receiver);
                    receiver.on_control(&msg, &mut self.sim);
                    self.receiver = receiver;
                }
            }
            Event::Datagram { to, bytes } => {
                self.sim.datagrams_delivered += 1;
                // 数据面永远发往接收侧
                if to == self.receiver_endpoint() {
                    let mut receiver = std::mem::take(&mut self.receiver);
                    receiver.on_datagram(&bytes, &mut self.sim);
                    self.receiver = receiver;
                } else {
                    // 发错端点：按协议丢弃，不写入任何位图
                    self.sim.datagrams_dropped += 1;
                }
            }
            Event::ClientSendBatch => {
                let mut sender = std::mem::take(&mut self.sender);
                sender.on_tick(&mut self.sim);
                self.sender = sender;
            }
            Event::ServerVerdict => {
                let mut receiver = std::mem::take(&mut self.receiver);
                receiver.on_verdict_tick(&mut self.sim);
                self.receiver = receiver;
            }
        }
    }

    /// 接收端收到的内容是否与源文件一致。
    pub fn content_matches(&self) -> bool {
        digest(&self.receiver.buffer) == digest(&self.source)
    }

    /// 事件时间线（毫秒 → 事件描述），用于生成可读的失败信息。
    pub fn timeline(&self) -> BTreeMap<u64, String> {
        let mut out = BTreeMap::new();
        out.insert(
            self.sim.now_ms(),
            format!(
                "完成：verdicts={} empty_verdicts={} requests={} retransmits={:?}",
                self.receiver.verdicts_sent,
                self.receiver.empty_verdicts_sent,
                self.sender.requests_sent,
                self.sender
                    .send_counts
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| **c > 1)
                    .map(|(i, c)| (i, *c))
                    .collect::<Vec<_>>()
            ),
        );
        out
    }
}
