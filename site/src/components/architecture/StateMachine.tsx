import React from 'react';
import Mermaid from '../ui/Mermaid';

const StateMachine = () => {
  const chart = `
stateDiagram-v2
    [*] --> INIT
    INIT --> NEGOTIATING: upload_start
    NEGOTIATING --> TRANSFERRING: upload_accept

    TRANSFERRING --> COMPLETED: upload_complete
    TRANSFERRING --> ABORTED: session_abort

    TRANSFERRING --> VERIFY: optional
    VERIFY --> COMMIT: hash OK
    VERIFY --> TRANSFERRING: verify_fail
    COMMIT --> COMPLETED: commit_done

    COMPLETED --> [*]
    ABORTED --> [*]

    note right of TRANSFERRING
        UDP 数据面在此阶段传输
        状态推进仅由 MQTT 控制面触发
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
        <p><span className="font-bold text-gray-900">NEGOTIATING:</span> 参数协商（chunk_size、window等），由 upload_accept 进入传输。</p>
        <p><span className="font-bold text-gray-900">TRANSFERRING:</span> UDP 数据传输阶段，缺失统计与重传由 MQTT 触发。</p>
        <p><span className="font-bold text-gray-900">VERIFY（可选）:</span> 传输完成后的完整性校验（内容哈希比对）。</p>
        <p><span className="font-bold text-gray-900">COMMIT（可选）:</span> 校验通过，原子重命名临时文件并提交。</p>
        <p><span className="font-bold text-gray-900">COMPLETED:</span> 会话完成，由 upload_complete 触发。</p>
        <p><span className="font-bold text-gray-900">ABORTED:</span> 任意阶段致命错误或超时终止，由 session_abort 触发。</p>
      </div>
      <div className="mt-4 p-4 bg-yellow-50 border-l-4 border-yellow-400 text-yellow-700">
        <p className="text-sm font-bold">注意</p>
        <p className="text-sm">状态推进仅由 MQTT 控制消息触发；UDP 数据面不直接改变会话状态。</p>
      </div>
    </div>
  );
};

export default StateMachine;
