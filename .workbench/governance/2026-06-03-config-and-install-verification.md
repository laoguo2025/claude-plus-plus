# 2026-06-03 Config And Install Verification

## Scope
User approved fixing the next 1-7 batch:
1. Runtime-configurable proxy port.
2. Real Claude Desktop page-enhance install verification.
3. Browser preview mock shrink.
4. Scaffold favicon cleanup.
5. Mapping monitor debounce and diagnostics.
6. Cross-platform compile boundary check.
7. Workbench `reademe.md` handling.

## Changes
- Added `src-tauri/src/settings.rs`; proxy port resolves from `CLAUDE_PLUS_PROXY_PORT`, then `%USERPROFILE%\.claude-plus-plus\settings.json` (`proxyPort` or `proxy_port`), then default `15722`.
- Routed proxy startup, status, Claude Desktop config apply/refresh, self-recovery, and enhance bridge URL generation through the runtime port.
- Added exact-port health checks so a live old port is not treated as healthy after a configured-port change.
- Added mapping refresh debounce, duplicate-refresh suppression, and diagnostics events for refresh ok/fail/skip.
- Reduced browser preview mock data to shared constants and removed repeated hardcoded preview status objects.
- Replaced the Vite scaffold favicon with the existing Claude++ PNG icon and removed `public/vite.svg`.
- Added canonical `.workbench/readme.md`; kept `.workbench/reademe.md` as a compatibility pointer because current project instructions still mention the old path.
- Aligned non-Windows enhance feature field types with the Windows status shape.
- Added a real-install verification test for the token usage enhancement bridge.

## Validation
- `npm run build` passed.
- `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo check'` passed.
- `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo test --lib'` passed: 47 passed, 4 ignored.
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_plugins_enhance_writes_skills_bridges --lib -- --ignored --nocapture` passed.
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_title_i18n_enhance_writes_bridge --lib -- --ignored --nocapture` passed.
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_token_usage_enhance_writes_bridge --lib -- --ignored --nocapture` passed.
- `rustup target list --installed` shows only `x86_64-pc-windows-msvc`; non-Windows target compilation could not be run locally in this environment.

## External Effects
- The three ignored install-verification tests wrote current page-enhance assets into the local Claude Desktop resource bundle through the formal install path. They verified the installed feature status, but Claude Desktop may still need a restart for a running app instance to load those resources.

## Rollback
- Revert this commit for source changes.
- For installed Claude Desktop page-enhance resources, use Claude++ page-enhance uninstall/restore paths or restore the latest `.claude-plus-enhance-backups` set.
