# Changelog

## Unreleased

- 仓库聚焦 FSP 协议本身：移除产品官网 `jiayi-web/` 与旧稿备份 `jiayi-fsp_bak/`
- 清理 `site/` 中未使用的 Vite 模板残留（`Empty.tsx`、`pages/Home.tsx`、`assets/react.svg`）
- 统一仓库换行符为 LF，新增 `.gitattributes`
- `.gitignore` 补全构建产物与依赖忽略规则
- README（中/英）与 AGENTS.md 补充协议仓库结构说明与部署流程
- 修复 `site/` 的 5 处 ESLint 报错，并补全 MQTT `status` 消息示例

## v0.1
- Initial public draft
- MQTT control plane
- UDP data plane
