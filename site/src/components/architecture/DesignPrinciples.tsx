import React from 'react';
import { Target, Server, Layers, Zap, Database, Wifi } from 'lucide-react';

const principles = [
  {
    title: '控制面与数据面解耦',
    description: 'MQTT 负责控制信令，UDP 负责数据传输。控制面拥有最终裁决权，数据面仅负责搬运与落盘。',
    icon: Layers,
  },
  {
    title: 'Server 为最终裁决者',
    description: 'Server 是最终一致性来源，所有状态推进由 MQTT 控制消息触发，UDP 数据面不得改变会话状态。',
    icon: Server,
  },
  {
    title: '仅 MQTT 推进状态机',
    description: '状态机仅由控制面推进，数据面不得直接改变会话状态。',
    icon: Target,
  },
  {
    title: '控制慢而可靠，数据快而可丢',
    description: '状态推进、裁决、重传在 MQTT；高吞吐在 UDP/TCP。',
    icon: Zap,
  },
  {
    title: '内存与文件大小解耦',
    description: '使用随机写盘与位图，内存占用仅与 chunk_size 成正比，不随文件大小增长。',
    icon: Database,
  },
  {
    title: '局域网优先',
    description: '专注于家庭局域网内的高性能传输，不提供公网穿透，保护用户隐私。',
    icon: Wifi,
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
