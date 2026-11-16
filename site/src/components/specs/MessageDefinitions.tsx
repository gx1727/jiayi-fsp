import React from 'react';
import CodeBlock from '../ui/CodeBlock';

const MessageDefinitions = () => {
  const loginTopic = `/jiayi/fsp/client/{client_id}/login`;
  const loginPayload = `{
  "type": "login",
  "device_id": "string",
  "username": "string|null"
}`;

  const loginAckTopic = `/jiayi/fsp/server/{client_id}/login_ack`;
  const loginAckPayload = `{
  "type": "login_ack",
  "result": "success|fail",
  "message": "string|null",
  "user": {
    "user_id": "i64",
    "username": "string",
    "device_id": "string",
    "status": "string",
    "last_login_time": "i64",
    "created_time": "i64"
  }
}`;

  const statusTopic = `/jiayi/fsp/{role}/{client_id}/status`;
  const statusPayload = `{
  "type": "status",
  "device_id": "string",
  "status": "online|offline"
}`;

  const uploadStartTopic = `/jiayi/fsp/client/{client_id}/upload_start`;
  const uploadStartPayload = `{
  "type": "upload_start",
  "session_id": "string|null",
  "file_hash": "string",
  "file_name": "string",
  "size": "u64",
  "mtime": "u64",
  "chunk_size": "u32"
}`;

  const uploadAcceptTopic = `/jiayi/fsp/server/{client_id}/upload_accept`;
  const uploadAcceptPayload = `{
  "type": "upload_accept",
  "file_hash": "string",
  "session_id": "string",
  "chunk_count": "u32",
  "window": "u32",
  "udp_port": "u16",
  "chunk_size": "u32",
  "file_exists": "bool"
}`;

  const udpReadyTopic = `/jiayi/fsp/server/{client_id}/udp_ready`;
  const udpReadyPayload = `{
  "type": "udp_ready",
  "session_id": "string"
}`;

  const missingChunksTopic = `/jiayi/fsp/{role}/{client_id}/missing_chunks`;
  const missingChunksPayload = `{
  "type": "missing_chunks",
  "session_id": "string",
  "chunks": ["u32"]
}`;

  const progressUpdateTopic = `/jiayi/fsp/{role}/{client_id}/progress_update`;
  const progressUpdatePayload = `{
  "type": "progress_update",
  "session_id": "string",
  "received": "u64",
  "total": "u64"
}`;

  const sessionPauseTopic = `/jiayi/fsp/{role}/{client_id}/session_pause`;
  const sessionPausePayload = `{
  "type": "session_pause",
  "session_id": "string"
}`;

  const resumeTopic = `/jiayi/fsp/client/{client_id}/resume_session`;
  const resumePayload = `{
  "type": "resume_session",
  "session_id": "string"
}`;

  const resumeAckTopic = `/jiayi/fsp/server/{client_id}/resume_ack`;
  const resumeAckPayload = `{
  "type": "resume_ack",
  "session_id": "string",
  "next_chunk_id": "u32",
  "udp_port": "u16"
}`;

  const uploadCompleteTopic = `/jiayi/fsp/server/{client_id}/upload_complete`;
  const uploadCompletePayload = `{
  "type": "upload_complete",
  "session_id": "string",
  "result": "string",
  "file_hash": "string|null"
}`;

  const sessionAbortTopic = `/jiayi/fsp/{role}/{client_id}/session_abort`;
  const sessionAbortPayload = `{
  "type": "session_abort",
  "session_id": "string",
  "reason": "string"
}`;

  const downloadStartTopic = `/jiayi/fsp/client/{client_id}/download_start`;
  const downloadStartPayload = `{
  "type": "download_start",
  "session_id": "string",
  "file_hash": "string",
  "file_name": "string",
  "size": "u64",
  "chunk_size": "u32"
}`;

  const downloadAcceptTopic = `/jiayi/fsp/server/{client_id}/download_accept`;
  const downloadAcceptPayload = `{
  "type": "download_accept",
  "session_id": "string",
  "chunk_count": "u32",
  "window": "u32",
  "tcp_port": "u16"
}`;

  const downloadCompleteTopic = `/jiayi/fsp/{role}/{client_id}/download_complete`;
  const downloadCompletePayload = `{
  "type": "download_complete",
  "session_id": "string",
  "result": "string"
}`;

  const assetRecallTopic = `/jiayi/fsp/client/{client_id}/asset_recall`;
  const assetRecallPayload = `{
  "type": "asset_recall",
  "file_hash": "string",
  "asset_id": "string|null"
}`;

  const assetRecallCompleteTopic = `/jiayi/fsp/server/{client_id}/asset_recall_complete`;
  const assetRecallCompletePayload = `{
  "type": "asset_recall_complete",
  "file_hash": "string",
  "asset_id": "string|null",
  "deleted": "bool",
  "result": "string"
}`;

  return (
    <div className="space-y-8">
      <h2 className="text-2xl font-bold text-gray-900">MQTT Topic 与载荷示例</h2>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">认证类 (Authentication)</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Login</h4>
            <p className="text-sm text-gray-500">Client → Server：设备登录认证</p>
            <CodeBlock code={loginTopic} language="text" />
            <CodeBlock code={loginPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Login Ack</h4>
            <p className="text-sm text-gray-500">Server → Client：登录响应</p>
            <CodeBlock code={loginAckTopic} language="text" />
            <CodeBlock code={loginAckPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Status</h4>
            <p className="text-sm text-gray-500">双向：设备在线状态报告</p>
            <CodeBlock code={statusTopic} language="text" />
            <CodeBlock code={statusPayload} language="json" />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">上传（Upload）</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Upload Start</h4>
            <p className="text-sm text-gray-500">Client → Server：发起上传会话</p>
            <CodeBlock code={uploadStartTopic} language="text" />
            <CodeBlock code={uploadStartPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Upload Accept</h4>
            <p className="text-sm text-gray-500">Server → Client：接受上传请求</p>
            <CodeBlock code={uploadAcceptTopic} language="text" />
            <CodeBlock code={uploadAcceptPayload} language="json" />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">传输控制 (Transfer Control)</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">UDP Ready</h4>
            <p className="text-sm text-gray-500">Server → Client：UDP 可用性确认</p>
            <CodeBlock code={udpReadyTopic} language="text" />
            <CodeBlock code={udpReadyPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Missing Chunks</h4>
            <p className="text-sm text-gray-500">双向：缺失数据块列表</p>
            <CodeBlock code={missingChunksTopic} language="text" />
            <CodeBlock code={missingChunksPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Progress Update</h4>
            <p className="text-sm text-gray-500">双向：传输进度更新</p>
            <CodeBlock code={progressUpdateTopic} language="text" />
            <CodeBlock code={progressUpdatePayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Session Pause</h4>
            <p className="text-sm text-gray-500">双向：暂停会话</p>
            <CodeBlock code={sessionPauseTopic} language="text" />
            <CodeBlock code={sessionPausePayload} language="json" />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">会话管理 (Session Management)</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Resume Session</h4>
            <p className="text-sm text-gray-500">Client → Server：请求恢复中断的会话</p>
            <CodeBlock code={resumeTopic} language="text" />
            <CodeBlock code={resumePayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Resume Ack</h4>
            <p className="text-sm text-gray-500">Server → Client：恢复确认</p>
            <CodeBlock code={resumeAckTopic} language="text" />
            <CodeBlock code={resumeAckPayload} language="json" />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">会话完成与终止 (Completion & Termination)</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Upload Complete</h4>
            <p className="text-sm text-gray-500">Server → Client：上传完成通知</p>
            <CodeBlock code={uploadCompleteTopic} language="text" />
            <CodeBlock code={uploadCompletePayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Session Abort</h4>
            <p className="text-sm text-gray-500">双向：会话终止</p>
            <CodeBlock code={sessionAbortTopic} language="text" />
            <CodeBlock code={sessionAbortPayload} language="json" />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">下载（Download）</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Download Start</h4>
            <p className="text-sm text-gray-500">Client → Server：请求下载文件</p>
            <CodeBlock code={downloadStartTopic} language="text" />
            <CodeBlock code={downloadStartPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Download Accept</h4>
            <p className="text-sm text-gray-500">Server → Client：接受下载请求</p>
            <CodeBlock code={downloadAcceptTopic} language="text" />
            <CodeBlock code={downloadAcceptPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Download Complete</h4>
            <p className="text-sm text-gray-500">双向：下载完成通知</p>
            <CodeBlock code={downloadCompleteTopic} language="text" />
            <CodeBlock code={downloadCompletePayload} language="json" />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-bold text-gray-900 mb-4">资产撤回 (Asset Recall)</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Asset Recall</h4>
            <p className="text-sm text-gray-500">Client → Server：请求撤回（删除）已上传的素材记录</p>
            <CodeBlock code={assetRecallTopic} language="text" />
            <CodeBlock code={assetRecallPayload} language="json" />
          </div>
          <div className="space-y-2">
            <h4 className="font-semibold text-gray-700">Asset Recall Complete</h4>
            <p className="text-sm text-gray-500">Server → Client：撤回处理结果回执</p>
            <CodeBlock code={assetRecallCompleteTopic} language="text" />
            <CodeBlock code={assetRecallCompletePayload} language="json" />
          </div>
        </div>
      </div>

      <div className="bg-yellow-50 border-l-4 border-yellow-400 p-4 rounded-r-lg">
        <p className="font-bold text-yellow-800 mb-2">重要说明</p>
        <ul className="text-sm text-yellow-700 space-y-1 list-disc list-inside">
          <li>所有消息必须携带 <code className="bg-yellow-100 px-1 rounded">type</code> 字段标识消息类型</li>
          <li><code className="bg-yellow-100 px-1 rounded">session_id</code> 为 u32 类型，在 JSON 中以字符串形式传输，由 Server 生成</li>
          <li><code className="bg-yellow-100 px-1 rounded">file_hash</code> 一律为 Blake3 哈希的 64 字符十六进制串</li>
          <li>Client 发送 upload_start 时 session_id 可为空</li>
          <li>未识别的消息类型必须忽略，保持幂等性</li>
          <li>Client 仅发布 <code className="bg-yellow-100 px-1 rounded">{'/jiayi/fsp/client/{client_id}/#'}</code>，订阅 <code className="bg-yellow-100 px-1 rounded">{'/jiayi/fsp/server/{client_id}/#'}</code></li>
          <li>Server 订阅 <code className="bg-yellow-100 px-1 rounded">/jiayi/fsp/client/+/#</code>，发布 <code className="bg-yellow-100 px-1 rounded">{'/jiayi/fsp/server/{client_id}/#'}</code>（Topic 固定 5 层）</li>
        </ul>
      </div>
    </div>
  );
};

export default MessageDefinitions;
