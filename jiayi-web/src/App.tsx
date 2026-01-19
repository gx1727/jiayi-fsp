import { motion } from "framer-motion";
import { ShieldCheck, UploadCloud, Images, Wifi, CheckCircle2, AlertTriangle, CloudOff, Smartphone, HardDrive } from "lucide-react";

const Section = ({ id, title, children }: { id?: string; title: string; children: React.ReactNode }) => (
  <section id={id} className="max-w-6xl mx-auto px-6 py-16">
    <h2 className="text-3xl md:text-4xl font-semibold text-gray-900 mb-8">{title}</h2>
    {children}
  </section>
);

export function App() {
  return (
    <div className="min-h-screen gradient-bg animated-bg">
      <header className="sticky top-0 z-40">
        <div className="glass border-b border-gray-200">
          <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <img src="/logo.png" alt="家忆 · JIAYI" className="h-8 w-8 rounded" />
              <span className="text-xl font-semibold text-gray-900">家忆 · JIAYI</span>
            </div>
            <nav className="hidden md:flex items-center gap-6 text-gray-700">
              <a href="#intro">介绍</a>
              <a href="#why">为什么</a>
              <a href="#solution">方案</a>
              <a href="#steps">使用步骤</a>
              <a href="#compare">对比</a>
              <a href="#audience">适用人群</a>
              <a href="#security">隐私</a>
              <a href="https://gx1727.github.io/jiayi-fsp/">JIAYI-FSP</a>
              <a href="#cta" className="px-3 py-1.5 rounded bg-gradient-to-r from-brand-blue to-brand-purple text-white">下载</a>
            </nav>
          </div>
        </div>
      </header>

      <main>
        <section className="relative overflow-hidden">
          <div className="max-w-6xl mx-auto px-6 pt-20 pb-16">
            <motion.h1
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6 }}
              className="text-4xl md:text-6xl font-bold text-gray-900 tracking-tight"
            >
              家庭照片私有同步系统
            </motion.h1>
            <motion.p
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1, duration: 0.6 }}
              className="mt-4 text-xl md:text-2xl text-gray-700"
            >
              把一家人的照片，安心地留在家里
            </motion.p>

            <div className="mt-8 flex flex-wrap gap-4">
              <a
                href="#cta"
                className="px-6 py-3 rounded-lg bg-gradient-to-r from-brand-blue to-brand-purple text-white shadow-soft"
              >
                立即下载
              </a>
              <a
                href="#intro"
                className="px-6 py-3 rounded-lg border border-gray-300 text-gray-800"
              >
                了解更多
              </a>
            </div>

            <div className="mt-12 grid grid-cols-1 md:grid-cols-3 gap-6">
              <div className="p-6 rounded-xl bg-white shadow-soft border border-gray-100">
                <ShieldCheck className="text-brand-blue" />
                <h3 className="mt-3 font-semibold text-gray-900">隐私优先</h3>
                <p className="mt-2 text-gray-700">仅在家庭局域网内传输，不经过任何第三方服务器。</p>
              </div>
              <div className="p-6 rounded-xl bg-white shadow-soft border border-gray-100">
                <UploadCloud className="text-brand-purple" />
                <h3 className="mt-3 font-semibold text-gray-900">自动同步</h3>
                <p className="mt-2 text-gray-700">手机连上家里 Wi‑Fi 即自动备份，无需手动操作。</p>
              </div>
              <div className="p-6 rounded-xl bg-white shadow-soft border border-gray-100">
                <Images className="text-brand-blue" />
                <h3 className="mt-3 font-semibold text-gray-900">集中管理</h3>
                <p className="mt-2 text-gray-700">一家人的照片统一存放，按时间与设备自动整理。</p>
              </div>
            </div>
          </div>
        </section>

        <Section id="intro" title="一句话介绍">
          <p className="text-gray-800 text-lg">
            家忆（JIAYI）是一款运行在家庭局域网内的照片同步系统。手机连上家里的 Wi‑Fi，照片就会自动备份到家里的电脑，不经过云端，不对外传输，只属于你和你的家人。
          </p>
        </Section>

        <Section id="why" title="为什么需要家忆？">
          <div className="glass rounded-2xl p-6 border border-gray-200">
            <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6">
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                whileHover={{ y: -2 }}
                className="p-[1px] rounded-xl bg-gradient-to-r from-brand-blue/30 to-brand-purple/30"
              >
                <div className="rounded-xl bg-white p-6 shadow-soft">
                  <div className="h-10 w-10 rounded-full bg-gradient-to-r from-brand-blue/15 to-brand-purple/15 flex items-center justify-center">
                    <AlertTriangle className="text-brand-blue" />
                  </div>
                  <h3 className="mt-3 font-semibold text-gray-900">换机担心丢失</h3>
                  <p className="mt-2 text-gray-700">照片越来越多，换手机迁移容易遗漏。</p>
                </div>
              </motion.div>
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                whileHover={{ y: -2 }}
                className="p-[1px] rounded-xl bg-gradient-to-r from-brand-blue/30 to-brand-purple/30"
              >
                <div className="rounded-xl bg-white p-6 shadow-soft">
                  <div className="h-10 w-10 rounded-full bg-gradient-to-r from-brand-blue/15 to-brand-purple/15 flex items-center justify-center">
                    <Smartphone className="text-brand-purple" />
                  </div>
                  <h3 className="mt-3 font-semibold text-gray-900">家庭设备分散</h3>
                  <p className="mt-2 text-gray-700">多人多设备，重要照片到处都是。</p>
                </div>
              </motion.div>
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                whileHover={{ y: -2 }}
                className="p<[>][1px] rounded-xl bg-gradient-to-r from-brand-blue/30 to-brand-purple/30"
              >
                <div className="rounded-xl bg-white p-6 shadow-soft">
                  <div className="h-10 w-10 rounded-full bg-gradient-to-r from-brand-blue/15 to-brand-purple/15 flex items-center justify-center">
                    <CloudOff className="text-brand-blue" />
                  </div>
                  <h3 className="mt-3 font-semibold text-gray-900">云端隐私顾虑</h3>
                  <p className="mt-2 text-gray-700">数据依赖第三方，长期安全不踏实。</p>
                </div>
              </motion.div>
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                whileHover={{ y: -2 }}
                className="p-[1px] rounded-xl bg-gradient-to-r from-brand-blue/30 to-brand-purple/30"
              >
                <div className="rounded-xl bg-white p-6 shadow-soft">
                  <div className="h-10 w-10 rounded-full bg-gradient-to-r from-brand-blue/15 to-brand-purple/15 flex items-center justify-center">
                    <HardDrive className="text-brand-purple" />
                  </div>
                  <h3 className="mt-3 font-semibold text-gray-900">整理不成体系</h3>
                  <p className="mt-2 text-gray-700">没有统一归档方式，管理效率低。</p>
                </div>
              </motion.div>
            </div>
            <p className="mt-6 text-gray-900 font-medium">照片是回忆，一旦丢了，就再也回不来。</p>
          </div>
        </Section>

        <Section id="solution" title="家忆是怎么解决的？">
          <div className="grid md:grid-cols-3 gap-6">
            <div className="p-6 rounded-xl bg-white shadow-soft border border-gray-100">
              <Wifi className="text-brand-blue" />
              <h3 className="mt-3 font-semibold text-gray-900">照片只在家里</h3>
              <p className="mt-2 text-gray-700">所有照片只在家庭局域网内传输，不经过任何第三方服务器。</p>
            </div>
            <div className="p-6 rounded-xl bg-white shadow-soft border border-gray-100">
              <UploadCloud className="text-brand-purple" />
              <h3 className="mt-3 font-semibold text-gray-900">自动同步</h3>
              <p className="mt-2 text-gray-700">新拍的照片会自动同步到家忆服务器，后台完成，无需操心。</p>
            </div>
            <div className="p-6 rounded-xl bg-white shadow-soft border border-gray-100">
              <Images className="text-brand-blue" />
              <h3 className="mt-3 font-semibold text-gray-900">集中管理</h3>
              <p className="mt-2 text-gray-700">多部手机的照片统一存放，按时间、设备自动整理，在电脑集中浏览与管理。</p>
            </div>
          </div>
        </Section>

        <Section id="steps" title="使用只需要 3 步">
          <ol className="grid md:grid-cols-3 gap-6 text-gray-800">
            <li className="p-6 rounded-xl bg-white shadow-soft border border-gray-100">
              <CheckCircle2 className="text-brand-blue" />
              <p className="mt-2">在家里的电脑上安装 JIAYI Server</p>
            </li>
            <li className="p-6 rounded-xl bg-white shadow-soft border border-gray-100">
              <CheckCircle2 className="text-brand-purple" />
              <p className="mt-2">在手机上安装 家忆 App</p>
            </li>
            <li className="p-6 rounded-xl bg-white shadow-soft border border-gray-100">
              <CheckCircle2 className="text-brand-blue" />
              <p className="mt-2">手机连上家庭 Wi‑Fi，照片自动同步</p>
            </li>
          </ol>
          <p className="mt-4 text-gray-700">不需要复杂设置，也不需要注册账号。</p>
        </Section>

        <Section id="compare" title="和云相册有什么不同？">
          <div className="overflow-x-auto rounded-xl border border-gray-200">
            <table className="min-w-full bg-white">
              <thead>
                <tr className="text-left">
                  <th className="p-4">对比项</th>
                  <th className="p-4">家忆 JIAYI</th>
                  <th className="p-4">云相册</th>
                </tr>
              </thead>
              <tbody className="text-gray-800">
                <tr>
                  <td className="p-4">照片存放位置</td>
                  <td className="p-4">家里电脑</td>
                  <td className="p-4">云服务器</td>
                </tr>
                <tr className="bg-gray-50">
                  <td className="p-4">是否经过第三方</td>
                  <td className="p-4">否</td>
                  <td className="p-4">是</td>
                </tr>
                <tr>
                  <td className="p-4">隐私控制</td>
                  <td className="p-4">完全自主</td>
                  <td className="p-4">依赖平台</td>
                </tr>
                <tr className="bg-gray-50">
                  <td className="p-4">同步环境</td>
                  <td className="p-4">家庭局域网</td>
                  <td className="p-4">互联网</td>
                </tr>
                <tr>
                  <td className="p-4">使用方式</td>
                  <td className="p-4">一次部署</td>
                  <td className="p-4">长期依赖</td>
                </tr>
              </tbody>
            </table>
          </div>
          <p className="mt-4 text-gray-800">
            家忆不是替代云服务，而是为重视家庭隐私和长期保存的用户提供另一种选择。
          </p>
        </Section>

        <Section id="audience" title="适合哪些家庭？">
          <ul className="grid md:grid-cols-2 gap-4 text-gray-800">
            <li>给父母手机做自动照片备份</li>
            <li>长期保存孩子成长照片</li>
            <li>多部手机需要集中管理</li>
            <li>不希望家庭照片长期放在云端</li>
          </ul>
        </Section>

        <Section id="security" title="关于安全与隐私">
          <ul className="grid md:grid-cols-2 gap-4 text-gray-800">
            <li>家忆不会收集任何照片内容</li>
            <li>不连接云端服务器</li>
            <li>所有数据都存放在你家中的设备</li>
            <li>你随时可以备份或迁移数据</li>
          </ul>
          <p className="mt-4 text-gray-900 font-medium">照片的安全，由你自己决定。</p>
        </Section>

        <Section id="cta" title="开始使用家忆">
          <div className="flex flex-wrap gap-4">
            <a className="px-6 py-3 rounded-lg bg-gradient-to-r from-brand-blue to-brand-purple text-white shadow-soft" href="#">
              下载家忆
            </a>
            <a className="px-6 py-3 rounded-lg border border-gray-300 text-gray-800" href="#">
              查看安装说明
            </a>
          </div>
          <p className="mt-6 text-gray-700">把家人的照片，安心地留在家里。</p>
        </Section>
      </main>

      <footer className="py-10 border-t border-gray-200">
        <div className="max-w-6xl mx-auto px-6 text-gray-600">
          <p>家忆 · JIAYI — 家庭照片私有同步系统</p>
        </div>
      </footer>
    </div>
  );
}
