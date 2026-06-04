# Claude++ Project Map

## Purpose
`Claude++` is a local Tauri gateway between Claude Desktop 3P and CC Switch. By default Claude Desktop points to `127.0.0.1:15722/claude-desktop`; the gateway reads CC Switch's current claude-desktop provider mapping from the CC Switch SQLite database and forwards requests to CC Switch.

## Stable Boundaries
- Read-only source of model routes: `%USERPROFILE%\.cc-switch\cc-switch.db`, table `providers`, current row `app_type='claude-desktop' AND is_current=1`.
- Read-only weak fallback for Claude Desktop token usage: `%USERPROFILE%\.cc-switch\cc-switch.db`, table `proxy_request_logs`, successful `app_type='claude-desktop' AND data_source='proxy'` rows. The page enhancement passes the current turn start time as `sinceMs`; the local gateway reads only the latest matching row after that timestamp and marks it `source='cc-switch'`. Do not aggregate CC Switch rows by time range, because the same table includes history-load, title/status, and other background requests. Page-captured usage wins over this fallback, and finalized token cards must not be refreshed by later CC Switch polling.
- CC Switch route switch status is read from the CC Switch SQLite `proxy_config` table, not inferred from model mappings and not from stale `settings.json.enableLocalProxy`. When `proxy_config.enabled` exists, it is the CC Switch UI total route switch; `proxy_enabled` is only a legacy-schema fallback and must not be used alone on current schemas. Do not hardcode or display a fixed upstream route address in the status UI; users may configure different upstream addresses.
- Claude Desktop integration writes a separate `Claude++` configLibrary entry and must not edit CC Switch's `00000000-0000-4000-8000-000000157210` entry.
- The `Claude++` configLibrary entry must omit `inferenceModels` so Claude Desktop uses `/v1/models` discovery.
- The proxy must stay running while Claude Desktop is configured to use `Claude++`; otherwise Claude Desktop cannot load model discovery.
- The default proxy port is `15722`. Runtime port overrides are read from `CLAUDE_PLUS_PROXY_PORT` first, then `%USERPROFILE%\.claude-plus-plus\settings.json` (`proxyPort` or `proxy_port`), then the default.
- Claude Desktop discovers gateway models only during app startup in the observed Windows Store build. `Claude++` still sends no-cache headers on `/v1/models` and refreshes its own Claude Desktop configLibrary entry when CC Switch mappings change so credentials stay current, but the Claude Desktop model picker requires a Claude Desktop restart to show a changed model list.
- Model discovery IDs must be unique by role, not by CC Switch display label, because multiple roles can share the same labelOverride. Discovery display names must preserve the CC Switch menu display label exactly; duplicate labels are user configuration and should not be rewritten by Claude++. Request forwarding maps the generated internal ID back to the CC Switch role key and keeps legacy role-prefixed names compatible.
- Windows Claude Desktop localization is an optional local patch surface. It writes only Claude Desktop resource/config files, keeps backups under Claude `resources\.zh-cn-backups`, and must preserve a recovery path before changing frontend bundles, `app.asar`, or `Claude.exe`. Localization restore must remove only localization-owned language files, language-list entries, display-name patch, and locale state; it must not restore whole shared frontend or ASAR files from `.zh-cn-backups` because page enhancement scripts share those files.
- Claude Desktop install discovery must stay generic for open-source users. Discovery reads Claude++ settings overrides (`claudeDesktopPath` / `claudeDesktopResourcesPath`), running `Claude.exe` process paths, uninstall registry entries, Start Menu shortcuts, WindowsApps packages, Program Files, and `%LOCALAPPDATA%\Programs\Claude`, then validates candidates by the real `resources` layout before enabling localization or page enhancement.
- Welcome page developer mode enablement mirrors Claude Desktop's own behavior: write `allowDevTools: true` to `%APPDATA%\Claude\developer_settings.json`, preserve other JSON fields, keep a `.bak-*` sibling backup for existing files, verify by reading back, and restart Claude Desktop only when it was already running so the cached setting reloads.
- Welcome page Claude Code status checks for a local `claude` command. The install action launches a visible platform shell with the official Anthropic install command instead of running it silently in the background.

## Runtime Entry Points
- Tauri lifecycle and commands: `src-tauri/src/lib.rs`.
- Claude Desktop localization install/status/restore: `src-tauri/src/claude_zh.rs`.
- Claude Desktop simplified Chinese visible-copy overrides: `src-tauri/resources/claude-zh/frontend-visible-overrides-zh-CN.json`.
- Claude Desktop visible-copy audit: `npm run audit:claude-zh`.
- Proxy lifecycle and CC Switch config field reads: `src-tauri/src/server.rs`.
- Runtime settings, including proxy port resolution: `src-tauri/src/settings.rs`.
- Welcome page environment checks, Claude Code command install launcher, and Claude Desktop developer mode enablement: `src-tauri/src/welcome.rs`.
- HTTP gateway routes and model rewrite: `src-tauri/src/proxy.rs`.
- CC Switch DB mapping and token usage reads: `src-tauri/src/ccswitch_db.rs`.
- Claude Desktop configLibrary write/revert: `src-tauri/src/cd_config.rs`.
- Claude local skills discovery and recycle-bin deletion: `src-tauri/src/claude_skills.rs`; exposed through `/claude-plus/skills` and `/claude-plus/skills/:id/trash` on the local gateway for Claude++ app-side compatibility. The injected Claude Desktop skills popup must not depend on the Claude++ process after installation; it uses a preload bridge in Claude Desktop `app.asar` to scan local global/project skills and call Electron `shell.trashItem`.
- Claude Desktop page enhance feature definitions: `src/shared/enhance-features.json`; consumed by both the Tauri enhance status code and the React preview so labels/descriptions/order/version have a single source. Enhance status reads installed marker versions and upgrades only previously enabled outdated features to the current bundled version.

## Validation Entry Points
- Rust checks on Windows require the MSVC environment. Run from `src-tauri` with `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo check'`.
- Full Rust unit coverage for the local library: run the same MSVC wrapper with `cargo test --lib`.
- Frontend build and TypeScript check: `npm run build` from the repo root.

## Rollback
- In the app, use the revert command to set Claude Desktop `appliedId` back to CC Switch's `00000000-0000-4000-8000-000000157210` entry.
- For Claude Desktop localization, use the app's restore action to remove Chinese language resources, remove Chinese language-list entries, set locale to `en-US`, and restart Claude while preserving page enhancement scripts. `.zh-cn-backups` remains a safety backup for install-time file changes, not the normal restore mechanism for shared frontend files.
- For code rollback, revert the latest local commit; do not rewrite shared history or force push.
