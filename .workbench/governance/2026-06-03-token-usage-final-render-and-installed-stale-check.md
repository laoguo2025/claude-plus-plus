# Token usage final render and installed stale check

## Reason

用户反馈 Token 使用信息仍存在六个问题：每个请求结束显示一次、缓存命中率 100%、字段不符合目标口径、位置错误、对话结束后不显示、应显示在 assistant 回复底部操作行下方。

## Evidence

- 源码旧逻辑在 `cpuRememberUsage` 捕获 usage 后立即 `cpuRender()`，会导致每个请求结束都刷新/显示统计框。
- 源码旧缓存口径把 `cached_tokens` / `cachedTokens` 混入读取 token，截图中的 `cached == input` 会直接导致命中率 100%。
- 源码旧 UI 仍显示 `总计 / 缓存命中 / 缓存写`，不是用户要求的字段顺序。
- 源码旧挂载点使用 assistant turn 容器后方，且候选选择可能命中运行状态行。
- 只读检查当前 Claude Desktop 已安装资源：
  - 资源目录：`C:\Program Files\WindowsApps\Claude_1.4758.0.0_x64__pzs8sxrjxfjjc\app\resources`
  - `index-nw6xXXcG.js` 中 `__claudePlusEnhanceTokenUsageV1` 为 `v0.2`，但脚本内容仍包含旧公式 `Math.min(e.cachedReadTokens||e.cacheReadTokens||e.cached||0,t)`。
  - 已安装 bundle 不包含 `本轮调用合计` 和 `CPU_FINAL_RENDER_DELAY_MS`。
  - 结论：当前 Claude Desktop 确实仍在跑旧 Token 使用信息脚本，必须重新安装/升级页面增强资源后，新源码才会在 Claude Desktop 生效。

## Changes

- Token usage 页面脚本新增 `CPU_FINAL_RENDER_DELAY_MS` 和最终渲染调度。
- usage 捕获只更新 turn 聚合和内部状态，不立即显示；生成从 busy 变 idle 后调度最终渲染，解决结束后不显示。
- 缓存读取只来自 `cache_read_input_tokens` / `cacheReadInputTokens` / Responses details cached tokens，不再用泛化 `cached_tokens` 或 `cached_input_tokens`。
- UI 改为单行目标字段顺序：本轮调用合计、输入、输出、缓存创建、缓存读取、缓存命中率、上下文、调用次数、耗时。
- 挂载点改为最后 assistant 回复底部操作行之后；排除运行状态行。
- 新增回归测试覆盖最终渲染、缓存口径、字段顺序、挂载位置。

## Verification

- 新增测试先红灯：`cargo test token_usage_ --lib` 中 3 个新增测试失败。
- 修复后 `cargo test token_usage_ --lib`: passed, 15 passed / 2 ignored.
- `cargo test --lib`: passed, 68 passed / 5 ignored.
- `npm run build`: passed.

## Rollback

回退本轮本地提交可撤销源码改动。若后续写入 Claude Desktop 资源，使用 Claude++ 页面增强卸载/恢复入口或 `.claude-plus-enhance-backups` 恢复。
