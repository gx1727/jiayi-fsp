import React from 'react';
import Mermaid from '../ui/Mermaid';

const ProtocolSequence = () => {
  const chart = `
sequenceDiagram
    participant Client
    participant MQTT
    participant Server

    Note over Client, Server: 1. 会话建立 (控制面)
    Client->>MQTT: upload_start
    MQTT->>Server: upload_start
    Server->>MQTT: upload_accept (session_id)
    MQTT->>Client: upload_accept (session_id)

    Note over Client, Server: 2. UDP 可用性探测
    Client->>Server: UDP_HELLO (UDP)
    Server-->>Client: UDP_HELLO_ACK (UDP)
    Server->>MQTT: udp_ready
    MQTT->>Client: udp_ready

    Note over Client, Server: 3. 数据传输 (UDP 数据面)
    Client->>Server: UDP chunk #0
    Client->>Server: UDP chunk #1
    Client->>Server: UDP chunk #2 ...
    Client->>Server: UDP chunk #N

    Note over Client, Server: 4. 缺失重传 (控制面裁决)
    Server->>MQTT: missing_chunks [3, 7, 9]
    MQTT->>Client: missing_chunks [3, 7, 9]
    Client->>Server: UDP chunk #3 (Resend)
    Client->>Server: UDP chunk #7 (Resend)
    Client->>Server: UDP chunk #9 (Resend)

    Note over Client, Server: 5. 校验与提交
    Server->>Server: Verify Hash & Commit
    Server->>MQTT: upload_complete
    MQTT->>Client: upload_complete
  `;

  return (
    <div className="mt-16">
      <h2 className="text-2xl font-bold text-gray-900 mb-8">端到端时序图 (End-to-End Sequence)</h2>
      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6 overflow-x-auto">
        <div className="min-w-[800px]">
          <Mermaid chart={chart} />
        </div>
      </div>
      <div className="mt-6 grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="bg-blue-50 p-4 rounded-lg border border-blue-100">
              <h4 className="font-bold text-blue-800 mb-2">核心流程解析</h4>
              <ul className="list-disc list-inside text-sm text-blue-700 space-y-1">
                  <li><strong>控制优先：</strong> 所有状态变更（开始、重传、完成）均由 MQTT 驱动。</li>
                  <li><strong>UDP 探测：</strong> 传输前必须确认 UDP 通路，否则降级 TCP。</li>
                  <li><strong>被动重传：</strong> Client 不猜测丢包，完全依赖 Server 的 missing_chunks 指令。</li>
              </ul>
          </div>
          <div className="bg-purple-50 p-4 rounded-lg border border-purple-100">
              <h4 className="font-bold text-purple-800 mb-2">异常处理设计</h4>
              <ul className="list-disc list-inside text-sm text-purple-700 space-y-1">
                  <li><strong>断点续传：</strong> 任意时刻中断，再次连接时仅需重传 missing 列表。</li>
                  <li><strong>乱序容忍：</strong> UDP 包到达顺序不影响写入，Chunk ID 决定落盘位置。</li>
              </ul>
          </div>
      </div>
    </div>
  );
};

export default ProtocolSequence;
