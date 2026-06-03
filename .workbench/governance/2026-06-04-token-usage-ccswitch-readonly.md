# Token usage CC Switch readonly source

## Scope
Restore the Claude Desktop page enhancement card for `Token 使用信息` and fix its data source so the existing display fields use CC Switch's persisted proxy usage rows when available.

## Change
- Added a read-only CC Switch SQLite query for successful `claude-desktop` rows in `proxy_request_logs`.
- Extended `/claude-plus/token-usage` to accept `sinceMs`, aggregate rows for the current turn, and fall back to the existing in-memory stream parser when no fresh DB usage exists.
- Updated the injected token usage bridge to pass the current turn start time, preserve backend call counts, and stop clamping cache-read tokens to input tokens.
- Restored the `token_usage` feature definition so the page enhancement card is visible again.

## Validation
- `npm run build` passed.
- `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo test --manifest-path src-tauri\Cargo.toml --lib'` passed: 83 passed, 5 ignored.

## Rollback
Revert the local commit for this change. No CC Switch files, database rows, or configuration are modified by this implementation.
