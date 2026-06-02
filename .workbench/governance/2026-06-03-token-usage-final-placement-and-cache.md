# Token usage final placement and cache fix

## Reason
用户反馈 Claude Desktop 中 token 使用信息仍存在三类问题：
- 本轮对话开始后提前显示，结束后反而不稳定。
- 缓存命中率出现超过 100% 的异常值。
- 显示位置不在 AI 回复下方的对话区域，且不是居中两行。

## Changes
- `src-tauri/src/proxy.rs`
  - 流式响应 chunk 只累积 token usage，不再逐块写入 `/claude-plus/token-usage` 状态。
  - 新请求开始时清空待发布状态，避免上一轮统计被前端当成本轮数据。
  - 响应流结束后一次性发布最终统计。
  - `cached_tokens` 夹紧到 `input_tokens`，`cache_creation_tokens` 夹紧到剩余输入 token。
- `src-tauri/src/claude_enhance.rs`
  - 前端只在轮询到新的 finalized usage id 后渲染统计块，不再在普通 DOM tick 中移动旧统计。
  - 统计块插入到最近 assistant turn 外层节点之后，作为对话流中的居中块。
  - 统计文案拆为两行，前端缓存率也按 `Math.min(cached,input)/input` 计算。

## Validation
- `npm run build`: passed.
- `cargo fmt --check`: passed.
- MSVC 环境下 `cargo test --lib`: passed, 49 passed / 4 ignored.
- MSVC 环境下 `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_token_usage_enhance_writes_bridge --lib -- --ignored --nocapture`: passed.

## Rollback
如需回退本次改动，回退本轮本地提交即可；不需要改写历史或 force push。
