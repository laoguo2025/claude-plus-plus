# Local Gateway Hardening

## Scope
- Hardened local HTTP gateway origin handling.
- Reduced default proxy exposure on app startup.
- Added minimal CI and ignore rules for build outputs.

## Reason
Full project review found that localhost HTTP routes exposed skills, token usage, title translation, model discovery, and request forwarding without origin checks. The app also started the proxy unconditionally, so the HTTP surface was exposed even when Claude Desktop had not been connected to Claude++.

## Changes
- Local gateway now rejects browser requests with untrusted `Origin` values while preserving non-browser callers without `Origin`.
- Startup restores the proxy only when Claude Desktop is already configured to use Claude++.
- Installing `conversation_title_i18n` or `token_usage` starts the proxy after the install succeeds, because those enhancements depend on the local gateway.
- Added CI for frontend checks, Rust checks, and visible-copy audit.
- Added `.gitignore` entries for local release/output/Tauri target artifacts.

## Validation
- `npm run typecheck` passed.
- `npm run build` passed.
- `npm run audit:claude-zh` passed with `count: 0`.
- `npm run check:rust` passed.
- `npm run test:rust` passed: 118 passed, 5 ignored.

## Rollback
Revert the local commit. This change only touches repository files and does not modify Claude Desktop, CC Switch, or external services.
