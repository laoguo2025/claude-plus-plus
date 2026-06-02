# 2026-06-03 Review Remaining Fixes

## Scope
User approved fixing review items 1-7:
1. MSVC/Rust validation environment.
2. Safer `unregister_languages`.
3. Incremental token usage SSE parsing.
4. Proxy health check and self-recovery.
5. Mapping monitor consistency.
6. Enhance feature single data source.
7. Configurable local bridge URLs instead of source hardcoded `127.0.0.1:15722`.

## Changes
- Verified Rust checks by initializing the Visual Studio Build Tools environment through `vcvars64.bat`.
- Limited Claude language unregistering to the known language-list bundle instead of global string replacement.
- Reworked token usage tracking to parse SSE lines incrementally and keep only the pending partial line plus aggregate usage.
- Added proxy health probing and restart-on-applied-config paths for status, route activation, setup, and mapping monitor loops.
- Included mapping API key/base URL in the monitor fingerprint so credential/route changes refresh Claude Desktop config consistently.
- Moved enhance feature labels/descriptions/order to `src/shared/enhance-features.json`, used by both React preview and Tauri status.
- Generated title/token local gateway URLs from `DEFAULT_PROXY_PORT` in the injected/main bridge scripts.

## Validation
- `npm run build` passed.
- `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo check'` passed without warnings.
- `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo test --lib'` passed: 46 passed, 3 ignored.

## Rollback
- Revert this local commit to restore prior parsing, proxy lifecycle, enhance feature definitions, and bridge URL behavior.
- Claude Desktop installed resource changes were not applied during this task; no external app rollback is required for the verification performed here.
