# AGENTS.md

## Repository Layout

本仓库专注于 **JIAYI-FSP 协议本身**（规范、参考实现、开发者工具），不包含终端用户产品应用。

```
proto/        协议数据结构定义（C / Rust）
docs/spec/    规范性文档——唯一事实来源
docs/         规范源文件（GitHub Pages 对外提供；构建产物不入库）
doc/          单文件版规范合集（由 tools/ 自动生成，勿手改）
examples/     Broker ACL 与参考实现示例
  rust/       可执行的一致性套件（确定性仿真 + 真实 UDP）
tools/        规范工具（合集生成 / 一致性检查）+ 调试工具集
site/         协议官网源码
```

## Specification Workflow

规范变更的固定流程，**`docs/spec/` 是唯一事实来源**：

```bash
# 1. 修改 docs/spec/ 下的分章节规范
# 2. 重新生成单文件合集（doc/ 下的合集是产物，不要手动编辑）
python3 tools/build-spec-compilation.py

# 3. 跑一致性检查（端序 / session_id / file_hash / Topic 层级 / proto 对齐 / 版本号）
python3 tools/check-spec-consistency.py

# 4. 校验模式（CI 用）：合集与源文件不一致时退出码为 1
python3 tools/build-spec-compilation.py --check
```

CI（`.github/workflows/spec-check.yml`）会强制执行第 3、4 步，
并在 `conformance` 作业中运行参考实现的一致性场景（`cargo test`）。

### 参考实现的一致性场景（examples/rust/）

规范里的**规范性要求**必须在 `examples/rust/fsp-sim` 中有对应场景：

```bash
cd examples/rust
cargo test                # 29 个核心单测 + 14 个一致性场景
cargo run -p fsp-sim      # 跑一遍全部场景并打印观测结果
```

- 新增/修改规范中的**规范性**行为时，**必须**同步新增或调整对应场景；
- 场景的判据写在 `fsp-sim/src/scenarios.rs` 的 `pass` 字段里；
- 仿真必须**确定性**（带种子的 PRNG）；故障注入只作用于首次出现。
- `World` 区分**会话方向**：`World::new` 是上传（C→S），`World::new_download` 是下载（S→C）。
  方向决定角色 —— 上传方向 Client 发送 / Server 接收，下载方向相反；
  但**无论哪个方向，发起会话的都是 Client，发布完成消息的都是接收侧**。

## Build, Lint, and Test Commands

### Frontend Site (site/)

所有命令在 `site/` 目录下运行，使用 pnpm：

```bash
# 安装依赖
pnpm install

# 开发模式
pnpm dev

# 构建生产版本
pnpm build

# 类型检查
pnpm check

# 代码检查
pnpm lint

# 预览构建结果
pnpm preview

# 把规范源文件复制进构建产物（部署必需，否则 /spec/*.md 会 404）
npm run copy:spec

# 部署到 GitHub Pages（发布到 gh-pages 分支）
# 产物包含：站点本体 + docs/spec/ + docs/diagrams/
npm run deploy
```

### 官网部署模型

官网由 **gh-pages 分支**承载，**不提交构建产物到 main**：

| 位置 | 内容 | 是否入库 |
|------|------|----------|
| `main:site/` | 站点源码 | ✅ |
| `main:docs/spec/`、`main:docs/diagrams/` | 规范源文件（Pages 对外提供） | ✅ |
| `main:docs/assets/` 等 | 带 hash 的构建产物 | ❌ 已 gitignore |
| `gh-pages` 分支 | 部署产物（站点 + spec + diagrams） | — |

- 部署由 `.github/workflows/deploy-pages.yml` 自动执行（push 到 main 时触发）。
- **一次性前置设置**：Settings → Pages → Source ＝ *Deploy from a branch*，
  分支选 `gh-pages`，目录选 `/(root)`。
- 本地预览部署产物：`cd site && npm run build && npm run copy:spec`，
  然后用任意静态服务器指向 `site/dist/`。

## Code Style Guidelines

### Imports

```tsx
// 标准导入顺序
import React, { useState, useEffect } from 'react';           // React 核心
import { BrowserRouter, Routes, Route } from 'react-router-dom'; // 第三方库
import Layout from './components/layout/Layout';              // 相对路径导入
import type { SomeType } from '@/types';                      // 别名路径
```

### TypeScript

- 使用显式类型注解，特别是在函数参数和返回值
- 使用 `React.FC<Props>` 定义组件类型
- 自定义类型使用 PascalCase（如 `type Theme = 'light' | 'dark'`）
- 接口使用 PascalCase（如 `interface LayoutProps`）

```tsx
interface LayoutProps {
  children: React.ReactNode;
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
  return <div>{children}</div>;
};
```

### React Components

- 组件文件使用 PascalCase（如 `ArchitectureDiagram.tsx`）
- 默认导出组件：`export default Layout;`
- 使用函数组件而非类组件
- 推荐使用 hooks 管理状态和副作用
- 页面组件使用 lazy loading 异步加载

```tsx
const HomePage = lazy(() => import('./pages/HomePage'));

function App() {
  return (
    <Suspense fallback={<Loading />}>
      <Routes>
        <Route path="/" element={<HomePage />} />
      </Routes>
    </Suspense>
  );
}
```

### Hooks

- Hooks 使用 camelCase，前缀 `use`（如 `useTheme`）
- 放在 `src/hooks/` 目录下
- 自定义 hooks 返回对象或数组，包含状态和相关方法

```tsx
export function useTheme() {
  const [theme, setTheme] = useState<Theme>('light');
  const toggleTheme = () => setTheme(prev => prev === 'light' ? 'dark' : 'light');
  return { theme, toggleTheme, isDark: theme === 'dark' };
}
```

### Utilities

- 工具函数放在 `src/lib/` 或 `src/utils/` 目录
- 使用 camelCase 命名（如 `cn`, `formatDate`）
- 辅助函数默认导出

```tsx
// src/lib/utils.ts
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

### CSS and Styling

- 使用 Tailwind CSS 进行样式设计
- 自定义颜色配置在 `tailwind.config.js` 的 `theme.extend.colors`
- 使用 `cn()` 工具函数合并类名

```tsx
<div className="min-h-screen flex flex-col bg-white">
  <motion.main
    initial={{ opacity: 0, y: 20 }}
    animate={{ opacity: 1, y: 0 }}
    className="flex-grow pt-16"
  >
    {children}
  </motion.main>
</div>
```

### Animations

- 使用 `framer-motion` 库实现动画
- 常用动画模式：`initial`, `animate`, `exit`, `transition`

```tsx
<motion.div
  initial={{ opacity: 0, y: 20 }}
  animate={{ opacity: 1, y: 0 }}
  transition={{ duration: 0.5 }}
>
  {children}
</motion.div>
```

### Naming Conventions

- 组件/类型：PascalCase（如 `ArchitecturePage`）
- 函数/变量/属性：camelCase（如 `toggleTheme`）
- 常量：UPPER_SNAKE_CASE 或 PascalCase
- 文件：kebab-case 用于配置文件，PascalCase 用于组件

### Error Handling

- 使用 try-catch 处理异步操作
- 组件边界使用 Suspense 和 Error Boundary
- 用户可见错误应提供友好的提示信息

### Path Aliases

- 使用 `@/` 别名指向 `src/` 目录
- 配置在 `tsconfig.json` 的 `paths` 中

### Protocol Implementation (proto/)

- C 结构体使用 `#[repr(C)]` 属性
- Rust 结构体使用 `pub` 字段暴露公共成员
- 协议相关代码需与 docs/spec 目录下的规范保持一致

### Documentation

- 协议变更**只需**修改 `docs/spec/` 目录下的分章节规范，随后运行 `tools/build-spec-compilation.py` 重新生成合集
- `doc/` 下的单文件合集是**生成产物**，禁止手动编辑
- 非规范性的说明需明确标注
- 示例代码必须严格遵循协议规范
- 协议字段/类型/Topic 变更后，必须通过 `tools/check-spec-consistency.py`

### State Management

- 简单状态使用 React useState/useReducer
- 复杂全局状态使用 Zustand（已集成在项目中）
- 优先考虑局部状态，仅在必要时引入全局状态管理
