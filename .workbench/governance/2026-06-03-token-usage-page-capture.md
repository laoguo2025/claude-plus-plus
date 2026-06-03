# Token usage page capture

## Reason
用户要求对照 Codex++ `codex-token-usage` 0.1.6 脚本和本机 Codex++ 配置，修复 Claude++ Token 使用信息增强。排查确认 Codex++ 是页面内包装 `fetch`、`XMLHttpRequest`、`WebSocket`、`postMessage` 捕获真实响应 usage；Claude++ 之前只轮询 `/claude-plus/token-usage` 全局代理缓存，容易出现空缓存、跨轮次污染和缓存命中口径错误。

## Changes
- `src-tauri/src/claude_enhance.rs`
  - Token 使用信息注入脚本新增页面内 usage 捕获：`fetch`、`XMLHttpRequest`、`WebSocket`、`postMessage`。
  - 支持 JSON 与 SSE `data:` 片段提取 usage。
  - 保留 `/claude-plus/token-usage` 作为代理兜底来源。
  - 缓存显示改为优先使用 `cachedReadTokens` / `cacheReadTokens`，并显示缓存写入 token。
  - 新增防回归测试，要求注入脚本包含页面内网络捕获和缓存读写分离字段。

## Validation
- 定向红灯已确认：新增测试在旧实现下因缺少 `window.__claudePlusTokenUsage` 失败。
- MSVC 环境下 `cargo test token_usage --lib`: passed, 9 passed / 1 ignored.
- `npm run build`: passed.
- MSVC 环境下 `cargo test --lib`: passed, 52 passed / 4 ignored.
- MSVC 环境下 `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_token_usage_enhance_writes_bridge --lib -- --ignored --nocapture`: passed，并通过正式安装路径重写 Claude Desktop Token 使用信息增强资源。
- 实际 Claude Desktop 前端 bundle 已确认包含 `cpuInstallFetchObserver`、`cpuInstallXhrObserver`、`cpuInstallWebSocketObserver`、`cpuInstallPostMessageObserver` 和 `cachedReadTokens`。

## Rollback
回退本轮本地提交即可撤销源码改动；如需撤销已写入 Claude Desktop 的页面增强资源，使用 Claude++ 页面增强恢复/卸载入口或恢复最新 `.claude-plus-enhance-backups`。
