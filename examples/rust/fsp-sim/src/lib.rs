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
//! - 重复 chunk **必须**幂等丢弃（19 章）。
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

use std::collections::BTreeMap;

/// 仿真世界：虚拟引擎 + 发送侧 + 接收侧。
#[derive(Debug)]
pub struct World {
    /// 仿真引擎。
    pub sim: Sim,
    /// 发送侧代理（上传方向 = Client）。
    pub client: SenderAgent,
    /// 接收侧代理（上传方向 = Server）。
    pub server: ReceiverAgent,
    /// 源文件内容（用于校验接收结果）。
    pub source: Vec<u8>,
}

impl Default for SenderAgent {
    fn default() -> Self {
        SenderAgent::new("client1", Vec::new(), fsp_core::packet::DEFAULT_CHUNK_SIZE, fsp_core::packet::DEFAULT_WINDOW)
    }
}

impl Default for ReceiverAgent {
    fn default() -> Self {
        ReceiverAgent::new(Vec::new())
    }
}

impl World {
    /// 新建仿真世界：`file` 是要传输的内容。
    pub fn new(file: Vec<u8>, chunk_size: u32, window: u32, seed: u64) -> Self {
        World {
            sim: Sim::new(seed),
            client: SenderAgent::new("client1", file.clone(), chunk_size, window),
            server: ReceiverAgent::new(file.clone()),
            source: file,
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
                match to {
                Endpoint::Client => {
                    let mut client = std::mem::take(&mut self.client);
                    client.on_control(&msg, &mut self.sim);
                    self.client = client;
                }
                Endpoint::Server => {
                    let mut server = std::mem::take(&mut self.server);
                    server.on_control(&msg, &mut self.sim);
                    self.server = server;
                }
                }
            }
            Event::Datagram { to, bytes } => {
                self.sim.datagrams_delivered += 1;
                match to {
                Endpoint::Client => {
                    // 上传方向下 Client 不是接收方
                }
                Endpoint::Server => {
                    let mut server = std::mem::take(&mut self.server);
                    server.on_datagram(&bytes, &mut self.sim);
                    self.server = server;
                }
                }
            }
            Event::ClientSendBatch => {
                let mut client = std::mem::take(&mut self.client);
                client.on_tick(&mut self.sim);
                self.client = client;
            }
            Event::ServerVerdict => {
                let mut server = std::mem::take(&mut self.server);
                server.on_verdict_tick(&mut self.sim);
                self.server = server;
            }
        }
    }

    /// 接收端收到的内容是否与源文件一致。
    pub fn content_matches(&self) -> bool {
        digest(&self.server.buffer) == digest(&self.source)
    }

    /// 事件时间线（毫秒 → 事件描述），用于生成可读的失败信息。
    pub fn timeline(&self) -> BTreeMap<u64, String> {
        let mut out = BTreeMap::new();
        out.insert(self.sim.now_ms(), format!(
            "完成：verdicts={} empty_verdicts={} requests={} retransmits={:?}",
            self.server.verdicts_sent,
            self.server.empty_verdicts_sent,
            self.client.requests_sent,
            self.client
                .send_counts
                .iter()
                .enumerate()
                .filter(|(_, c)| **c > 1)
                .map(|(i, c)| (i, *c))
                .collect::<Vec<_>>()
        ));
        out
    }
}
