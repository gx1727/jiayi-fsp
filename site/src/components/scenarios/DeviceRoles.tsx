import React from 'react';
import { Monitor, Smartphone } from 'lucide-react';

const DeviceRoles = () => {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mt-12">
      <div className="bg-white p-8 rounded-2xl shadow-sm border border-gray-100 flex flex-col items-center text-center">
        <div className="p-4 bg-primary-50 rounded-full mb-6">
          <Monitor className="w-12 h-12 text-primary-600" />
        </div>
        <h3 className="text-xl font-bold text-gray-900">Server (家庭 PC)</h3>
        <ul className="mt-6 space-y-3 text-left w-full">
          <li className="flex items-start">
            <span className="flex-shrink-0 h-1.5 w-1.5 rounded-full bg-primary-500 mt-2 mr-2"></span>
            <span className="text-gray-600">最终裁决者，控制面与可靠性裁决的唯一通道</span>
          </li>
          <li className="flex items-start">
            <span className="flex-shrink-0 h-1.5 w-1.5 rounded-full bg-primary-500 mt-2 mr-2"></span>
            <span className="text-gray-600">负责文件最终校验与原子提交</span>
          </li>
          <li className="flex items-start">
            <span className="flex-shrink-0 h-1.5 w-1.5 rounded-full bg-primary-500 mt-2 mr-2"></span>
            <span className="text-gray-600">乱序随机写盘，维护接收位图</span>
          </li>
        </ul>
      </div>

      <div className="bg-white p-8 rounded-2xl shadow-sm border border-gray-100 flex flex-col items-center text-center">
        <div className="p-4 bg-secondary-50 rounded-full mb-6">
          <Smartphone className="w-12 h-12 text-secondary-600" />
        </div>
        <h3 className="text-xl font-bold text-gray-900">Client (手机)</h3>
        <ul className="mt-6 space-y-3 text-left w-full">
          <li className="flex items-start">
            <span className="flex-shrink-0 h-1.5 w-1.5 rounded-full bg-secondary-500 mt-2 mr-2"></span>
            <span className="text-gray-600">发起与发送者，主动发起所有请求</span>
          </li>
          <li className="flex items-start">
            <span className="flex-shrink-0 h-1.5 w-1.5 rounded-full bg-secondary-500 mt-2 mr-2"></span>
            <span className="text-gray-600">数据分片（Chunking）、UDP 发送与重传</span>
          </li>
          <li className="flex items-start">
            <span className="flex-shrink-0 h-1.5 w-1.5 rounded-full bg-secondary-500 mt-2 mr-2"></span>
            <span className="text-gray-600">适应移动端网络切换与电量限制，支持 resume_session 恢复</span>
          </li>
        </ul>
      </div>
    </div>
  );
};

export default DeviceRoles;
