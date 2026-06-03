# Token usage Codex++ 0.1.6 alignment

## Reason

用户提供 Codex++ `codex-token-usage` 0.1.6 的脚本配置和架构，要求调整 Claude++ 的 Token 使用信息脚本代码和配置。只读排查确认本机 Codex++ 用户脚本已启用，Claude++ 已具备页面捕获和 turn 聚合，但缺少完整的 Context Meter 联动、project/conversation scope、跨源去重、估算标记和完整调试导出状态。

## Changes

- `src-tauri/src/claude_enhance.rs`
  - Token 使用信息注入脚本补齐 Codex++ 对齐常量：recent/debug/ledger 上限、context 轮询、turn idle、context merge、跨源去重窗口。
  - 增加 projectId + conversationId + scopeKey，切换项目或对话时隔离当前轮和导出状态。
  - 增加 Context Meter 轮询与 `captureState.inspectText/inspectValue` hook。
  - 增加跨源 usage 去重，代理兜底以 `proxy` source 进入当前 turn，避免与页面捕获重复累计。
  - 增加 `totalEstimated` 标记和 UI `(估算)` 显示。
  - 暴露 `window.__claudePlusTokenUsageDebug` 与 `window.__claudePlusTokenUsage.export()`，包含 ledgerEvents 和当前 scope。
  - 新增回归测试覆盖上述合同。
- `src/shared/enhance-features.json`
  - Token 使用信息增强版本从 `v0.1` 提升到 `v0.2`，用于已启用增强的版本迁移识别。

## Verification

- 新增测试先红灯：`cargo test token_usage_matches_codex_plus_scope_context_and_debug_contract --lib` 因缺少 `CPU_RECENT_LIMIT` 失败。
- 实现后 `cargo test token_usage_matches_codex_plus_scope_context_and_debug_contract --lib`: passed.
- 实现后 `cargo test token_usage --lib`: passed, 12 passed / 2 ignored.
- `cargo test --lib`: passed, 65 passed / 5 ignored.
- `npm run build`: passed.
- `cargo fmt --check` 未作为通过项：检查失败点在既有 `src-tauri/src/lib.rs` 格式差异，不属于本轮授权范围，未改动该文件。

## Rollback

回退本轮本地提交可撤销源码和配置版本变更。若后续已通过 Claude++ 安装入口写入 Claude Desktop 资源，使用页面增强卸载/恢复入口或 `.claude-plus-enhance-backups` 恢复，不改写历史、不 force push。
