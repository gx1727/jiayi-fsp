//! 会话模型与状态机 —— 规范 `09-session-model.md` / `10-state-machine.md`。
//!
//! ## 双视角
//!
//! 同一个会话在**发送侧**与**接收侧**各自持有一份状态视图。
//! 接收侧使用全部状态；发送侧不使用 VERIFY / COMMIT —— 发送侧没有位图，
//! 无法做文件级校验。
//!
//! ## 转移表即数据
//!
//! [`LEGAL_TRANSITIONS`] 把 10 章的合法转移表原样编码成数据。
//! 这样「规范与实现一致」不再靠人工核对 —— 改表只需改一处。

use crate::bitmap::Bitmap;
use crate::message::{Direction, SessionId};
use std::fmt;

/// 会话状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionState {
    /// 初始化状态 - 会话刚创建。
    Init,
    /// 协商状态 - 正在协商传输参数。
    Negotiating,
    /// 传输状态 - 正在收发数据。
    Transferring,
    /// 暂停状态 - 会话被显式暂停，位图与临时文件保留。
    Paused,
    /// 验证状态 - 正在做文件级完整性校验。
    Verify,
    /// 提交状态 - 正在原子提交（重命名）。
    Commit,
    /// 完成状态 - 传输成功完成。
    Completed,
    /// 中止状态 - 传输被终止；终态，无出边。
    Aborted,
}

impl SessionState {
    /// 状态名的规范字面量。
    pub fn as_str(self) -> &'static str {
        match self {
            SessionState::Init => "INIT",
            SessionState::Negotiating => "NEGOTIATING",
            SessionState::Transferring => "TRANSFERRING",
            SessionState::Paused => "PAUSED",
            SessionState::Verify => "VERIFY",
            SessionState::Commit => "COMMIT",
            SessionState::Completed => "COMPLETED",
            SessionState::Aborted => "ABORTED",
        }
    }
}

impl fmt::Display for SessionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 状态视角。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionView {
    /// 发送数据的一方。
    Sender,
    /// 持有位图与临时文件的一方。
    Receiver,
}

impl SessionView {
    fn label(self) -> &'static str {
        match self {
            SessionView::Sender => "发送侧",
            SessionView::Receiver => "接收侧",
        }
    }
}

/// 合法转移的触发原因（用于日志与断言）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// `upload_start` / `download_start`
    Start,
    /// `upload_accept` / `download_accept`
    Accept,
    /// 秒传路径（`file_exists = true`）
    InstantUpload,
    /// 接收侧位图全真
    BitmapComplete,
    /// 接收方发布的完成消息
    Complete,
    /// 文件级 Blake3 校验通过
    VerifyOk,
    /// 原子重命名完成
    CommitDone,
    /// `session_pause`
    Pause,
    /// `resume_session` / `resume_ack`
    Resume,
    /// `session_abort`
    Abort,
}

impl Trigger {
    /// 触发对应的消息名（若有）。
    pub fn message(self) -> Option<&'static str> {
        match self {
            Trigger::Start => Some("upload_start/download_start"),
            Trigger::Accept => Some("upload_accept/download_accept"),
            Trigger::InstantUpload => Some("upload_accept/download_accept"),
            Trigger::BitmapComplete => None,
            Trigger::Complete => Some("upload_complete/download_complete"),
            Trigger::VerifyOk => None,
            Trigger::CommitDone => None,
            Trigger::Pause => Some("session_pause"),
            Trigger::Resume => Some("resume_session/resume_ack"),
            Trigger::Abort => Some("session_abort"),
        }
    }
}

/// 一条合法转移。
#[derive(Debug, Clone, Copy)]
pub struct Transition {
    /// 起始状态。
    pub from: SessionState,
    /// 目标状态。
    pub to: SessionState,
    /// 允许驱动该转移的视角。
    pub views: &'static [SessionView],
    /// 触发原因。
    pub trigger: Trigger,
}

const BOTH: &[SessionView] = &[SessionView::Sender, SessionView::Receiver];
const SENDER: &[SessionView] = &[SessionView::Sender];
const RECEIVER: &[SessionView] = &[SessionView::Receiver];

/// 10 章的合法状态转移表（原样编码）。
///
/// `ABORTED` 没有出边 —— 它是终态。
pub const LEGAL_TRANSITIONS: &[Transition] = &[
    Transition { from: SessionState::Init, to: SessionState::Negotiating, views: BOTH, trigger: Trigger::Start },
    Transition { from: SessionState::Init, to: SessionState::Aborted, views: BOTH, trigger: Trigger::Abort },
    Transition { from: SessionState::Negotiating, to: SessionState::Transferring, views: SENDER, trigger: Trigger::Accept },
    Transition { from: SessionState::Negotiating, to: SessionState::Verify, views: RECEIVER, trigger: Trigger::InstantUpload },
    Transition { from: SessionState::Negotiating, to: SessionState::Completed, views: SENDER, trigger: Trigger::InstantUpload },
    Transition { from: SessionState::Negotiating, to: SessionState::Aborted, views: BOTH, trigger: Trigger::Abort },
    Transition { from: SessionState::Transferring, to: SessionState::Verify, views: RECEIVER, trigger: Trigger::BitmapComplete },
    Transition { from: SessionState::Transferring, to: SessionState::Paused, views: BOTH, trigger: Trigger::Pause },
    Transition { from: SessionState::Transferring, to: SessionState::Completed, views: SENDER, trigger: Trigger::Complete },
    Transition { from: SessionState::Transferring, to: SessionState::Aborted, views: BOTH, trigger: Trigger::Abort },
    Transition { from: SessionState::Paused, to: SessionState::Transferring, views: BOTH, trigger: Trigger::Resume },
    Transition { from: SessionState::Paused, to: SessionState::Aborted, views: BOTH, trigger: Trigger::Abort },
    Transition { from: SessionState::Verify, to: SessionState::Commit, views: RECEIVER, trigger: Trigger::VerifyOk },
    Transition { from: SessionState::Verify, to: SessionState::Aborted, views: BOTH, trigger: Trigger::Abort },
    Transition { from: SessionState::Commit, to: SessionState::Completed, views: RECEIVER, trigger: Trigger::CommitDone },
    Transition { from: SessionState::Commit, to: SessionState::Aborted, views: BOTH, trigger: Trigger::Abort },
];

/// 非法状态转移。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError {
    /// 该转移不在合法转移表中。
    Illegal {
        /// 起始状态。
        from: SessionState,
        /// 目标状态。
        to: SessionState,
        /// 视角。
        view: SessionView,
    },
    /// `ABORTED` 是终态，没有出边。
    Terminal {
        /// 目标状态。
        to: SessionState,
    },
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransitionError::Illegal { from, to, view } => {
                write!(f, "非法状态转移: {from} -> {to}（{}）", view.label())
            }
            TransitionError::Terminal { to } => write!(f, "{to} 是终态，没有出边"),
        }
    }
}

impl std::error::Error for TransitionError {}

/// 查询某条转移是否合法。
pub fn is_legal(from: SessionState, to: SessionState, view: SessionView) -> bool {
    LEGAL_TRANSITIONS
        .iter()
        .any(|t| t.from == from && t.to == to && t.views.contains(&view))
}

/// 会话。
#[derive(Debug)]
pub struct Session {
    /// 会话唯一标识，Server 生成。
    pub session_id: SessionId,
    /// 客户端设备 ID。
    pub client_id: String,
    /// 会话方向。
    pub direction: Direction,
    /// 文件哈希（Blake3 十六进制串）。
    pub file_hash: String,
    /// 文件名。
    pub file_name: String,
    /// 文件大小。
    pub size: u64,
    /// 文件修改时间戳（UTC 秒）。
    pub mtime: u64,
    /// chunk 大小。
    pub chunk_size: u32,
    /// 滑动窗口（**在途上限**，Client 不得修改）。
    pub window: u32,
    /// 数据面 UDP 端口。
    pub udp_port: u16,
    /// TCP 降级端口（0 = 不支持）。
    pub tcp_port: u16,
    /// 最后活动时间戳。
    pub last_activity: u64,
    /// 接收侧状态（持有位图）。
    pub state: SessionState,
    /// 发送侧状态。
    pub sender_state: SessionState,
    /// 接收侧位图。
    pub bitmap: Bitmap,
}

impl Session {
    /// 新建一个会话。
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: SessionId,
        client_id: impl Into<String>,
        direction: Direction,
        file_hash: impl Into<String>,
        file_name: impl Into<String>,
        size: u64,
        mtime: u64,
        chunk_size: u32,
        window: u32,
        udp_port: u16,
        tcp_port: u16,
    ) -> Self {
        let chunk_count = if chunk_size == 0 { 0 } else { size.div_ceil(u64::from(chunk_size)) as u32 };
        Self {
            session_id,
            client_id: client_id.into(),
            direction,
            file_hash: file_hash.into(),
            file_name: file_name.into(),
            size,
            mtime,
            chunk_size,
            window,
            udp_port,
            tcp_port,
            last_activity: 0,
            state: SessionState::Init,
            sender_state: SessionState::Init,
            bitmap: Bitmap::new(chunk_count),
        }
    }

    /// 总 chunk 数量 = `ceil(size / chunk_size)`。
    pub fn chunk_count(&self) -> u32 {
        self.bitmap.chunk_count()
    }

    /// 在指定视角上执行一次状态转移；非法转移返回 `Err`。
    pub fn transition(
        &mut self,
        to: SessionState,
        view: SessionView,
    ) -> Result<(), TransitionError> {
        let from = match view {
            SessionView::Sender => self.sender_state,
            SessionView::Receiver => self.state,
        };
        if from == SessionState::Aborted {
            return Err(TransitionError::Terminal { to });
        }
        if !is_legal(from, to, view) {
            return Err(TransitionError::Illegal { from, to, view });
        }
        match view {
            SessionView::Sender => self.sender_state = to,
            SessionView::Receiver => self.state = to,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session {
        Session::new(
            SessionId(1),
            "dev1",
            Direction::Upload,
            "a".repeat(64),
            "x.bin",
            1200 * 10,
            0,
            1200,
            4096,
            5000,
            0,
        )
    }

    #[test]
    fn 发送侧与接收侧走不同路径() {
        let mut s = session();
        // 接收侧有 VERIFY/COMMIT，发送侧没有
        assert!(is_legal(SessionState::Verify, SessionState::Commit, SessionView::Receiver));
        assert!(!is_legal(SessionState::Verify, SessionState::Commit, SessionView::Sender));
        // 发送侧的完成来自对端的完成消息
        assert!(is_legal(SessionState::Transferring, SessionState::Completed, SessionView::Sender));
    }

    #[test]
    fn aborted_是终态() {
        let mut s = session();
        s.transition(SessionState::Negotiating, SessionView::Sender).unwrap();
        s.transition(SessionState::Aborted, SessionView::Sender).unwrap();
        assert_eq!(
            s.transition(SessionState::Transferring, SessionView::Sender),
            Err(TransitionError::Terminal { to: SessionState::Transferring })
        );
    }

    #[test]
    fn 表外转移被拒绝() {
        let mut s = session();
        let _ = &mut s;
        // INIT -> COMPLETED 不在表中（秒传的 COMPLETED 只从 NEGOTIATING 可达）
        assert!(matches!(
            s.transition(SessionState::Completed, SessionView::Sender),
            Err(TransitionError::Illegal { .. })
        ));
    }

    #[test]
    fn 秒传路径两视角各自成立() {
        let mut s = session();
        s.transition(SessionState::Negotiating, SessionView::Sender).unwrap();
        s.transition(SessionState::Negotiating, SessionView::Receiver).unwrap();
        s.transition(SessionState::Completed, SessionView::Sender).unwrap();
        s.transition(SessionState::Verify, SessionView::Receiver).unwrap();
        s.transition(SessionState::Commit, SessionView::Receiver).unwrap();
        s.transition(SessionState::Completed, SessionView::Receiver).unwrap();
    }
}
