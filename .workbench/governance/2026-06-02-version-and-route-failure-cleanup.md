# 2026-06-02 version and route failure cleanup

## Basis
- User approved the third batch of fixes from the documented review.
- Scope is limited to Claude++ route partial-failure cleanup, app version source, and stable backend constants.

## Changes
- `use_claude_plus_route` stops the proxy after config apply failure only when this call started it.
- Added `app_version` Tauri command backed by `env!("CARGO_PKG_VERSION")`.
- About loads the backend version in Tauri runtime; browser preview diagnostics use the Vite-injected package version fallback.
- Moved Claude++ config entry constants and upstream fallback URL into `constants.rs`.

## Non-changes
- Did not alter configured UUIDs, displayed app name, or fallback upstream URL values.
- Did not modify injected Claude Desktop enhancement scripts.
- Did not change About page layout or update-check placeholders.

## Rollback
- Revert this local commit; no external writes or push are required.

## Verification
- `npm run build` passed.
- `cargo fmt --check` passed.
- `git diff --check` passed.
- `cargo check` could not reach project compilation because this shell cannot find MSVC `cl.exe`; native dependency build scripts for `libsqlite3-sys`, `vswhom-sys`, and `ring` fail first.
