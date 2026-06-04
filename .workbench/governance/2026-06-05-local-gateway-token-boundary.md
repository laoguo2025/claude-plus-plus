# Local gateway token boundary

## Context
The P1 review found that localhost auxiliary gateway routes still trusted local reachability and origin checks too broadly. `/claude-plus/skills`, `/claude-plus/skills/:id/trash`, `/claude-plus/token-usage`, and `/claude-plus/conversation-title-i18n` expose local filesystem-derived data or local translation/usage helpers and should not be callable by arbitrary local processes or untrusted browser pages.

## Change
- Added a user-scoped persistent random local gateway token under the Claude++ app state directory.
- Required the token header for auxiliary `/claude-plus/*` routes while preserving trusted-origin checks.
- Kept Claude Desktop proxy/model routes on the existing origin boundary so non-browser Claude Desktop traffic is not broken.
- Updated installed title-translation and token-usage main-process bridges to read the token and send the header.
- Bumped only `conversation_title_i18n` and `token_usage` enhancement definitions to `v0.3` so already installed bridges can be refreshed.

## Validation
- `npm run typecheck`: passed.
- `cmd.exe /c build.bat test --lib`: passed, 123 passed / 5 ignored.
- `cmd.exe /c build.bat fmt --check`: still fails on the pre-existing `src-tauri/src/lib.rs` import ordering diff; this file was outside the P1 gateway boundary scope and was not modified.

## Rollback
Revert the local commit for source and project-map changes. Existing installed Claude Desktop enhancement bridges can be refreshed back through the Claude++ enhancement install/restore flows after rollback.
