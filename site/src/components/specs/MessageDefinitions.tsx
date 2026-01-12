import React from 'react';
import CodeBlock from '../ui/CodeBlock';

const MessageDefinitions = () => {
  const uploadStartTopic = `jiayi/fsp/client/{client_id}/upload_start`;
  const uploadStartPayload = `{
  "session_id": "uint64",
  "file_id": "string",
  "file_name": "string",
  "size": "uint64",
  "mtime": "uint64",
  "chunk_size": "uint32"
}`;

  const uploadAcceptTopic = `jiayi/fsp/server/{client_id}/upload_accept`;
  const uploadAcceptPayload = `{
  "session_id": "uint64",
  "chunk_count": "uint32",
  "window": "uint32",
  "udp_port": "uint16"
}`;

  const udpReadyTopic = `jiayi/fsp/server/{client_id}/udp_ready`;
  const udpReadyPayload = `{
  "session_id": "uint64"
}`;

  const missingChunksTopic = `jiayi/fsp/server/{client_id}/missing_chunks`;
  const missingChunksPayload = `{
  "session_id": "uint64",
  "missing_chunks": ["uint32", "uint32"]
}`;

  const resumeTopic = `jiayi/fsp/client/{client_id}/resume_session`;
  const resumePayload = `{
  "session_id": "uint64"
}`;

  const uploadCompleteTopic = `jiayi/fsp/server/{client_id}/upload_complete`;
  const uploadCompletePayload = `{
  "session_id": "uint64",
  "status": "ok"
}`;

  const sessionAbortTopic = `jiayi/fsp/server/{client_id}/session_abort`;
  const sessionAbortPayload = `{
  "session_id": "uint64",
  "reason": "string"
}`;

  const downloadStartTopic = `jiayi/fsp/client/{client_id}/download_start`;
  const downloadStartPayload = `{
  "file_id": "string"
}`;

  const downloadAcceptTopic = `jiayi/fsp/server/{client_id}/download_accept`;
  const downloadAcceptPayload = `{
  "session_id": "uint64",
  "chunk_count": "uint32",
  "window": "uint32",
  "tcp_port": "uint16"
}`;

  const downloadCompleteTopic = `jiayi/fsp/server/{client_id}/download_complete`;
  const downloadCompletePayload = `{
  "session_id": "uint64"
}`;

  return (
    <div className="space-y-8">
      <h2 className="text-2xl font-bold text-gray-900">MQTT Topic 与载荷示例</h2>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">上传（Upload）</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">1. Upload Start</h4>
            <p className="text-sm text-gray-500">Client → Server：发起上传会话</p>
            <CodeBlock code={uploadStartTopic} language="text" />
            <CodeBlock code={uploadStartPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">2. Upload Accept</h4>
            <p className="text-sm text-gray-500">Server → Client：接受上传请求</p>
            <CodeBlock code={uploadAcceptTopic} language="text" />
            <CodeBlock code={uploadAcceptPayload} language="json" />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">传输控制</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">3. UDP Ready</h4>
            <p className="text-sm text-gray-500">Server → Client：UDP 可用性确认</p>
            <CodeBlock code={udpReadyTopic} language="text" />
            <CodeBlock code={udpReadyPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">4. Missing Chunks</h4>
            <p className="text-sm text-gray-500">Server → Client：缺失数据块列表</p>
            <CodeBlock code={missingChunksTopic} language="text" />
            <CodeBlock code={missingChunksPayload} language="json" />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">会话完成与终止</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">5. Upload Complete</h4>
            <p className="text-sm text-gray-500">Server → Client：上传完成通知</p>
            <CodeBlock code={uploadCompleteTopic} language="text" />
            <CodeBlock code={uploadCompletePayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">6. Session Abort</h4>
            <p className="text-sm text-gray-500">Server → Client：会话终止</p>
            <CodeBlock code={sessionAbortTopic} language="text" />
            <CodeBlock code={sessionAbortPayload} language="json" />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">会话恢复</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">7. Resume Session</h4>
            <p className="text-sm text-gray-500">Client → Server：请求恢复中断的会话</p>
            <CodeBlock code={resumeTopic} language="text" />
            <CodeBlock code={resumePayload} language="json" />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">下载（Download）</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">8. Download Start</h4>
            <p className="text-sm text-gray-500">Client → Server：请求下载文件</p>
            <CodeBlock code={downloadStartTopic} language="text" />
            <CodeBlock code={downloadStartPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">9. Download Accept</h4>
            <p className="text-sm text-gray-500">Server → Client：接受下载请求</p>
            <CodeBlock code={downloadAcceptTopic} language="text" />
            <CodeBlock code={downloadAcceptPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">10. Download Complete</h4>
            <p className="text-sm text-gray-500">Server → Client：下载完成通知（可选）</p>
            <CodeBlock code={downloadCompleteTopic} language="text" />
            <CodeBlock code={downloadCompletePayload} language="json" />
          </div>
        </div>
      </div>

      <div className="bg-yellow-50 border-l-4 border-yellow-400 p-4 rounded-r-lg">
        <p className="font-bold text-yellow-800 mb-2">重要说明</p>
        <ul className="text-sm text-yellow-700 space-y-1 list-disc list-inside">
          <li>所有消息必须携带 <code className="bg-yellow-100 px-1 rounded">session_id</code> 字段</li>
          <li>未识别的消息类型必须忽略，保持幂等性</li>
          <li>Client 仅发布 <code className="bg-yellow-100 px-1 rounded">client/{"{client_id}"}/#</code>，订阅 <code className="bg-yellow-100 px-1 rounded">server/{"{client_id}"}/#</code></li>
          <li>Server 订阅 <code className="bg-yellow-100 px-1 rounded">client/+</code>，发布 <code className="bg-yellow-100 px-1 rounded">server/{"{client_id}"}/#</code></li>
        </ul>
      </div>
    </div>
  );
};

export default MessageDefinitions;
