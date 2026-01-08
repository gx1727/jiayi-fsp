import React from 'react';
import { Lightbulb, CheckSquare } from 'lucide-react';

const ImplementationGuide = () => {
  return (
    <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-8">
      <div className="flex items-center mb-6">
        <Lightbulb className="w-6 h-6 text-yellow-500 mr-3" />
        <h2 className="text-2xl font-bold text-gray-900">实现建议</h2>
      </div>
      
      <div className="space-y-6">
        <div>
          <h3 className="font-bold text-gray-900 flex items-center">
            <CheckSquare className="w-4 h-4 text-primary-500 mr-2" />
            从状态机开始
          </h3>
          <p className="mt-1 text-gray-600 ml-6">
            不要急于实现 UDP 传输。首先实现 MQTT 控制面的状态机，确保状态流转逻辑正确。当状态机跑通时，协议就已经成功了 60%。
          </p>
        </div>

        <div>
          <h3 className="font-bold text-gray-900 flex items-center">
            <CheckSquare className="w-4 h-4 text-primary-500 mr-2" />
            Mock UDP
          </h3>
          <p className="mt-1 text-gray-600 ml-6">
            在开发初期，UDP 部分可以只做一个 Mock Sender/Receiver，专注于调试控制逻辑和异常处理。
          </p>
        </div>

        <div>
          <h3 className="font-bold text-gray-900 flex items-center">
            <CheckSquare className="w-4 h-4 text-primary-500 mr-2" />
            不要优化过早
          </h3>
          <p className="mt-1 text-gray-600 ml-6">
            家庭局域网带宽通常充足。优先保证可靠性和正确性，而非极限吞吐量。简单的滑动窗口往往比复杂的拥塞控制更有效。
          </p>
        </div>
      </div>
    </div>
  );
};

export default ImplementationGuide;
