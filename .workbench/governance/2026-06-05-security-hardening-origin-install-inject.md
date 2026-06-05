# Security Hardening: Origin, Installer, Injected UI

## Scope
User-reported security issues:
- Missing `Origin` was treated as trusted by local gateway checks.
- Windows Claude Code install command used remote script pipe execution.
- Injected token usage observer parsed broad network payloads without size/depth limits.
- Skills popup rendered dynamic skill data through HTML string assembly.

## Cause
The gateway had a binary trusted/untrusted origin helper where missing `Origin` returned trusted. That was compatible for local non-browser callers but also allowed unauthenticated non-browser requests to Claude Desktop proxy/model routes.

The welcome installer optimized for a compact official command and used direct remote PowerShell execution. The injected script used escaping for dynamic Skills HTML, but the `innerHTML` assembly pattern remained fragile.

## Change
- Gateway origin handling is now tri-state: trusted, missing, untrusted.
- Auxiliary `/claude-plus/*` requests still require the local gateway token and always reject untrusted browser origins.
- Claude Desktop proxy/model routes accept missing `Origin` only with either the local gateway token header or the configured gateway bearer key.
- Windows Claude Code install downloads the official installer to a temp `.ps1`, checks that it exists and is non-trivially sized, then runs the local script. The direct `irm | iex` / `Invoke-Expression` path was removed.
- Skills popup dynamic content and token usage badge now use DOM/text APIs for user-controlled values.
- Token usage capture now limits captured text length, Blob size, object depth, array breadth, and SSE event count.
- `plugins` and `token_usage` enhancement versions were bumped to refresh already installed injected script copies.

## Validation
- `npm run test:rust`: passed, 149 passed, 5 ignored.

## Rollback
Revert the local commit. No external Claude Desktop, CC Switch, or Claude Code state was written during this task.
