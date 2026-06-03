# Token usage persist and takeover state

## Reason
用户反馈 Token 使用信息在新一轮对话开始时闪现后消失，回复结束后仍不显示；同时指出第三张 Claude++ 接管卡片应简单表示完整接管状态，不展示配置已接管但代理未运行等中间状态。

## Changes
- `src/App.tsx`
  - 第三张卡片的“已接管”改为同时要求 Claude Desktop 配置已指向 Claude++ 且 Claude++ 本地代理运行中。
  - 提示语保持“点击接管,让 Claude++ 生效”。
- `src-tauri/src/claude_enhance.rs`
  - Token 使用信息只在发送或检测到生成开始时清除旧统计。
  - 普通空轮询不再删除已拿到的统计；最终 usage 到达后继续重试渲染到最新 assistant 回复下方。
  - 更新防回归断言，禁止恢复空 usage 立即清除逻辑。

## Validation
- 定向红灯已确认：旧实现会因 `if(!t){cpuClear();return}` 断言失败。
- `npm run build`: passed.
- MSVC 环境下 `cargo test --lib`: passed, 51 passed / 4 ignored.
- MSVC 环境下 `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_token_usage_enhance_writes_bridge --lib -- --ignored --nocapture`: passed，并通过正式安装路径重写 Claude Desktop Token 使用信息增强资源。

## Rollback
回退本轮本地提交即可撤销源码改动；如需撤销已写入 Claude Desktop 的页面增强资源，使用 Claude++ 页面增强恢复/卸载入口或恢复最新 `.claude-plus-enhance-backups`。
