import React from 'react';
import Mermaid from '../ui/Mermaid';

const ArchitectureDiagram = () => {
  const chart = `
flowchart TB
    subgraph App["应用层 (家忆)"]
        Scanner["文件扫描 / 去重 / 目录策略"]
    end

    subgraph Control["控制层 (MQTT)"]
        Session["会话 / 状态 / 调度 / ACK"]
    end

    subgraph Data["传输层 (UDP + TCP)"]
        direction TB
        Chunk["UDP Chunk 流 (Primary)"]
        Fallback["TCP Stream (Fallback)"]
    end

    subgraph Network["网络层 (LAN)"]
        Conn["Wi-Fi / Ethernet"]
    end

    Scanner --> Session
    Session --> Chunk
    Session --> Fallback
    Chunk --> Conn
    Fallback --> Conn

    style App fill:#dbeafe,stroke:#2563eb,stroke-width:2px
    style Control fill:#ede9fe,stroke:#7c3aed,stroke-width:2px
    style Data fill:#fce7f3,stroke:#db2777,stroke-width:2px
    style Network fill:#f3f4f6,stroke:#4b5563,stroke-width:2px
  `;

  return (
    <div className="bg-white rounded-2xl shadow-lg p-8 border border-gray-100">
      <h2 className="text-2xl font-bold text-gray-900 mb-6">协议分层总览</h2>
      <div className="flex justify-center">
        <Mermaid chart={chart} />
      </div>
      <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6 text-left">
        <div className="bg-primary-50 p-4 rounded-lg">
            <h3 className="font-bold text-primary-700">MQTT = 大脑</h3>
            <p className="text-sm text-primary-600 mt-1">负责指挥、记账、纠错。通过轻量级的发布/订阅模式，维护全局状态一致性。</p>
        </div>
        <div className="bg-pink-50 p-4 rounded-lg">
            <h3 className="font-bold text-pink-700">UDP = 肌肉</h3>
            <p className="text-sm text-pink-600 mt-1">只负责把数据甩出去。利用无连接特性，最大化利用带宽，不进行拥塞控制。</p>
        </div>
        <div className="bg-purple-50 p-4 rounded-lg">
            <h3 className="font-bold text-purple-700">TCP = 保险</h3>
            <p className="text-sm text-purple-600 mt-1">当 UDP 彻底不可用时自动降级，保证“慢一点但能传完”，不追求极致性能。</p>
        </div>
      </div>
    </div>
  );
};

export default ArchitectureDiagram;
