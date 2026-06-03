# Token usage history autopoll fix

## Problem
Opening a historical Claude Desktop conversation could show and refresh `Token 使用信息` without sending a new message. The injected script polled `/claude-plus/token-usage` while idle, and the backend could return the latest CC Switch row when no turn start timestamp was supplied.

## Change
- `cpuPoll` now returns unless a current turn has `startedAt`.
- Idle `cpuTick` no longer starts token polling; the interval only polls when `currentTurn` exists.
- Fetch/XHR observers no longer create a token turn from historical page loads or background requests.
- `cpuRememberUsage` refuses usage payloads when no active current turn exists or when scope changes.
- The token usage endpoint queries CC Switch only when `sinceMs` is supplied, and filters stale snapshots for turn-scoped requests.

## Validation
- `npm run build` passed.
- `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo test --manifest-path src-tauri\Cargo.toml --lib'` passed: 84 passed, 5 ignored.

## Rollback
Revert the local commit for this change.
