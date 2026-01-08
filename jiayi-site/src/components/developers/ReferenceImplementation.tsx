import React from 'react';
import { Server, Smartphone, Database } from 'lucide-react';

const ReferenceImplementation = () => {
  return (
    <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
      <div className="bg-gray-50 rounded-xl p-6 border border-gray-100">
        <Server className="w-8 h-8 text-gray-700 mb-4" />
        <h3 className="text-lg font-bold text-gray-900">Server 端</h3>
        <p className="mt-2 text-sm text-gray-500">推荐使用 Rust + Tokio 实现，利用其内存安全和高并发特性处理 UDP 数据流。</p>
      </div>

      <div className="bg-gray-50 rounded-xl p-6 border border-gray-100">
        <Smartphone className="w-8 h-8 text-gray-700 mb-4" />
        <h3 className="text-lg font-bold text-gray-900">Client 端</h3>
        <p className="mt-2 text-sm text-gray-500">Android (Kotlin) / iOS (Swift)。需注意后台保活和电源管理策略。</p>
      </div>

      <div className="bg-gray-50 rounded-xl p-6 border border-gray-100">
        <Database className="w-8 h-8 text-gray-700 mb-4" />
        <h3 className="text-lg font-bold text-gray-900">存储层</h3>
        <p className="mt-2 text-sm text-gray-500">文件系统存储原始文件，SQLite 存储文件索引和元数据。</p>
      </div>
    </div>
  );
};

export default ReferenceImplementation;
