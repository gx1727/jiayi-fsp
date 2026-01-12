import React from 'react';
import { Image, Video, HardDrive, Download } from 'lucide-react';

const cases = [
  {
    title: '每日照片备份',
    desc: '每晚充电时，自动将当天拍摄的高清照片增量同步到 PC。',
    icon: Image,
  },
  {
    title: '4K 视频归档',
    desc: '利用 UDP 高吞吐特性，快速将占用大量空间的视频文件转移到大容量硬盘。',
    icon: Video,
  },
  {
    title: '旧设备数据迁移',
    desc: '更换新手机时，将旧手机全部媒体数据可靠迁移至家庭数据中心。',
    icon: HardDrive,
  },
  {
    title: '下载（Server → Client）',
    desc: 'Client 主动发起，Server 向 Client 推送数据，控制面与可靠性机制与上传一致。',
    icon: Download,
  },
];

const UseCases = () => {
  return (
    <div className="mt-16">
      <h2 className="text-2xl font-bold text-gray-900 mb-8">典型应用场景</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {cases.map((c) => (
          <div key={c.title} className="bg-gray-50 rounded-xl p-6 hover:bg-white hover:shadow-md transition-all duration-300">
            <c.icon className="w-8 h-8 text-gray-400 mb-4" />
            <h3 className="text-lg font-bold text-gray-900">{c.title}</h3>
            <p className="mt-2 text-sm text-gray-500">{c.desc}</p>
          </div>
        ))}
      </div>
    </div>
  );
};

export default UseCases;
