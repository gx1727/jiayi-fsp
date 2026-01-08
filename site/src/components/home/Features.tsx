import React from 'react';
import { motion } from 'framer-motion';
import { Wifi, RefreshCw, Layers, Database } from 'lucide-react';

const features = [
  {
    name: '控制与数据解耦',
    description: 'MQTT 负责控制信令，UDP 负责数据传输。大脑与肌肉分离，实现高效调度。',
    icon: Layers,
  },
  {
    name: '双向同步 (New)',
    description: '支持手机备份到 PC (Upload) 及 PC 资源回流手机 (Download)，全流程可靠保障。',
    icon: Wifi,
  },
  {
    name: 'TCP 智能降级',
    description: 'UDP 不可用时自动切换 TCP 兜底，确保在恶劣网络环境下依然能完成任务。',
    icon: RefreshCw,
  },
  {
    name: '最终一致性',
    description: 'Server 端作为单一可信源（Source of Truth），确保文件完整性。',
    icon: Database,
  },
];

const Features = () => {
  return (
    <div className="py-24 bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="lg:text-center">
          <h2 className="text-base text-primary-600 font-semibold tracking-wide uppercase">核心特性</h2>
          <p className="mt-2 text-3xl leading-8 font-extrabold tracking-tight text-gray-900 sm:text-4xl">
            不仅仅是传输，更是对完美的追求
          </p>
          <p className="mt-4 max-w-2xl text-xl text-gray-500 lg:mx-auto">
            JIAYI-FSP 在设计之初就考虑了家庭网络环境的复杂性，通过创新的架构解决实际问题。
          </p>
        </div>

        <div className="mt-20">
          <dl className="space-y-10 md:space-y-0 md:grid md:grid-cols-2 md:gap-x-8 md:gap-y-10">
            {features.map((feature, index) => (
              <motion.div
                key={feature.name}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1, duration: 0.5 }}
                className="relative"
              >
                <dt>
                  <div className="absolute flex items-center justify-center h-12 w-12 rounded-md bg-primary-500 text-white">
                    <feature.icon className="h-6 w-6" aria-hidden="true" />
                  </div>
                  <p className="ml-16 text-lg leading-6 font-medium text-gray-900">{feature.name}</p>
                </dt>
                <dd className="mt-2 ml-16 text-base text-gray-500">{feature.description}</dd>
              </motion.div>
            ))}
          </dl>
        </div>
      </div>
    </div>
  );
};

export default Features;
