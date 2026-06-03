# Token usage turn aggregation

## Reason
用户指出 Token 使用信息不应每次请求显示一个框，而应按每轮对话聚合统计；Codex++ `codex-token-usage` 脚本也是按 `currentTurn.calls` 聚合后显示一条统计。

## Changes
- `src-tauri/src/claude_enhance.rs`
  - Token 使用信息注入脚本新增 `currentTurn` 状态。
  - 发送消息或检测到生成开始时创建新轮次。
  - 页面内捕获和代理兜底 usage 都追加到当前轮 `calls`。
  - 多次调用聚合成一个统计框；调用数大于 1 时显示“调用 N 次”。
  - 代理兜底按 usage id 去重，避免轮询重复累加。
  - 新增防回归测试，要求注入脚本具备 `cpuBeginTurn`、`cpuAggregateTurn`、`callCount` 和代理兜底并入聚合。

## Validation
- 定向红灯已确认：旧实现缺少 `currentTurn`，新增测试失败。
- MSVC 环境下 `cargo test token_usage --lib`: passed, 10 passed / 1 ignored.
- `npm run build`: passed.
- MSVC 环境下 `cargo test --lib`: passed, 53 passed / 4 ignored.
- MSVC 环境下 `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_token_usage_enhance_writes_bridge --lib -- --ignored --nocapture`: passed，并通过正式安装路径重写 Claude Desktop Token 使用信息增强资源。
- 实际 Claude Desktop 前端 bundle 已确认包含 `cpuBeginTurn`、`cpuAggregateTurn`、`cpuRememberUsage`、`lastProxyId`、`callCount`。

## Rollback
回退本轮本地提交即可撤销源码改动；如需撤销已写入 Claude Desktop 的页面增强资源，使用 Claude++ 页面增强恢复/卸载入口或恢复最新 `.claude-plus-enhance-backups`。
