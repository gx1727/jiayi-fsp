import React from 'react';
import { Github, Scale } from 'lucide-react';

const OpenSourceInfo = () => {
  return (
    <div className="mt-12 bg-gradient-to-r from-gray-900 to-gray-800 rounded-2xl p-8 text-white shadow-lg">
      <div className="flex flex-col md:flex-row items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold mb-2">开源共建</h2>
          <p className="text-gray-300 max-w-xl">
            JIAYI-FSP 是一个开放协议。我们欢迎社区贡献代码、提交 Issue 或完善文档。
          </p>
        </div>
        <div className="mt-6 md:mt-0 flex space-x-4">
          <a href="#" className="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-gray-900 bg-white hover:bg-gray-50 transition-colors">
            <Github className="w-5 h-5 mr-2" />
            GitHub Repo
          </a>
        </div>
      </div>
      
      <div className="mt-8 pt-8 border-t border-gray-700 flex items-center text-sm text-gray-400">
        <Scale className="w-4 h-4 mr-2" />
        <span>建议协议：Apache License 2.0 或 MIT License</span>
      </div>
    </div>
  );
};

export default OpenSourceInfo;
