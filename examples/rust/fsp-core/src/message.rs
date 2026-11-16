//! 控制面消息 —— 规范 `06-control-plane-mqtt.md`。
//!
//! - 所有消息是 JSON，带 `type` 字段标识消息类型。
//! - `session_id` 是 u32，但在 JSON 中**以字符串传输**（如 `"1001"`）——
//!   见 06 章「重要约束」第 1 条。
//! - Topic 固定 5 层：`/jiayi/fsp/{role}/{client_id}/{action}`（见 22 章）。

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// 协议版本（06 章 `login.proto_version`）。
pub const PROTO_VERSION: &str = "1.1.0";

/// Topic 的角色段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Client → Server。
    Client,
    /// Server → Client。
    Server,
}

impl Role {
    /// Topic 中的角色字面量。
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Client => "client",
            Role::Server => "server",
        }
    }
}

/// 会话 ID：线上以字符串承载的 u32。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId(pub u32);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for SessionId {
    fn from(v: u32) -> Self {
        SessionId(v)
    }
}

/// u32 → JSON 字符串（与参考实现的 `u32_string` serde 一致）。
impl Serialize for SessionId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

/// JSON 字符串 → u32。
impl<'de> Deserialize<'de> for SessionId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl<'d> Visitor<'d> for V {
            type Value = SessionId;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("session_id：u32 的十进制字符串形式")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<SessionId, E> {
                v.parse::<u32>()
                    .map(SessionId)
                    .map_err(|_| E::custom(format!("非法 session_id: {v:?}")))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<SessionId, E> {
                u32::try_from(v).map(SessionId).map_err(|_| E::custom("session_id 越界"))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<SessionId, E> {
                u32::try_from(v).map(SessionId).map_err(|_| E::custom("session_id 越界"))
            }

            fn visit_none<E: de::Error>(self) -> Result<SessionId, E> {
                Err(E::custom("session_id 不允许为 null"))
            }

            fn visit_unit<E: de::Error>(self) -> Result<SessionId, E> {
                Err(E::custom("session_id 不允许为 null"))
            }
        }
        deserializer.deserialize_any(V)
    }
}

/// `missing_chunks` 的判别字段（06 章）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingMode {
    /// 报告：发送方报告自己尚未收到的 chunk。
    ///
    /// 这是**缺省值**：`mode` 缺失时按 `report` 处理（06 章「缺省兼容」），
    /// 使 v1.0.0 的实现与 v1.1.0 互通时行为正确。
    #[default]
    Report,
    /// 索要：发送方索对对端的缺失列表，此时 `chunks` 必须为空数组。
    Request,
}

impl MissingMode {
    /// 缺省按 `report` 处理（与 v1.0.0 实现互通）。
    pub fn default_mode() -> Self {
        MissingMode::Report
    }
}

/// 完成类消息的结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultKind {
    /// 成功。
    Success,
    /// 失败。
    Fail,
}

/// 设备在线状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusValue {
    /// 在线。
    Online,
    /// 离线。
    Offline,
}

/// 会话方向（09 章 `direction` 字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// 发送方 Client，接收方 Server。
    Upload,
    /// 发送方 Server，接收方 Client。
    Download,
}

impl Direction {
    /// 该方向下谁是接收方（持有位图与临时文件）。
    pub fn receiver(&self) -> Role {
        match self {
            Direction::Upload => Role::Server,
            Direction::Download => Role::Client,
        }
    }

    /// 该方向下谁是发送方。
    pub fn sender(&self) -> Role {
        match self {
            Direction::Upload => Role::Client,
            Direction::Download => Role::Server,
        }
    }
}

/// 控制面消息。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlMessage {
    /// Client→Server：设备登录认证。
    Login {
        /// Client 支持的协议版本，取 `"major.minor.patch"` 形式。
        proto_version: String,
        /// 设备标识；**必须**与 Topic 中的 `{client_id}` 一致。
        device_id: String,
        /// 用户名；纯设备场景可为空。
        username: Option<String>,
    },
    /// Server→Client：登录响应。
    LoginAck {
        /// `success` 或 `fail`。
        result: ResultKind,
        /// 失败时的机器可判定错误码（19 章）；成功时为 null。
        code: Option<String>,
        /// Server 选定的协议版本。
        proto_version: String,
        /// 可读补充说明，仅用于日志与 UI。
        message: Option<String>,
    },
    /// 双向：设备在线状态报告。
    Status {
        /// 设备标识。
        device_id: String,
        /// `online` 或 `offline`。
        status: StatusValue,
    },
    /// Client→Server：发起上传。
    UploadStart {
        /// 可为空；由 Server 生成并在 accept 中回填。
        session_id: Option<SessionId>,
        /// Blake3 十六进制串。
        file_hash: String,
        /// 文件名。
        file_name: String,
        /// 文件大小（字节）。
        size: u64,
        /// 修改时间戳（UTC 秒）。
        mtime: u64,
        /// 期望的 chunk 大小。
        chunk_size: u32,
    },
    /// Server→Client：接受上传。
    UploadAccept {
        /// 回显请求中的文件哈希。
        file_hash: String,
        /// Server 生成的会话 ID。
        session_id: SessionId,
        /// 总 chunk 数量 = ceil(size / chunk_size)。
        chunk_count: u32,
        /// 滑动窗口（**在途上限**），Client 不得修改；规范默认 4096。
        window: u32,
        /// 数据面 UDP 端口。
        udp_port: u16,
        /// TCP 降级端口；**0 表示不支持 TCP 降级**。
        tcp_port: u16,
        /// 协商确定的 chunk 大小。
        chunk_size: u32,
        /// 秒传标志：Server 侧已存在相同 file_hash 时为 true。
        file_exists: bool,
    },
    /// Client→Server：请求下载。
    DownloadStart {
        /// 可为空；由 Server 生成并在 accept 中回填。
        session_id: Option<SessionId>,
        /// 要下载文件的 Blake3 十六进制串。
        file_hash: String,
        /// 文件名。
        file_name: String,
        /// 文件大小（字节）。
        size: u64,
        /// 期望的 chunk 大小。
        chunk_size: u32,
    },
    /// Server→Client：接受下载。
    DownloadAccept {
        /// 回显请求中的文件哈希。
        file_hash: String,
        /// Server 生成的会话 ID。
        session_id: SessionId,
        /// 总 chunk 数量 = ceil(size / chunk_size)。
        chunk_count: u32,
        /// 滑动窗口（**在途上限**），Client 不得修改；规范默认 4096。
        window: u32,
        /// 数据面 UDP 端口。
        udp_port: u16,
        /// TCP 降级端口；**0 表示不支持 TCP 降级**。
        tcp_port: u16,
        /// 协商确定的 chunk 大小。
        chunk_size: u32,
    },
    /// Server→Client：UDP 可用性就绪通知。
    UdpReady {
        /// 会话 ID。
        session_id: SessionId,
        /// 数据面 UDP 端口；会话重建后可能变化，Client 必须以本消息为准。
        udp_port: u16,
    },
    /// 双向：缺失 chunk 列表报告 / 索要。
    MissingChunks {
        /// `report`（缺省）或 `request`。
        #[serde(default)]
        mode: MissingMode,
        /// 会话 ID。
        session_id: SessionId,
        /// 尚未补齐的缺失块**总数**；null 表示未知（v1.0.0 对端不带此字段）。
        #[serde(default)]
        missing_count: Option<u32>,
        /// 发送方**自己尚未收到**的 chunk_id 列表（升序，单条 ≤ 4096 条）。
        chunks: Vec<u32>,
    },
    /// 双向：传输进度更新（单位为 chunk 数）。
    ProgressUpdate {
        /// 会话 ID。
        session_id: SessionId,
        /// 已接收的 chunk 数。
        received: u64,
        /// 总 chunk 数。
        total: u64,
    },
    /// 双向：暂停会话。
    SessionPause {
        /// 会话 ID。
        session_id: SessionId,
    },
    /// Client→Server：恢复中断的会话。
    ResumeSession {
        /// 会话 ID。
        session_id: SessionId,
    },
    /// Server→Client：恢复确认。
    ResumeAck {
        /// 会话 ID。
        session_id: SessionId,
        /// **续传顺序起点**：第一个尚未收到的 chunk_id。
        next_chunk_id: u32,
        /// 尚未收到的 chunk 总数。
        missing_count: u32,
        /// 数据面 UDP 端口；可能因会话重建而变化。
        udp_port: u16,
    },
    /// Server→Client：上传完成（由接收方 Server 发布）。
    UploadComplete {
        /// 会话 ID。
        session_id: SessionId,
        /// `success` 或 `fail`。
        result: ResultKind,
        /// 失败时的错误码（19 章）；成功时为 null。
        code: Option<String>,
        /// 本次传输的文件哈希。
        file_hash: Option<String>,
        /// Server 分配的素材记录 ID。
        asset_id: Option<String>,
    },
    /// Client→Server：下载完成（由接收方 Client 发布）。
    DownloadComplete {
        /// 会话 ID。
        session_id: SessionId,
        /// `success` 或 `fail`。
        result: ResultKind,
        /// 失败时的错误码（19 章）；成功时为 null。
        code: Option<String>,
        /// 本次传输的文件哈希。
        file_hash: Option<String>,
    },
    /// 双向：终止会话。
    SessionAbort {
        /// 会话 ID。
        session_id: SessionId,
        /// 机器可判定的错误码（19 章）；不得填写自由文本。
        reason: String,
        /// 可读补充说明，仅用于日志与 UI。
        detail: Option<String>,
    },
    /// Client→Server：请求撤回素材记录。
    AssetRecall {
        /// 要撤回素材的 Blake3 十六进制串。
        file_hash: String,
        /// 素材记录 ID；为 null 时按 file_hash 定位。
        asset_id: Option<String>,
    },
    /// Server→Client：撤回回执。
    AssetRecallComplete {
        /// 被撤回素材的 Blake3 十六进制串。
        file_hash: String,
        /// 素材记录 ID。
        asset_id: Option<String>,
        /// 是否已删除素材记录。
        deleted: bool,
        /// 未删除时的错误码（19 章）；成功删除时为 null。
        code: Option<String>,
        /// 可读补充说明，仅用于日志与 UI。
        detail: Option<String>,
    },
}

impl ControlMessage {
    /// 该消息的 `type` 值（即 Topic 的 action 段）。
    pub fn action(&self) -> &'static str {
        match self {
            ControlMessage::Login { .. } => "login",
            ControlMessage::LoginAck { .. } => "login_ack",
            ControlMessage::Status { .. } => "status",
            ControlMessage::UploadStart { .. } => "upload_start",
            ControlMessage::UploadAccept { .. } => "upload_accept",
            ControlMessage::DownloadStart { .. } => "download_start",
            ControlMessage::DownloadAccept { .. } => "download_accept",
            ControlMessage::UdpReady { .. } => "udp_ready",
            ControlMessage::MissingChunks { .. } => "missing_chunks",
            ControlMessage::ProgressUpdate { .. } => "progress_update",
            ControlMessage::SessionPause { .. } => "session_pause",
            ControlMessage::ResumeSession { .. } => "resume_session",
            ControlMessage::ResumeAck { .. } => "resume_ack",
            ControlMessage::UploadComplete { .. } => "upload_complete",
            ControlMessage::DownloadComplete { .. } => "download_complete",
            ControlMessage::SessionAbort { .. } => "session_abort",
            ControlMessage::AssetRecall { .. } => "asset_recall",
            ControlMessage::AssetRecallComplete { .. } => "asset_recall_complete",
        }
    }

    /// 该消息的**合法发布方向**（接收方发布完成消息，见 06 章）。
    pub fn direction(&self) -> &'static [&'static str] {
        match self {
            ControlMessage::Login { .. }
            | ControlMessage::UploadStart { .. }
            | ControlMessage::DownloadStart { .. }
            | ControlMessage::ResumeSession { .. }
            | ControlMessage::AssetRecall { .. } => &["client"],
            ControlMessage::LoginAck { .. }
            | ControlMessage::UploadAccept { .. }
            | ControlMessage::DownloadAccept { .. }
            | ControlMessage::UdpReady { .. }
            | ControlMessage::ResumeAck { .. }
            | ControlMessage::UploadComplete { .. } => &["server"],
            ControlMessage::Status { .. }
            | ControlMessage::MissingChunks { .. }
            | ControlMessage::ProgressUpdate { .. }
            | ControlMessage::SessionPause { .. }
            | ControlMessage::SessionAbort { .. }
            | ControlMessage::DownloadComplete { .. }
            | ControlMessage::AssetRecallComplete { .. } => &["client", "server"],
        }
    }

    /// 该消息涉及的会话 ID（若有）。
    pub fn session_id(&self) -> Option<SessionId> {
        match self {
            ControlMessage::UploadStart { session_id, .. }
            | ControlMessage::DownloadStart { session_id, .. } => *session_id,
            ControlMessage::UploadAccept { session_id, .. }
            | ControlMessage::DownloadAccept { session_id, .. }
            | ControlMessage::UdpReady { session_id, .. }
            | ControlMessage::MissingChunks { session_id, .. }
            | ControlMessage::ProgressUpdate { session_id, .. }
            | ControlMessage::SessionPause { session_id, .. }
            | ControlMessage::ResumeSession { session_id, .. }
            | ControlMessage::ResumeAck { session_id, .. }
            | ControlMessage::UploadComplete { session_id, .. }
            | ControlMessage::DownloadComplete { session_id, .. }
            | ControlMessage::SessionAbort { session_id, .. } => Some(*session_id),
            _ => None,
        }
    }

    /// 构造完整 Topic：`/jiayi/fsp/{role}/{client_id}/{action}`。
    pub fn topic(&self, role: Role, client_id: &str) -> String {
        format!("/jiayi/fsp/{}/{}/{}", role.as_str(), client_id, self.action())
    }
}

/// 客户端的订阅过滤器：`/jiayi/fsp/server/{client_id}/#`（22 章）。
pub fn client_subscribe_filter(client_id: &str) -> String {
    format!("/jiayi/fsp/server/{client_id}/#")
}

/// 客户端的发布前缀：`/jiayi/fsp/client/{client_id}/`（22 章）。
pub fn client_publish_prefix(client_id: &str) -> String {
    format!("/jiayi/fsp/client/{client_id}/")
}

/// Server 的订阅过滤器：`/jiayi/fsp/client/+/#`（22 章）。
pub const SERVER_SUBSCRIBE_FILTER: &str = "/jiayi/fsp/client/+/#";

/// Server 的发布前缀：`/jiayi/fsp/server/`（22 章）。
pub const SERVER_PUBLISH_PREFIX: &str = "/jiayi/fsp/server/";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_以字符串传输() {
        let msg = ControlMessage::ResumeSession { session_id: SessionId(1001) };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"session_id\":\"1001\""), "session_id 必须是字符串：{json}");
        let back: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn 可空_session_id() {
        let msg = ControlMessage::UploadStart {
            session_id: None,
            file_hash: "a".repeat(64),
            file_name: "x.bin".into(),
            size: 10,
            mtime: 0,
            chunk_size: 1200,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"session_id\":null"));
        let back: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn missing_chunks_mode_缺省为_report() {
        // v1.0.0 实现不带 mode / missing_count，按 report + 未知处理（06 章「缺省兼容」）
        let json = r#"{"type":"missing_chunks","session_id":"7","chunks":[1,2,3]}"#;
        let msg: ControlMessage = serde_json::from_str(json).unwrap();
        match msg {
            ControlMessage::MissingChunks { mode, missing_count, chunks, .. } => {
                assert_eq!(mode, MissingMode::Report);
                assert_eq!(missing_count, None, "v1.0.0 对端不带 missing_count");
                assert_eq!(chunks, vec![1, 2, 3]);
            }
            other => panic!("类型不符: {other:?}"),
        }
    }

    #[test]
    fn topic_固定五层() {
        let msg = ControlMessage::Login {
            proto_version: PROTO_VERSION.into(),
            device_id: "dev1".into(),
            username: None,
        };
        assert_eq!(msg.topic(Role::Client, "dev1"), "/jiayi/fsp/client/dev1/login");
        assert_eq!(msg.topic(Role::Server, "dev1"), "/jiayi/fsp/server/dev1/login");
    }

    #[test]
    fn 完成消息由接收方发布() {
        // 上传方向：Server 发布 upload_complete
        let up = ControlMessage::UploadComplete {
            session_id: SessionId(1),
            result: ResultKind::Success,
            code: None,
            file_hash: None,
            asset_id: None,
        };
        assert_eq!(up.direction(), &["server"]);
        // 下载方向：Client 发布 download_complete
        let down = ControlMessage::DownloadComplete {
            session_id: SessionId(1),
            result: ResultKind::Success,
            code: None,
            file_hash: None,
        };
        assert_eq!(down.direction(), &["client", "server"]);
    }
}
