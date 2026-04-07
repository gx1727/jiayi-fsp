import { useState } from "react";
import { motion } from "framer-motion";
import { 
  Cloud, 
  Upload, 
  Settings, 
  Filter, 
  CheckCircle, 
  Lock, 
  Smartphone, 
  QrCode,
  Image,
  Video,
  Layers,
  ArrowRight,
  Menu,
  X
} from "lucide-react";

const Section = ({ id, title, children }: { id?: string; title: string; children: React.ReactNode }) => (
  <section id={id} className="max-w-5xl mx-auto px-6 py-12">
    <h2 className="text-2xl md:text-3xl font-semibold text-gray-900 mb-6">{title}</h2>
    {children}
  </section>
);

const FeatureCard = ({ icon: Icon, title, description }: { icon: any; title: string; description: string }) => (
  <motion.div
    initial={{ opacity: 0, y: 10 }}
    whileInView={{ opacity: 1, y: 0 }}
    viewport={{ once: true }}
    className="p-6 rounded-xl bg-white shadow-sm border border-gray-100"
  >
    <div className="h-12 w-12 rounded-lg bg-gradient-to-r from-brand-blue/15 to-brand-purple/15 flex items-center justify-center mb-4">
      <Icon className="h-6 w-6 text-brand-blue" />
    </div>
    <h3 className="font-semibold text-gray-900">{title}</h3>
    <p className="mt-2 text-gray-600 text-sm">{description}</p>
  </motion.div>
);

const StatusBadge = ({ icon: Icon, color, label }: { icon: any; color: string; label: string }) => (
  <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-gray-50 border border-gray-200">
    <Icon className={`h-4 w-4 ${color}`} />
    <span className="text-sm text-gray-700">{label}</span>
  </div>
);

export default function AppGuide() {
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <div className="min-h-screen bg-gradient-to-b from-gray-50 to-white">
      <header className="sticky top-0 z-40 bg-white/80 backdrop-blur-md border-b border-gray-200">
        <div className="max-w-5xl mx-auto px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <img src="/logo.png" alt="家忆 · JIAYI" className="h-8 w-8 rounded" />
            <span className="text-xl font-semibold text-gray-900">家忆 · JIAYI</span>
          </div>
          <nav className="hidden md:flex items-center gap-4 text-sm text-gray-700">
            <a href="/" className="hover:text-brand-blue">首页</a>
            <a href="/app-guide" className="text-brand-blue font-medium">使用手册</a>
          </nav>
          <button
            className="md:hidden p-2 text-gray-700"
            onClick={() => setMenuOpen(!menuOpen)}
            aria-label="Toggle menu"
          >
            {menuOpen ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
          </button>
        </div>
        {menuOpen && (
          <nav className="md:hidden px-6 pb-4 flex flex-col gap-3 text-sm text-gray-700 border-t border-gray-200">
            <a href="/" onClick={() => setMenuOpen(false)} className="py-1">首页</a>
            <a href="/app-guide" onClick={() => setMenuOpen(false)} className="py-1 text-brand-blue font-medium">使用手册</a>
          </nav>
        )}
      </header>

      {/* Hero */}
      <section className="max-w-5xl mx-auto px-6 pt-16 pb-12">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
        >
          <h1 className="text-4xl md:text-5xl font-bold text-gray-900 tracking-tight">
            家忆（JIAYI）APP 使用手册
          </h1>
          <p className="mt-4 text-xl text-gray-600">
            欢迎使用家忆（JIAYI）APP！本手册将帮助您快速了解界面的各项功能并掌握基本的操作流程。
          </p>
        </motion.div>
      </section>

      {/* 界面概览 */}
      <Section id="overview" title="📱 界面概览与功能说明">
        <div className="grid md:grid-cols-2 gap-6">
          <FeatureCard
            icon={Cloud}
            title="1. 首页"
            description="首页是管理和查看您所有照片和视频的主界面。顶部显示应用名称、服务端连接状态、传输页和设置页按钮。筛选按钮和一键按钮帮助您快速管理内容。"
          />
          <FeatureCard
            icon={Filter}
            title="2. 首页筛选"
            description="点击筛选按钮，可从底部弹出筛选菜单。可选择媒体类型（全部、照片、视频）和同步状态（全部、未同步、已同步）。"
          />
          <FeatureCard
            icon={Image}
            title="3. 详细页"
            description="在首页点击任意图片即可进入详细页查看大图。底部包含同步和锁定两个主要操作按钮。"
          />
          <FeatureCard
            icon={Layers}
            title="4. 传输页"
            description="查看正在上传或已完成的任务列表。可显示全部传输记录，包括文件名、上传进度条和完成状态。"
          />
          <FeatureCard
            icon={Settings}
            title="5. 设置页"
            description="配置服务端的 IP 和端口。支持扫码功能，可扫描服务端二维码自动填充配置。提供重置工具。"
          />
          <FeatureCard
            icon={Smartphone}
            title="6. 服务端"
            description="在电脑或服务器上运行的后台管理系统。显示终端数、文件总数和任务状态。提供二维码供手机 APP 扫码连接。"
          />
        </div>
      </Section>

      {/* 状态说明 */}
      <Section id="status" title="📊 状态说明">
        <div className="bg-white rounded-2xl shadow-sm border border-gray-200 p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">照片/视频状态角标</h3>
          <div className="grid sm:grid-cols-2 md:grid-cols-3 gap-4">
            <StatusBadge
              icon={Cloud}
              color="text-gray-400"
              label="未同步（云朵图标）"
            />
            <StatusBadge
              icon={CheckCircle}
              color="text-green-500"
              label="已同步（绿色对勾）"
            />
            <StatusBadge
              icon={Lock}
              color="text-orange-500"
              label="已锁定（橙色锁）"
            />
            <StatusBadge
              icon={Upload}
              color="text-blue-500"
              label="传输中（刷新图标）"
            />
            <StatusBadge
              icon={Cloud}
              color="text-green-500"
              label="服务端已连接（绿色云朵）"
            />
            <StatusBadge
              icon={Cloud}
              color="text-gray-400"
              label="服务端未连接（灰色云朵）"
            />
          </div>
        </div>
      </Section>

      {/* 使用说明 */}
      <Section id="guide" title="🚀 使用说明">
        {/* 首次连接 */}
        <div className="mb-8">
          <h3 className="text-xl font-semibold text-gray-900 mb-4">1. 首次打开与连接配置</h3>
          <div className="bg-blue-50 border-l-4 border-blue-400 p-6 rounded-r-xl">
            <ol className="space-y-3 text-gray-700">
              <li className="flex items-start gap-3">
                <span className="flex-shrink-0 h-6 w-6 rounded-full bg-blue-500 text-white text-sm flex items-center justify-center">1</span>
                <span>第一次打开 APP 时，首先查看首页顶部的<strong>服务端连接状态</strong>图标</span>
              </li>
              <li className="flex items-start gap-3">
                <span className="flex-shrink-0 h-6 w-6 rounded-full bg-blue-500 text-white text-sm flex items-center justify-center">2</span>
                <span>如果图标是<strong>灰色</strong>，说明手机尚未连接到服务端</span>
              </li>
              <li className="flex items-start gap-3">
                <span className="flex-shrink-0 h-6 w-6 rounded-full bg-blue-500 text-white text-sm flex items-center justify-center">3</span>
                <span>APP 具备<strong>局域网自动发现功能</strong>，通常会自动寻找并连接同局域网内的服务端</span>
              </li>
              <li className="flex items-start gap-3">
                <span className="flex-shrink-0 h-6 w-6 rounded-full bg-blue-500 text-white text-sm flex items-center justify-center">4</span>
                <span>如果未能自动连接，请点击该灰色图标尝试<strong>手动连接</strong></span>
              </li>
              <li className="flex items-start gap-3">
                <span className="flex-shrink-0 h-6 w-6 rounded-full bg-blue-500 text-white text-sm flex items-center justify-center">5</span>
                <span>如果仍然连接失败，请进入<strong>设置页</strong>，点击<strong>服务器配置</strong>旁的<strong>"扫码"</strong>按钮</span>
              </li>
              <li className="flex items-start gap-3">
                <span className="flex-shrink-0 h-6 w-6 rounded-full bg-blue-500 text-white text-sm flex items-center justify-center">6</span>
                <span>将摄像头对准电脑端打开的<strong>服务端界面中的二维码</strong>进行扫描，APP 将自动填写正确的 IP 和端口并完成连接</span>
              </li>
            </ol>
          </div>
        </div>

        {/* 上传操作 */}
        <div>
          <h3 className="text-xl font-semibold text-gray-900 mb-4">2. 照片/视频上传操作</h3>
          <p className="text-gray-600 mb-6">
            APP 提供三种便捷的上传方式，以满足不同场景的需求：
          </p>
          
          <div className="grid md:grid-cols-3 gap-6">
            {/* 方式一 */}
            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
              <div className="flex items-center gap-3 mb-4">
                <div className="h-10 w-10 rounded-full bg-gradient-to-r from-brand-blue/15 to-brand-purple/15 flex items-center justify-center">
                  <Image className="h-5 w-5 text-brand-blue" />
                </div>
                <h4 className="font-semibold text-gray-900">方式一：单张上传</h4>
              </div>
              <p className="text-sm text-gray-600">
                在首页点击<strong>任意一张图片</strong>进入详细页，点击屏幕下方的<strong>"同步"</strong>按钮，该图片即会加入传输队列。
              </p>
            </div>

            {/* 方式二 */}
            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
              <div className="flex items-center gap-3 mb-4">
                <div className="h-10 w-10 rounded-full bg-gradient-to-r from-brand-blue/15 to-brand-purple/15 flex items-center justify-center">
                  <Layers className="h-5 w-5 text-brand-purple" />
                </div>
                <h4 className="font-semibold text-gray-900">方式二：批量上传</h4>
              </div>
              <p className="text-sm text-gray-600">
                在首页<strong>长按任意一张图片</strong>，界面将进入多选模式。此时您可以点选多张需要上传的图片，选择完毕后，点击界面下方的<strong>上传按钮</strong>即可批量同步。
              </p>
            </div>

            {/* 方式三 */}
            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
              <div className="flex items-center gap-3 mb-4">
                <div className="h-10 w-10 rounded-full bg-gradient-to-r from-brand-blue/15 to-brand-purple/15 flex items-center justify-center">
                  <Upload className="h-5 w-5 text-brand-blue" />
                </div>
                <h4 className="font-semibold text-gray-900">方式三：一键上传（推荐）</h4>
              </div>
              <p className="text-sm text-gray-600">
                如果您需要备份所有未上传的内容，只需直接点击首页右侧的<strong>蓝色"一键"按钮</strong>，APP 会自动将所有未备份（且未锁定）的文件加入上传队列，省时省力。
              </p>
            </div>
          </div>
        </div>
      </Section>

      {/* 快速开始 */}
      <Section id="quick-start" title="💡 快速开始">
        <div className="bg-gradient-to-r from-brand-blue/5 to-brand-purple/5 rounded-2xl p-8 border border-brand-blue/10">
          <div className="flex items-start gap-4">
            <div className="h-12 w-12 rounded-xl bg-gradient-to-r from-brand-blue to-brand-purple flex items-center justify-center flex-shrink-0">
              <ArrowRight className="h-6 w-6 text-white" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">开始使用</h3>
              <p className="text-gray-700">
                现在您已经了解了家忆 APP 的基本功能，赶快开始使用吧！确保手机和电脑连接在同一局域网内，打开服务端，然后在 APP 中扫描二维码即可开始同步您的珍贵回忆。
              </p>
            </div>
          </div>
        </div>
      </Section>

      {/* Footer */}
      <footer className="py-12 border-t border-gray-200 mt-12">
        <div className="max-w-5xl mx-auto px-6 text-center text-gray-600">
          <p>家忆 · JIAYI — 家庭照片私有同步系统</p>
          <p className="mt-2 text-sm">把家人的照片，安心地留在家里</p>
        </div>
      </footer>
    </div>
  );
}
