import React from 'react';
import { CheckCircle, AlertOctagon, RotateCcw, Activity, Shield, Database } from 'lucide-react';

const mechanisms = [
  {
    problem: '丢包',
    solution: 'Missing Chunks + Resend',
    description: '接收方每 50ms 通过 MQTT 下发 missing_chunks（mode=report），列出自己尚未收到的 Chunk ID，发送方针对性重传。空列表也必须下发——它是「可以继续推进」的裁决。',
    icon: AlertOctagon,
  },
  {
    problem: '乱序',
    solution: 'Chunk ID 重排',
    description: '每个 UDP 包携带 Chunk ID，Server 根据 ID 将数据写入文件的正确偏移位置，无需按序接收。',
    icon: RotateCcw,
  },
  {
    problem: '重复',
    solution: 'Bitmap / Received Set',
    description: '接收方维护已接收 Chunk 的位图集合，重复收到的包会被直接丢弃，不影响文件正确性；位图同时用于计算缺失列表与恢复续传。',
    icon: CheckCircle,
  },
  {
    problem: '中断',
    solution: 'resume_session 恢复',
    description: 'Session ID 关联文件状态。中断后通过 resume_session 恢复，resume_ack 返回 next_chunk_id（第一个缺失块）与数据面端口；低于最大已收 id 的空洞块由周期裁决持续指认补发。',
    icon: Activity,
  },
  {
    problem: 'UDP不可用',
    solution: 'TCP 降级 (Fallback)',
    description: '当 UDP 探测失败（PROBE_ACK 超时约 2s）或异常时，协议可降级为 TCP 传输，保持 chunk 与 bitmap 语义不变。TCP 线路格式待定义。',
    icon: Shield,
  },
  {
    problem: '内存溢出',
    solution: 'Random Access Write + Bitmap',
    description: '强制要求接收端不缓存大文件，而是按 offset = chunk_id × chunk_size 直接写入磁盘；仅维护位图状态，内存占用与文件大小无关。',
    icon: Database,
  },
];

const ReliabilityMechanisms = () => {
  return (
    <div className="mt-16">
      <h2 className="text-2xl font-bold text-gray-900 mb-8">可靠性保障机制</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {mechanisms.map((item) => (
          <div key={item.problem} className="flex items-start p-6 bg-white rounded-xl border border-gray-200 shadow-sm">
            <div className="flex-shrink-0">
              <div className="p-2 bg-primary-50 rounded-lg text-primary-600">
                <item.icon className="w-6 h-6" />
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-lg font-medium text-gray-900">
                应对：{item.problem}
              </h3>
              <p className="text-sm font-semibold text-primary-600 mt-1">
                方案：{item.solution}
              </p>
              <p className="mt-2 text-sm text-gray-500">
                {item.description}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default ReliabilityMechanisms;
