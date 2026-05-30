# Claude++ Project Map

## Purpose
`Claude++` is a local Tauri gateway between Claude Desktop 3P and CC Switch. Claude Desktop points to `127.0.0.1:15722/claude-desktop`; the gateway reads CC Switch's current claude-desktop provider mapping from the CC Switch SQLite database and forwards requests to CC Switch.

## Stable Boundaries
- Read-only source of model routes: `%USERPROFILE%\.cc-switch\cc-switch.db`, table `providers`, current row `app_type='claude-desktop' AND is_current=1`.
- Claude Desktop integration writes a separate `Claude++` configLibrary entry and must not edit CC Switch's `00000000-0000-4000-8000-000000157210` entry.
- The `Claude++` configLibrary entry must omit `inferenceModels` so Claude Desktop uses `/v1/models` discovery.
- The proxy must stay running while Claude Desktop is configured to use `Claude++`; otherwise Claude Desktop cannot load model discovery.
- Claude Desktop discovers gateway models only during app startup in the observed Windows Store build. `Claude++` still sends no-cache headers on `/v1/models` and refreshes its own Claude Desktop configLibrary entry when CC Switch mappings change so credentials stay current, but the Claude Desktop model picker requires a Claude Desktop restart to show a changed model list.
- Model discovery IDs must be unique by role, not by CC Switch display label, because multiple roles can share the same labelOverride. Discovery display names should include role plus label, e.g. `Opus - mimo-v2.5-pro`, while request forwarding maps the discovered ID back to the CC Switch role key.
- Windows Claude Desktop localization is an optional local patch surface. It writes only Claude Desktop resource/config files, keeps backups under Claude `resources\.zh-cn-backups`, and must preserve a recovery path before changing frontend bundles, `app.asar`, or `Claude.exe`.

## Runtime Entry Points
- Tauri lifecycle and commands: `src-tauri/src/lib.rs`.
- Claude Desktop localization install/status/restore: `src-tauri/src/claude_zh.rs`.
- Claude Desktop simplified Chinese visible-copy overrides: `src-tauri/resources/claude-zh/frontend-visible-overrides-zh-CN.json`.
- Claude Desktop visible-copy audit: `npm run audit:claude-zh`.
- Proxy lifecycle and CC Switch config field reads: `src-tauri/src/server.rs`.
- HTTP gateway routes and model rewrite: `src-tauri/src/proxy.rs`.
- CC Switch DB mapping read: `src-tauri/src/ccswitch_db.rs`.
- Claude Desktop configLibrary write/revert: `src-tauri/src/cd_config.rs`.

## Rollback
- In the app, use the revert command to set Claude Desktop `appliedId` back to CC Switch's `00000000-0000-4000-8000-000000157210` entry.
- For Claude Desktop localization, use the app's restore action to copy the latest `.zh-cn-backups` set back, remove Chinese language resources, set locale to `en-US`, and restart Claude.
- For code rollback, revert the latest local commit; do not rewrite shared history or force push.
