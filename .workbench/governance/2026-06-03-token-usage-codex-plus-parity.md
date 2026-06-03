# Token usage Codex++ parity fix

## Reason
用户反馈 token 使用信息仍在本轮开始后显示旧数据，且缓存命中率长期 100%。随后要求参考本机 Codex++ 安装和用户脚本的 token usage 条插入位置、插入时机和缓存口径。

## Reference
- Codex++ 安装目录：`C:\Users\Administrator\AppData\Local\Programs\Codex++`
- Codex++ 用户脚本目录：`C:\Users\Administrator\AppData\Roaming\Codex++\user_scripts`
- 对标脚本：`market-codex-token-usage.js`
- 关键参考口径：
  - 发送/网络请求开始即标记本轮开始。
  - 只在捕获到完整 usage metric 后渲染统计条。
  - 缓存读取和缓存写入分开统计，缓存命中率使用缓存读取 token / 输入 token。

## Changes
- `src-tauri/src/proxy.rs`
  - token usage 从跨节点 `max()` 聚合改为候选 usage 打分选择，避免把不同层级/阶段字段拼成 `cached == input`。
  - 不再把 `cached_input_tokens` 当作缓存命中字段。
  - 继续支持 Responses 风格 `input_tokens_details.cached_tokens` 和 `prompt_tokens_details.cached_tokens`。
  - 保留流结束后才发布 finalized usage。
- `src-tauri/src/claude_enhance.rs`
  - 收到空 usage 时立即清除旧统计条。
  - 发送、submit、Enter 触发时立即清除旧统计条并轮询。
  - 检测到生成中 stop 按钮时立即清除旧统计条。
  - 保留对话区最近 assistant 回复下方两行显示。

## Validation
- `npm run build`: passed.
- `cargo fmt --check`: passed.
- MSVC 环境下 `cargo test --lib`: passed, 51 passed / 4 ignored.
- MSVC 环境下 `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_token_usage_enhance_writes_bridge --lib -- --ignored --nocapture`: passed.
- MSVC 环境下 `npm run tauri build`: passed, produced release exe and NSIS installer.

## Rollback
回退本轮本地提交即可；不需要改写历史或 force push。
