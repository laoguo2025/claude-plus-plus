# Token usage uninstall semantics

## Context

After disabling `Token 使用信息` and restarting Claude Desktop, local Code chat still stayed in thinking state and the stop button did not interrupt the run.

## Evidence

- Claude Desktop logs showed `LocalSessions.sendMessage` followed by long cycles with `hadFirstResponse=false`, then `query.interrupt() failed` and `Query closed before response received`.
- Installed Claude Desktop resources no longer had the token usage enable payload or token usage bridge after uninstall, but the shared injected script still executed `cpuInstallFetchObserver`, `cpuInstallXhrObserver`, `cpuInstallWebSocketObserver`, and `cpuInstallPostMessageObserver` unconditionally.

## Change

- Token usage network/message observers are now installed only when `__claudePlusEnhanceTokenUsageV1` is enabled.
- When the last enabled page enhancement is uninstalled, the shared injected script is removed from the Claude Desktop frontend bundle.
- Enhance card actions now use the explicit labels `安装` and `卸载`.
- Added unit coverage for guarded token usage observer installation, last-feature shared-script removal, and ignored real-uninstall verification.

## Verification

- `cargo test claude_enhance::imp::tests --lib`: 38 passed, 4 ignored.
- `npm run build`: passed.
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_uninstall_token_usage_enhance_removes_observers --lib -- --ignored --nocapture`: passed and left local Claude Desktop token usage uninstalled.
- Installed resource grep after uninstall showed no token usage enable payload or bridge; remaining `cpuInstallObservers` exits unless the token usage marker is enabled.

## Rollback

Revert this commit for source changes. For installed Claude Desktop resources, use the Claude++ page-enhancement install/uninstall controls or restore the latest `.claude-plus-enhance-backups` snapshot under the Claude Desktop resources directory.
