//! 确定性仿真骨架：虚拟时钟、虚拟 Broker（控制面）、虚拟 UDP（数据面）。
//!
//! 为什么要仿真而不是直接连真 Broker：
//! - **可复现**：丢包 / 乱序 / 重复由带种子的 PRNG 决定，同一场景永远同一结果；
//! - **可断言**：规范性语义（例如「零丢包时不得卡死」「空洞块必须被补发」）
//!   可以被写成断言，而不是靠"跑通了顺便发现"；
//! - **零外部依赖**：CI 可直接跑，不需要 mosquitto。

use crate::rng::Rng;
use fsp_core::message::ControlMessage;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// 仿真端点。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Endpoint {
    /// 客户端。
    Client,
    /// 服务端。
    Server,
}

/// 事件。
#[derive(Debug, Clone)]
pub enum Event {
    /// 控制面投递（模拟 MQTT 消息送达）。
    Control {
        /// 目标端点。
        to: Endpoint,
        /// 消息体。
        msg: ControlMessage,
    },
    /// 数据面投递（模拟 UDP 报文送达）。
    Datagram {
        /// 目标端点。
        to: Endpoint,
        /// 线上字节。
        bytes: Vec<u8>,
    },
    /// 发送方请求发送下一批。
    ClientSendBatch,
    /// 接收方应当下发一次缺失裁决。
    ServerVerdict,
}

/// 事件队列条目。
#[derive(Debug)]
struct Entry {
    at_us: u64,
    seq: u64,
    event: Event,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.at_us == other.at_us && self.seq == other.seq
    }
}
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 升序：(时刻, 序号)。配合 `Reverse` 即为「时刻最小者先出，
        // 同刻按序号升序」——必须是**无歧义**的，否则事件会被跳过。
        self.at_us.cmp(&other.at_us).then(self.seq.cmp(&other.seq))
    }
}

/// 控制面故障注入配置。
#[derive(Debug, Clone, Default)]
pub struct ControlFaults {
    /// 按消息 action 精确丢弃（用于模拟「裁决丢失」）。
    pub drop_actions: Vec<&'static str>,
    /// 丢弃控制消息的概率分子 / 分母。
    pub loss_num: u64,
    /// 丢弃控制消息的概率分母。
    pub loss_den: u64,
    /// 重复投递控制消息的概率分子 / 分母（模拟 QoS 1 的重复投递）。
    pub dup_num: u64,
    /// 重复投递控制消息的概率分母。
    pub dup_den: u64,
    /// 控制消息投递延迟（毫秒）。
    pub delay_ms: u64,
}

/// 数据面故障注入配置。
#[derive(Debug, Clone, Default)]
pub struct DataFaults {
    /// 按 chunk_id 精确丢弃（用于构造确定的「空洞」）。
    pub drop_chunk_ids: Vec<u32>,
    /// 按 chunk_id 精确损坏（翻转一个字节，用于验证 CRC32 丢弃）。
    pub corrupt_chunk_ids: Vec<u32>,
    /// 丢包概率分子 / 分母。
    pub loss_num: u64,
    /// 丢包概率分母。
    pub loss_den: u64,
    /// 重复概率分子 / 分母。
    pub dup_num: u64,
    /// 重复概率分母。
    pub dup_den: u64,
    /// 乱序延迟上界（毫秒）—— 报文可能被推迟，造成乱序到达。
    pub reorder_ms: u64,
}

/// 仿真引擎：虚拟时钟 + 事件队列 + 故障注入。
#[derive(Debug)]
pub struct Sim {
    /// 当前虚拟时刻（微秒）。
    pub now_us: u64,
    /// 事件序号。
    seq: u64,
    /// 事件队列。
    queue: BinaryHeap<Reverse<Entry>>,
    /// 控制面故障注入。
    pub control_faults: ControlFaults,
    /// 数据面故障注入。
    pub data_faults: DataFaults,
    /// 伪随机数发生器（带种子，保证可复现）。
    pub rng: Rng,
    /// 已投递的控制消息计数。
    pub control_delivered: u64,
    /// 已投递的数据报计数。
    pub datagrams_delivered: u64,
    /// 已丢弃的控制消息计数。
    pub control_dropped: u64,
    /// 已丢弃的数据报计数。
    pub datagrams_dropped: u64,
    /// 已注入过故障的 chunk。
    ///
    /// 故障只作用于**首次**出现 —— 否则被丢/被损坏的块永远补不回来，
    /// 重传会陷入无限循环（而真实网络中丢包是随机的）。
    pub faulted: std::collections::BTreeSet<u32>,
}

impl Default for Sim {
    fn default() -> Self {
        Sim {
            now_us: 0,
            seq: 0,
            queue: BinaryHeap::new(),
            control_faults: ControlFaults::default(),
            data_faults: DataFaults::default(),
            rng: Rng::new(0x5EED),
            control_delivered: 0,
            datagrams_delivered: 0,
            control_dropped: 0,
            datagrams_dropped: 0,
            faulted: std::collections::BTreeSet::new(),
        }
    }
}

impl Sim {
    /// 新建仿真。
    pub fn new(seed: u64) -> Self {
        Sim { rng: Rng::new(seed), ..Sim::default() }
    }

    /// 当前时刻（毫秒）。
    pub fn now_ms(&self) -> u64 {
        self.now_us / 1000
    }

    /// 安排一个事件在 `delay_ms` 毫秒后触发。
    pub fn schedule(&mut self, delay_ms: u64, event: Event) {
        let at_us = self.now_us + delay_ms * 1000;
        self.seq += 1;
        let seq = self.seq;
        self.queue.push(Reverse(Entry { at_us, seq, event }));
    }

    /// 立即安排一个事件（时刻 = 当前时刻）。
    pub fn schedule_now(&mut self, event: Event) {
        self.schedule(0, event);
    }

    /// 安排一个事件在 `delay_us` 微秒后触发。
    pub fn schedule_us(&mut self, delay_us: u64, event: Event) {
        let at_us = self.now_us + delay_us;
        self.seq += 1;
        let seq = self.seq;
        self.queue.push(Reverse(Entry { at_us, seq, event }));
    }

    /// 下一个待处理事件的时刻（微秒）；无事件时返回 `None`。
    pub fn next_at_us(&self) -> Option<u64> {
        self.queue.peek().map(|Reverse(e)| e.at_us)
    }

    /// 取出下一个事件（若其时刻已到）。
    pub fn pop_due(&mut self) -> Option<(u64, Event)> {
        match self.queue.peek() {
            Some(Reverse(entry)) if entry.at_us <= self.now_us => {
                let Reverse(entry) = self.queue.pop().expect("peek 成功则 pop 必成功");
                self.now_us = entry.at_us;
                Some((entry.at_us, entry.event))
            }
            _ => None,
        }
    }

    /// 投递一条控制消息（应用控制面故障注入）。
    pub fn post_control(&mut self, to: Endpoint, msg: ControlMessage) {
        let (loss_num, loss_den, dup_num, dup_den, delay) = {
            let f = &self.control_faults;
            (f.loss_num, f.loss_den, f.dup_num, f.dup_den, f.delay_ms)
        };
        if self.control_faults.drop_actions.contains(&msg.action()) {
            self.control_dropped += 1;
            return;
        }
        if loss_den != 0 && self.rng.chance(loss_num, loss_den) {
            self.control_dropped += 1;
            return;
        }
        if std::env::var("FSP_TRACE").is_ok() {
            eprintln!("  [post_control] to={:?} action={} seq_will_be={}", to, msg.action(), self.seq + 1);
        }
        self.schedule(delay, Event::Control { to, msg: msg.clone() });
        if dup_den != 0 && self.rng.chance(dup_num, dup_den) {
            // QoS 1 的重复投递：接收方必须幂等地处理
            self.schedule(delay, Event::Control { to, msg });
            self.control_delivered += 1;
        }
    }

    /// 投递一个数据报（应用数据面故障注入）。
    pub fn post_datagram(&mut self, to: Endpoint, mut bytes: Vec<u8>) {
        let (loss_num, loss_den, dup_num, dup_den, reorder_ms) = {
            let f = &self.data_faults;
            (f.loss_num, f.loss_den, f.dup_num, f.dup_den, f.reorder_ms)
        };
        // 从字节流中取出 chunk_id（偏移 4..8，小端），用于精确故障注入
        let chunk_id = if bytes.len() >= 8 {
            Some(u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))
        } else {
            None
        };
        if let Some(id) = chunk_id {
            if self.data_faults.drop_chunk_ids.contains(&id) && !self.faulted.contains(&id) {
                // 只丢**首次**出现 —— 否则该块永远补不回来
                self.faulted.insert(id);
                self.datagrams_dropped += 1;
                return;
            }
            if self.data_faults.corrupt_chunk_ids.contains(&id) && !self.faulted.contains(&id) {
                // 只损坏**首次**出现：翻转 payload 的第一个字节 —— CRC32 必然失配
                self.faulted.insert(id);
                if bytes.len() > 12 {
                    bytes[12] ^= 0xFF;
                }
            }
        }
        if loss_den != 0 && self.rng.chance(loss_num, loss_den) {
            self.datagrams_dropped += 1;
            return;
        }
        // 乱序：报文可能被推迟
        let delay = if reorder_ms == 0 { 0 } else { self.rng.below(reorder_ms + 1) };
        self.schedule(delay, Event::Datagram { to, bytes: bytes.clone() });
        if dup_den != 0 && self.rng.chance(dup_num, dup_den) {
            self.schedule(delay, Event::Datagram { to, bytes });
            self.datagrams_delivered += 1;
        }
    }
}
