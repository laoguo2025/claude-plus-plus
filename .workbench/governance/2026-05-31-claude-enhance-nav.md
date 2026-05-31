# Claude Desktop 页面增强软入口

## 目标
- 在 Claude++ 的“页面增强”页提供第一阶段管理入口。
- 对 Claude Desktop 前端注入侧边栏软入口：第三方API、插件与技能、MCP与扩展。
- 三个软入口均带独立图标，避免只显示纯文字菜单项。
- 页面增强管理页改为 5 张增强项卡片加 1 张重启卡片；每个增强项只保留“增强 / 取消”。
- 三个入口只跳转/定位到 Claude Desktop 已有自定义/开发者页面，不新增 Claude Desktop 页面。

## 变更原因
- Claude Desktop 相关入口已存在但路径较深，用户希望把软入口放到左侧菜单“计划任务”下方。
- Codex++ 的页面增强采用外部注入和观察 DOM 的思路，本项目已有 Claude Desktop 资源备份、停止、启动链路，因此本轮复用本项目资源补丁模式。

## 接入与回退
- 接入点：Claude Desktop `resources/ion-dist/assets/v1/index-*.js` 前端入口文件。
- 写入前自动备份到 `resources/.claude-plus-enhance-backups/<timestamp>/`。
- 回退方式：Claude++ 页面增强页点击“恢复原版”，恢复最近一次增强备份并重启 Claude Desktop。
- 本轮不改 `app.asar`，避免非等长补丁带来的 asar header 重写风险。

## 验证
- `npm run build` 通过。
- `cargo check` 通过；仅剩既有 `server.rs` dead_code warning。
- 内容检查确认前端包含 `claude_enhance_status`、`install_claude_enhance`、`第三方API`、`Markdown 导出`、`Conversation Timeline`。
- 内容检查确认 Claude Desktop 注入脚本包含 `cpe-icon` 与三个内联 SVG 图标。
- 浏览器验证页面增强页只有 6 张卡片：第三方API、插件与技能、MCP与扩展、Markdown 导出、Conversation Timeline、重启 Claude Desktop；旧检测/备份/范围卡片已移除。
