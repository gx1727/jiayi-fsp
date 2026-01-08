import React from 'react';
import { motion } from 'framer-motion';
import { ArrowRight, Shield, Zap, Lock } from 'lucide-react';
import { Link } from 'react-router-dom';

const Hero = () => {
  return (
    <div className="relative overflow-hidden bg-white pt-16 pb-32">
      <div className="absolute inset-0 z-0 opacity-30">
        <div className="absolute top-0 left-0 w-96 h-96 bg-primary-200 rounded-full mix-blend-multiply filter blur-3xl opacity-70 animate-blob"></div>
        <div className="absolute top-0 right-0 w-96 h-96 bg-secondary-200 rounded-full mix-blend-multiply filter blur-3xl opacity-70 animate-blob animation-delay-2000"></div>
        <div className="absolute -bottom-8 left-20 w-96 h-96 bg-pink-200 rounded-full mix-blend-multiply filter blur-3xl opacity-70 animate-blob animation-delay-4000"></div>
      </div>

      <div className="relative z-10 container mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
        >
          <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-primary-50 text-primary-700 mb-8">
            <span className="flex h-2 w-2 relative mr-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2 w-2 bg-primary-500"></span>
            </span>
            JIAYI-FSP 1.0 现已发布
          </span>
          
          <h1 className="text-5xl md:text-7xl font-extrabold tracking-tight text-gray-900 mb-8">
            局域网文件同步的
            <span className="block text-transparent bg-clip-text bg-gradient-to-r from-primary-600 to-secondary-600 mt-2">
              终极协议
            </span>
          </h1>
          
          <p className="mt-4 max-w-2xl mx-auto text-xl text-gray-500 mb-10">
            专为局域网环境设计。
            <span className="font-semibold text-gray-700">可靠</span>、
            <span className="font-semibold text-gray-700">隐私</span>、
            <span className="font-semibold text-gray-700">高速</span>。
            <br />
            让您的照片和视频在手机与电脑间无缝流转。
          </p>

          <div className="flex justify-center gap-4">
            <Link
              to="/specification"
              className="inline-flex items-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 md:text-lg transition-all duration-200 shadow-lg hover:shadow-xl hover:-translate-y-1"
            >
              查看技术规范
              <ArrowRight className="ml-2 -mr-1 w-5 h-5" />
            </Link>
            <Link
              to="/architecture"
              className="inline-flex items-center px-8 py-3 border border-gray-300 text-base font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 md:text-lg transition-all duration-200"
            >
              了解架构设计
            </Link>
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4, duration: 0.8 }}
          className="mt-20 grid grid-cols-1 gap-8 sm:grid-cols-3"
        >
          <div className="flex flex-col items-center p-6 bg-white rounded-2xl shadow-xl border border-gray-100 hover:border-primary-100 transition-colors">
            <div className="p-3 bg-primary-50 rounded-full mb-4">
              <Shield className="w-8 h-8 text-primary-600" />
            </div>
            <h3 className="text-lg font-bold text-gray-900">可靠传输</h3>
            <p className="mt-2 text-gray-500">
              基于状态机和滑动窗口的重传机制，确保每一个字节都准确无误。
            </p>
          </div>

          <div className="flex flex-col items-center p-6 bg-white rounded-2xl shadow-xl border border-gray-100 hover:border-secondary-100 transition-colors">
            <div className="p-3 bg-secondary-50 rounded-full mb-4">
              <Lock className="w-8 h-8 text-secondary-600" />
            </div>
            <h3 className="text-lg font-bold text-gray-900">隐私优先</h3>
            <p className="mt-2 text-gray-500">
              纯局域网传输，数据不经过第三方服务器，完全掌控您的数据。
            </p>
          </div>

          <div className="flex flex-col items-center p-6 bg-white rounded-2xl shadow-xl border border-gray-100 hover:border-pink-100 transition-colors">
            <div className="p-3 bg-pink-50 rounded-full mb-4">
              <Zap className="w-8 h-8 text-pink-600" />
            </div>
            <h3 className="text-lg font-bold text-gray-900">极速体验</h3>
            <p className="mt-2 text-gray-500">
              UDP 数据流 + MQTT 控制流，最大化利用家庭网络带宽。
            </p>
          </div>
        </motion.div>
      </div>
    </div>
  );
};

export default Hero;
