import React from 'react';
import Mermaid from '../ui/Mermaid';

const StateMachine = () => {
  const chart = `
stateDiagram-v2
    [*] --> INIT
    INIT --> NEGOTIATING: upload_start
    INIT --> ABORTED: session_abort

    NEGOTIATING --> TRANSFERRING: upload_accept
    NEGOTIATING --> VERIFY: 秒传 file_exists=true
    NEGOTIATING --> ABORTED: session_abort

    TRANSFERRING --> VERIFY: 位图全真
    TRANSFERRING --> PAUSED: session_pause
    TRANSFERRING --> ABORTED: session_abort

    PAUSED --> TRANSFERRING: resume_session
    PAUSED --> ABORTED: session_abort

    VERIFY --> COMMIT: Blake3 校验通过
    VERIFY --> ABORTED: FILE_HASH_MISMATCH

    COMMIT --> COMPLETED: 原子重命名完成
    COMMIT --> ABORTED: session_abort

    COMPLETED --> [*]
    ABORTED --> [*]

    note right of TRANSFERRING
        接收侧：UDP 只更新位图
        发送侧：等待 missing_chunks 裁决推进
        裁决周期 50ms，空数组亦须下发
    end note
`;

  return (
    <div className="bg-white rounded-2xl shadow-lg p-8 border border-gray-100 mt-12">
      <h2 className="text-2xl font-bold text-gray-900 mb-6">会话生命周期状态机</h2>
      <div className="flex justify-center">
        <Mermaid chart={chart} />
      </div>
      <div className="mt-6 text-gray-600 space-y-2">
        <p><span className="font-bold text-gray-900">INIT:</span> 会话初始化，等待 upload_start。</p>
        <p><span className="font-bold text-gray-900">NEGOTIATING:</span> 参数协商（chunk_size、window 等），由 upload_accept 进入传输。</p>
        <p><span className="font-bold text-gray-900">TRANSFERRING:</span> UDP 数据传输阶段，缺失统计与重传由 MQTT 裁决驱动。</p>
        <p><span className="font-bold text-gray-900">PAUSED:</span> 会话被显式暂停，位图与临时文件保留，可恢复。</p>
        <p><span className="font-bold text-gray-900">VERIFY:</span> 接收侧位图全真后做文件级完整性校验（Blake3）。</p>
        <p><span className="font-bold text-gray-900">COMMIT:</span> 校验通过，原子重命名临时文件并提交。</p>
        <p><span className="font-bold text-gray-900">COMPLETED:</span> 会话完成，由接收方发布完成消息触发。</p>
        <p><span className="font-bold text-gray-900">ABORTED:</span> 任意阶段致命错误或超时终止，由 session_abort 触发（终态）。</p>
      </div>
      <div className="mt-6 p-4 bg-blue-50 border-l-4 border-blue-400 text-blue-700">
        <p className="text-sm font-bold">发送侧 / 接收侧两个视角</p>
        <p className="text-sm mt-1">
          上传方向的接收方是 Server、发送方是 Client；下载方向相反。
          发送侧状态仅由收到的 MQTT 控制消息推进；接收侧由本地事件推进，
          但每次推进必须通过 MQTT 同步给对端。
        </p>
        <p className="text-sm mt-1">
          <strong>完成消息一律由接收方发布</strong>：上传方向 Server 发布 upload_complete，
          下载方向 Client 发布 download_complete。
        </p>
      </div>
      <div className="mt-4 p-4 bg-yellow-50 border-l-4 border-yellow-400 text-yellow-700">
        <p className="text-sm font-bold">注意</p>
        <p className="text-sm">UDP 数据面只更新位图，不直接触发状态转移；位图变化是接收侧推进的输入。</p>
      </div>
    </div>
  );
};

export default StateMachine;
