import React from 'react';
import { Check, X, AlertCircle } from 'lucide-react';

const environments = [
  { name: '单一局域网', status: 'supported', desc: 'Server 与 Client 连接同一路由器' },
  { name: '二级路由 (NAT)', status: 'conditional', desc: 'Client 必须主动发起连接' },
  { name: '跨公网 NAT', status: 'unsupported', desc: '不支持 STUN/TURN 穿透' },
  { name: '移动蜂窝网络', status: 'unsupported', desc: '必须使用 Wi-Fi 连接' },
];

const NetworkEnvironment = () => {
  return (
    <div className="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
      <div className="px-6 py-8 border-b border-gray-100">
        <h3 className="text-xl font-bold text-gray-900">网络环境支持</h3>
        <p className="mt-2 text-gray-500">JIAYI-FSP 专为家庭环境设计，明确界定支持边界。</p>
      </div>
      <div className="divide-y divide-gray-100">
        {environments.map((env) => (
          <div key={env.name} className="px-6 py-4 flex items-center justify-between">
            <div className="flex items-center">
              {env.status === 'supported' && <Check className="w-5 h-5 text-green-500 mr-3" />}
              {env.status === 'conditional' && <AlertCircle className="w-5 h-5 text-yellow-500 mr-3" />}
              {env.status === 'unsupported' && <X className="w-5 h-5 text-red-500 mr-3" />}
              <span className="font-medium text-gray-900">{env.name}</span>
            </div>
            <span className="text-sm text-gray-500">{env.desc}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default NetworkEnvironment;
