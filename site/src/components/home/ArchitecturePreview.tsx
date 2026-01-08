import React from 'react';
import { Link } from 'react-router-dom';
import { ArrowRight } from 'lucide-react';
import Mermaid from '../ui/Mermaid';

const ArchitecturePreview = () => {
  const chart = `
graph TD
    A[应用层] --> B[控制层 MQTT]
    A --> C[数据层 UDP]
    B --> D[网络层 LAN]
    C --> D
    style A fill:#eff6ff,stroke:#3b82f6,stroke-width:2px
    style B fill:#f5f3ff,stroke:#8b5cf6,stroke-width:2px
    style C fill:#fce7f3,stroke:#ec4899,stroke-width:2px
    style D fill:#f3f4f6,stroke:#9ca3af,stroke-width:2px
  `;

  return (
    <div className="py-24 bg-white overflow-hidden">
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="relative lg:grid lg:grid-cols-2 lg:gap-8 lg:items-center">
          <div className="relative">
            <h3 className="text-2xl font-extrabold text-gray-900 tracking-tight sm:text-3xl">
              分层架构，清晰至极
            </h3>
            <p className="mt-3 text-lg text-gray-500">
              我们将复杂的同步过程拆解为清晰的层次。应用层负责业务逻辑，控制层通过 MQTT 维护状态，数据层利用 UDP 极速传输，网络层适配局域网环境。
            </p>

            <div className="mt-10 space-y-4">
              <div className="flex items-start">
                <div className="flex-shrink-0">
                  <span className="flex items-center justify-center h-8 w-8 rounded-full bg-primary-100 text-primary-600 font-bold">1</span>
                </div>
                <div className="ml-4">
                  <h4 className="text-lg font-medium text-gray-900">MQTT 控制面</h4>
                  <p className="mt-1 text-gray-500">会话管理、状态同步、调度指令。</p>
                </div>
              </div>
              <div className="flex items-start">
                <div className="flex-shrink-0">
                  <span className="flex items-center justify-center h-8 w-8 rounded-full bg-secondary-100 text-secondary-600 font-bold">2</span>
                </div>
                <div className="ml-4">
                  <h4 className="text-lg font-medium text-gray-900">UDP 数据面</h4>
                  <p className="mt-1 text-gray-500">Chunk 数据流，无连接，低延迟。</p>
                </div>
              </div>
            </div>

            <div className="mt-10">
              <Link
                to="/architecture"
                className="inline-flex items-center font-medium text-primary-600 hover:text-primary-500"
              >
                深入了解架构细节
                <ArrowRight className="ml-2 h-4 w-4" />
              </Link>
            </div>
          </div>

          <div className="mt-10 -mx-4 relative lg:mt-0" aria-hidden="true">
            <div className="relative mx-auto w-full rounded-lg shadow-lg lg:max-w-md bg-white p-6 border border-gray-100">
               <div className="flex justify-center">
                  <Mermaid chart={chart} />
               </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ArchitecturePreview;
