//! # fsp-core
//!
//! JIAYI-FSP 协议核心库 —— 规范 **v1.1.0** 的可执行断言。
//!
//! 本库只包含**协议语义**，不含任何 I/O：MQTT 与 UDP 由调用方（或 `fsp-sim`
//! 的虚拟实现）提供。这样协议规则可以被确定性地测试，而不受网络时序干扰。
//!
//! | 模块 | 规范章节 | 职责 |
//! |------|----------|------|
//! | [`crc32`] | 08 | CRC-32/ISO-HDLC（自实现，附规范校验值断言） |
//! | [`packet`] | 08 / 07 | UDP 数据包编解码与校验 |
//! | [`bitmap`] | 18 | 接收进度位图 |
//! | [`message`] | 06 / 22 | 控制面消息模型与 Topic 构造 |
//! | [`session`] | 09 / 10 | 会话模型与状态机（转移表即数据） |
//! | [`flow`] | 11 | 滑动窗口、AIMD、裁决节奏 |
//! | [`verdict`] | 11 / 18 | 缺失裁决的构造与调度 |
//!
//! ## 用法
//!
//! ```
//! use fsp_core::bitmap::Bitmap;
//! use fsp_core::packet::{DecodeCtx, Packet, flags};
//!
//! let ctx = DecodeCtx::new(1, 1200, 2400);
//! let pkt = Packet::new(1, 0, flags::DATA, vec![0xABu8; 1200]);
//! let bytes = pkt.encode();
//! assert_eq!(bytes.len(), 12 + 1200 + 4);
//! assert_eq!(Packet::decode(&bytes, &ctx).unwrap(), pkt);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod bitmap;
pub mod crc32;
pub mod flow;
pub mod message;
pub mod packet;
pub mod session;
pub mod verdict;

/// 本库对齐的协议版本。
pub const PROTO_VERSION: &str = message::PROTO_VERSION;
