# AGENTS.md

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

# 部署到 GitHub Pages
pnpm predeploy && pnpm deploy
```

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

- 协议变更需同步更新 docs/spec 目录下的规范文档
- 非规范性的说明需明确标注
- 示例代码必须严格遵循协议规范

### State Management

- 简单状态使用 React useState/useReducer
- 复杂全局状态使用 Zustand（已集成在项目中）
- 优先考虑局部状态，仅在必要时引入全局状态管理
