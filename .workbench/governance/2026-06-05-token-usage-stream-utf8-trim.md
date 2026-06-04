# Token usage stream UTF-8 trim

## Context
The P2 review found that token usage stream parsing could panic when trimming an oversized pending SSE line. The old trim used a byte offset directly with `String::drain`; if the offset landed inside a multi-byte UTF-8 character, Rust would panic.

## Change
- Added a safe pending-line trim helper that advances the trim point to a valid UTF-8 character boundary before draining.
- Kept the existing bounded-memory behavior for oversized pending lines.
- Added a regression test with a multi-byte pending line followed by a valid token usage SSE event.

## Validation
- `cmd.exe /c build.bat test token_usage_tracker --lib`: passed.
- `cmd.exe /c build.bat test --lib`: passed, 125 passed / 5 ignored.
- `npm run typecheck`: passed.
- `cmd.exe /c build.bat fmt --check`: still fails on the pre-existing `src-tauri/src/lib.rs` import ordering diff; this file was outside the token usage parser scope and was not modified.

## Rollback
Revert the local commit for this source-only parser robustness change.
