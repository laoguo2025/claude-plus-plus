# Topbar Global Actions

## Reason
Every Claude++ page needed Codex++-style top-right quick actions: theme toggle, restart Claude++, and refresh current page.

## Scope
- Added a global topbar action group on all pages.
- Added quick theme toggle using the existing `claude-plus-theme` storage.
- Added a `restart_claude_plus` Tauri command that relaunches the current executable.
- Kept the existing Claude Desktop restart workflow unchanged.

## Boundaries
- This restarts the Claude++ management tool, not Claude Desktop.
- No page enhancement injection behavior was changed.
- Existing About-page theme controls remain available.

## Validation
- `npm run build` passed.
- `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 && cargo test --manifest-path src-tauri\Cargo.toml --lib'` passed: 36 passed, 3 ignored.

## Rollback
- Remove the topbar action group, the `restart_claude_plus` command, and its invoke registration.
- Revert the local commit for code rollback.
