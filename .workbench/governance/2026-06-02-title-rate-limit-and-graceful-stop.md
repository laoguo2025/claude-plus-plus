# 2026-06-02 title rate limit and graceful stop

## Basis
- User approved the fifth batch of fixes from the documented review.
- Scope is limited to local title-translation rate limiting and Claude Desktop shutdown behavior.

## Changes
- Added an in-process fixed-window limiter for `/claude-plus/conversation-title-i18n`.
- Over-limit title translation requests return HTTP 429 with a JSON error before selecting models or calling upstream.
- Claude Desktop stop now tries non-forced `taskkill /IM Claude.exe /T` first, with an 8 second timeout, then falls back to `/F`.
- Preserved existing "not running" taskkill handling as success.

## Non-changes
- Did not change title request validation, prompt construction, model selection, or upstream forwarding for allowed requests.
- Did not persist rate-limit state or expose configuration.
- Did not change Claude Desktop launch paths.

## Rollback
- Revert this local commit; no external writes or push are required.

## Verification
- `npm run build` passed.
- `cargo fmt --check` passed.
- `git diff --check` passed.
- Targeted `cargo test rate_limiter_resets_after_window` could not reach project tests because this shell cannot find MSVC `cl.exe`; native dependency build scripts for `libsqlite3-sys`, `vswhom-sys`, and `ring` fail first.
- `cargo check` has the same `cl.exe` environment blocker.
