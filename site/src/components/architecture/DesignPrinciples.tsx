import React from 'react';
import { Target, Server, AlertTriangle, WifiOff } from 'lucide-react';

const principles = [
  {
    title: '控制面与数据面解耦',
    description: '通过物理分离控制信令与数据传输，使得控制逻辑更加清晰，数据传输更加高效。',
    icon: Target,
  },
  {
    title: 'Server 端为最终一致性来源',
    description: 'Server (PC) 拥有最终裁决权，避免了复杂的分布式一致性问题。',
    icon: Server,
  },
  {
    title: 'Client 主动拉取 (Pull Model)',
    description: '无论是上传还是下载，均由 Client 发起控制和连接，适应 NAT 环境，保障安全。',
    icon: Target,
  },
  {
    title: '失败优先设计 (Failure-first)',
    description: '假设网络总是不可靠的，协议内置了强大的重传和恢复机制。',
    icon: AlertTriangle,
  },
  {
    title: '局域网优先',
    description: '不强制公网穿透，专注于家庭局域网内的高性能传输，保护用户隐私。',
    icon: WifiOff,
  },
];

const DesignPrinciples = () => {
  return (
    <div className="mt-16">
      <h2 className="text-3xl font-bold text-gray-900 mb-8 text-center">设计原则</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        {principles.map((principle) => (
          <div key={principle.title} className="bg-white p-6 rounded-xl border border-gray-200 hover:shadow-md transition-shadow">
            <div className="flex items-center mb-4">
              <div className="p-2 bg-secondary-100 rounded-lg text-secondary-600 mr-4">
                <principle.icon className="w-6 h-6" />
              </div>
              <h3 className="text-xl font-semibold text-gray-900">{principle.title}</h3>
            </div>
            <p className="text-gray-600 leading-relaxed">
              {principle.description}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
};

export default DesignPrinciples;
