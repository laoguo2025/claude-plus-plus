# Enhance bridge runtime port

## Context
The P1 review found that Claude Desktop page-enhancement bridge scripts baked the local gateway URL with the proxy port resolved at install time. Changing `CLAUDE_PLUS_PROXY_PORT` or `%USERPROFILE%\.claude-plus-plus\settings.json` after installation could leave `conversation_title_i18n` and `token_usage` calling the old port from Claude Desktop resources.

## Change
- Main-process bridges now resolve the local gateway port at bridge runtime from `CLAUDE_PLUS_PROXY_PORT`, then `settings.json` (`proxyPort` / `proxy_port`), then the default port.
- Title translation and token usage bridges build local gateway URLs from the runtime port and still send the local gateway token header.
- The token usage page script no longer contains a direct local-gateway fetch fallback; local gateway calls go through the authenticated main-process bridge.
- Bumped only `conversation_title_i18n` and `token_usage` to `v0.4` so installed bridge scripts can migrate.

## Validation
- Targeted bridge tests passed for title i18n and token usage.
- `npm run typecheck`: passed.
- `cmd.exe /c build.bat test --lib`: passed, 123 passed / 5 ignored.
- `cmd.exe /c build.bat fmt --check`: still fails on the pre-existing `src-tauri/src/lib.rs` import ordering diff; this file was outside the bridge URL scope and was not modified.

## Rollback
Revert the local commit for source and project-map changes. Existing installed Claude Desktop enhancement bridges can be refreshed through the Claude++ enhancement install/restore flows after rollback.
