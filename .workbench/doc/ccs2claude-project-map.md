# ccs2claude Project Map

## Purpose
`ccs2claude` is a local Tauri gateway between Claude Desktop 3P and CC Switch. Claude Desktop points to `127.0.0.1:15722/claude-desktop`; the gateway reads CC Switch's current claude-desktop provider mapping from the CC Switch SQLite database and forwards requests to CC Switch.

## Stable Boundaries
- Read-only source of model routes: `%USERPROFILE%\.cc-switch\cc-switch.db`, table `providers`, current row `app_type='claude-desktop' AND is_current=1`.
- Claude Desktop integration writes a separate `ccs2claude` configLibrary entry and must not edit CC Switch's `00000000-0000-4000-8000-000000157210` entry.
- The `ccs2claude` configLibrary entry must omit `inferenceModels` so Claude Desktop uses `/v1/models` discovery.
- The proxy must stay running while Claude Desktop is configured to use `ccs2claude`; otherwise Claude Desktop cannot load model discovery.

## Runtime Entry Points
- Tauri lifecycle and commands: `src-tauri/src/lib.rs`.
- Proxy lifecycle and CC Switch config field reads: `src-tauri/src/server.rs`.
- HTTP gateway routes and model rewrite: `src-tauri/src/proxy.rs`.
- CC Switch DB mapping read: `src-tauri/src/ccswitch_db.rs`.
- Claude Desktop configLibrary write/revert: `src-tauri/src/cd_config.rs`.

## Rollback
- In the app, use the revert command to set Claude Desktop `appliedId` back to CC Switch's `00000000-0000-4000-8000-000000157210` entry.
- For code rollback, revert the latest local commit; do not rewrite shared history or force push.
