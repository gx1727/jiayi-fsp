import React from 'react';
import Mermaid from '../ui/Mermaid';

const StateMachine = () => {
  const chart = `
stateDiagram-v2
    [*] --> IDLE
    IDLE --> SESSION_INIT: sync_start
    SESSION_INIT --> TRANSFER: 协商 chunk / window
    
    state TRANSFER {
        [*] --> Sending
        Sending --> Missing: missing_chunks
        Missing --> Sending: resend
    }
    
    TRANSFER --> VERIFY: transfer_complete
    VERIFY --> COMMIT: hash OK
    VERIFY --> TRANSFER: verify_fail
    COMMIT --> DONE: rename + index
    DONE --> [*]

    note right of TRANSFER
        UDP 数据流在此阶段传输
    end note
  `;

  return (
    <div className="bg-white rounded-2xl shadow-lg p-8 border border-gray-100 mt-12">
      <h2 className="text-2xl font-bold text-gray-900 mb-6">会话生命周期状态机</h2>
      <div className="flex justify-center">
        <Mermaid chart={chart} />
      </div>
      <div className="mt-6 text-gray-600 space-y-2">
        <p><span className="font-bold text-gray-900">IDLE:</span> 空闲状态，等待同步请求。</p>
        <p><span className="font-bold text-gray-900">SESSION_INIT:</span> 收到请求，进行参数协商（Chunk大小、窗口大小）。</p>
        <p><span className="font-bold text-gray-900">TRANSFER:</span> UDP 数据传输阶段，包含重传循环。</p>
        <p><span className="font-bold text-gray-900">VERIFY:</span> 传输完成，进行完整性校验（Hash比对）。</p>
        <p><span className="font-bold text-gray-900">COMMIT:</span> 校验通过，原子写入文件并建立索引。</p>
        <p><span className="font-bold text-gray-900">DONE:</span> 同步完成。</p>
      </div>
      <div className="mt-4 p-4 bg-yellow-50 border-l-4 border-yellow-400 text-yellow-700">
        <p className="text-sm font-bold">注意</p>
        <p className="text-sm">任何阶段异常都可能导致回退到 SESSION_INIT 或 TRANSFER 状态，确保系统具备自愈能力。</p>
      </div>
    </div>
  );
};

export default StateMachine;
