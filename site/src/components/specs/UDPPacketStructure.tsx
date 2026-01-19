import React from 'react';

const UDPPacketStructure = () => {
    return (
        <div className="mt-12 space-y-8">
            <div>
                <h2 className="text-2xl font-bold text-gray-900 mb-6">UDP 数据包结构 (Packet Format)</h2>
                <div className="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden mb-6">
                    <div className="grid grid-cols-4 gap-0 text-center font-mono text-xs border-b border-gray-200 bg-gray-50">
                        <div className="col-span-8 p-2 border-b border-gray-200 text-sm font-bold text-gray-500">32 Bits Width</div>
                        <div className="p-1 border-r border-gray-200">0-7</div>
                        <div className="p-1 border-r border-gray-200">8-15</div>
                        <div className="p-1 border-r border-gray-200">16-23</div>
                        <div className="p-1">24-31</div>

                    </div>

                    <div className="font-mono text-sm">
                        <div className="p-4 bg-blue-50 text-blue-800 text-center border-b border-gray-200">
                            session_id (32 bit)
                        </div>
                        <div className="p-4 bg-green-50 text-blue-800 text-center border-b border-gray-200">
                            chunk_id (32 bit)
                        </div>
                        <div className="grid grid-cols-2 divide-x divide-gray-200 border-b border-gray-200">
                            <div className="col-span-1 p-4 bg-yellow-50 text-yellow-800 text-center">payload_len (16)</div>
                            <div className="col-span-1 p-4 bg-gray-50 text-gray-800 text-center">flags (16)</div>
                        </div>
                        <div className="p-8 bg-purple-50 text-purple-800 text-center border-b border-gray-200">
                            payload (Variable Length: N bytes)
                        </div>
                        <div className="p-4 bg-red-50 text-red-800 text-center">
                            crc32 (32 bit)
                        </div>
                    </div>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                <div>
                    <h3 className="text-lg font-bold text-gray-900 mb-4">字段定义</h3>
                    <div className="overflow-hidden border border-gray-200 rounded-lg">
                        <table className="min-w-full divide-y divide-gray-200">
                            <thead className="bg-gray-50">
                                <tr>
                                    <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">字段</th>
                                    <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">位宽</th>
                                    <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">偏移 (Byte)</th>
                                    <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">说明</th>
                                </tr>
                            </thead>
                            <tbody className="bg-white divide-y divide-gray-200 text-sm">
                                <tr>
                                    <td className="px-4 py-2 font-mono text-blue-600">session_id</td>
                                    <td className="px-4 py-2 text-gray-500">32</td>
                                    <td className="px-4 py-2 text-gray-500">0</td>
                                    <td className="px-4 py-2 text-gray-700">会话唯一标识，与 MQTT Session 一致</td>
                                </tr>
                                <tr>
                                    <td className="px-4 py-2 font-mono text-green-600">chunk_id</td>
                                    <td className="px-4 py-2 text-gray-500">32</td>
                                    <td className="px-4 py-2 text-gray-500">4</td>
                                    <td className="px-4 py-2 text-gray-700">数据块索引，从 0 开始</td>
                                </tr>
                                <tr>
                                    <td className="px-4 py-2 font-mono text-yellow-600">payload_len</td>
                                    <td className="px-4 py-2 text-gray-500">16</td>
                                    <td className="px-4 py-2 text-gray-500">8</td>
                                    <td className="px-4 py-2 text-gray-700">Payload 实际字节数</td>
                                </tr>
                                <tr>
                                    <td className="px-4 py-2 font-mono text-gray-600">flags</td>
                                    <td className="px-4 py-2 text-gray-500">16</td>
                                    <td className="px-4 py-2 text-gray-500">10</td>
                                    <td className="px-4 py-2 text-gray-700">控制位掩码 (见右侧)</td>
                                </tr>
                                <tr>
                                    <td className="px-4 py-2 font-mono text-purple-600">payload</td>
                                    <td className="px-4 py-2 text-gray-500">N</td>
                                    <td className="px-4 py-2 text-gray-500">12</td>
                                    <td className="px-4 py-2 text-gray-700">原始文件数据</td>
                                </tr>
                                <tr>
                                    <td className="px-4 py-2 font-mono text-red-600">crc32</td>
                                    <td className="px-4 py-2 text-gray-500">32</td>
                                    <td className="px-4 py-2 text-gray-500">12+N</td>
                                    <td className="px-4 py-2 text-gray-700">完整性校验 (Header + Payload)</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>

                <div className="space-y-6">
                    <div>
                        <h3 className="text-lg font-bold text-gray-900 mb-4">Flags 定义 (16 bits)</h3>
                        <div className="bg-gray-50 p-4 rounded-lg border border-gray-200 text-sm font-mono space-y-2">
                            <div className="flex justify-between items-center">
                                <span className="text-gray-500">Bit 0</span>
                                <span className="font-bold text-gray-900">DATA</span>
                                <span className="text-gray-600">普通数据包</span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-gray-500">Bit 1</span>
                                <span className="font-bold text-gray-900">RESEND</span>
                                <span className="text-gray-600">重传数据包</span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-gray-500">Bit 2</span>
                                <span className="font-bold text-gray-900">LAST_CHUNK</span>
                                <span className="text-gray-600">最后一个 Chunk</span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-gray-500">Bit 3</span>
                                <span className="font-bold text-gray-900">PROBE</span>
                                <span className="text-gray-600">探测 / 测试包</span>
                            </div>
                            <div className="flex justify-between items-center pt-2 border-t border-gray-200 text-gray-400">
                                <span>Bit 4-15</span>
                                <span>RESERVED</span>
                                <span>保留位 (Must be 0)</span>
                            </div>
                        </div>
                    </div>

                    <div>
                        <h3 className="text-lg font-bold text-gray-900 mb-2">错误处理规范 (Error Handling)</h3>
                        <ul className="list-disc list-inside text-sm text-gray-600 space-y-1">
                            <li>所有多字节字段必须使用 <strong>网络字节序 (Big Endian)</strong></li>
                            <li>CRC32 校验失败的包 <strong>必须丢弃 (MUST discard)</strong></li>
                            <li>Session ID 不匹配的包 <strong>必须丢弃 (MUST discard)</strong></li>
                            <li>File ID Hash 不匹配的包 <strong>必须丢弃并记录 (Discard & Log)</strong></li>
                            <li>Payload Length 非法的包 <strong>必须丢弃 (MUST discard)</strong></li>
                        </ul>
                    </div>
                </div>
            </div>

            <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
                <h3 className="text-lg font-bold text-gray-900 mb-3">尺寸限制 (Size Limits)</h3>
                <div className="space-y-2 text-sm text-gray-700">
                    <div className="flex items-center gap-2">
                        <span className="w-2 h-2 rounded-full bg-blue-500"></span>
                        <span>chunk_size: 由协商决定（建议 <span className="font-mono font-bold text-gray-900">1200</span> 字节）</span>
                    </div>
                    <div className="flex items-center gap-2">
                        <span className="w-2 h-2 rounded-full bg-green-500"></span>
                        <span>payload_len ≤ chunk_size</span>
                    </div>
                    <div className="flex items-center gap-2">
                        <span className="w-2 h-2 rounded-full bg-purple-500"></span>
                        <span>总包大小 = <span className="font-mono">12</span> (固定字段) + <span className="font-mono">4</span> (CRC32) + <span className="font-mono">payload_len</span></span>
                    </div>
                     <div className="flex items-center gap-2">
                        <span className="w-2 h-2 rounded-full bg-yellow-500"></span>
                        <span>数据大小 = （建议 <span className="font-mono font-bold text-gray-900">1200</span> 字节）- <span className="font-mono">12</span> (固定字段) - <span className="font-mono">4</span> (CRC32) = <span className="font-mono font-bold text-gray-900">1184</span> 字节</span>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default UDPPacketStructure;
