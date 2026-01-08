import React from 'react';
import CodeBlock from '../ui/CodeBlock';

const MessageDefinitions = () => {
  const uploadStartTopic = `jiayi/fsp/client/{client_id}/upload_start`;
  const uploadStartPayload = `{
  "session_id": "uint64",
  "file_id_hash": "uint64",
  "file_size": "uint64",
  "chunk_size": "uint32"
}`;

  const uploadAcceptTopic = `jiayi/fsp/server/{client_id}/upload_accept`;
  const uploadAcceptPayload = `{
  "session_id": "uint64",
  "chunk_count": "uint64",
  "window": "uint32"
}`;

  const udpReadyTopic = `jiayi/fsp/server/{client_id}/udp_ready`;
  const udpReadyPayload = `{
  "session_id": "uint64"
}`;

  const missingChunksTopic = `jiayi/fsp/server/{client_id}/missing_chunks`;
  const missingChunksPayload = `{
  "session_id": "uint64",
  "missing_chunks": [3,7,9]
}`;

  const resumeTopic = `jiayi/fsp/client/{client_id}/resume_session`;
  const resumePayload = `{
  "session_id": "uint64"
}`;

  const downloadStartTopic = `jiayi/fsp/client/{client_id}/download_start`;
  const downloadStartPayload = `{
  "file_id": "sha256:xxx",
  "target": "original",
  "range": null
}`;

  const downloadAcceptTopic = `jiayi/fsp/server/{client_id}/download_accept`;
  const downloadAcceptPayload = `{
  "session_id": "uint64",
  "chunk_size": "uint32",
  "window": "uint32",
  "total_chunks": "uint64"
}`;

  return (
    <div className="space-y-8">
      <h2 className="text-2xl font-bold text-gray-900">MQTT Topic 与载荷示例</h2>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="space-y-2">
          <h3 className="font-semibold text-gray-700">1. Upload Start</h3>
          <CodeBlock code={uploadStartTopic} language="text" />
          <CodeBlock code={uploadStartPayload} language="json" />
        </div>
        <div className="space-y-2">
          <h3 className="font-semibold text-gray-700">2. Upload Accept</h3>
          <CodeBlock code={uploadAcceptTopic} language="text" />
          <CodeBlock code={uploadAcceptPayload} language="json" />
        </div>
        <div className="space-y-2">
          <h3 className="font-semibold text-gray-700">3. UDP Ready</h3>
          <CodeBlock code={udpReadyTopic} language="text" />
          <CodeBlock code={udpReadyPayload} language="json" />
        </div>
        <div className="space-y-2">
          <h3 className="font-semibold text-gray-700">4. Missing Chunks</h3>
          <CodeBlock code={missingChunksTopic} language="text" />
          <CodeBlock code={missingChunksPayload} language="json" />
        </div>
        <div className="space-y-2">
          <h3 className="font-semibold text-gray-700">5. Resume Session</h3>
          <CodeBlock code={resumeTopic} language="text" />
          <CodeBlock code={resumePayload} language="json" />
        </div>
        <div className="space-y-2">
          <h3 className="font-semibold text-gray-700">6. Download Start（扩展）</h3>
          <CodeBlock code={downloadStartTopic} language="text" />
          <CodeBlock code={downloadStartPayload} language="json" />
        </div>
        <div className="space-y-2">
          <h3 className="font-semibold text-gray-700">7. Download Accept（扩展）</h3>
          <CodeBlock code={downloadAcceptTopic} language="text" />
          <CodeBlock code={downloadAcceptPayload} language="json" />
        </div>
      </div>
    </div>
  );
};

export default MessageDefinitions;
