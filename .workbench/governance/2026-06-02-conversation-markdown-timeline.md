# Conversation Markdown Export And Timeline

## Reason
The page enhancement list already exposed `markdown` and `timeline`, but install and uninstall rejected both features as not connected. The requested enhancement is to make both features installable while keeping the current Claude Desktop patch and rollback model.

## Scope
- Added renderer-side Markdown export for the current loaded conversation DOM.
- Added renderer-side question timeline for currently loaded user messages.
- Removed the install/uninstall blockers for `markdown` and `timeline`.
- Updated UI preview/status notes to state the loaded-content boundary.

## Boundaries
- No new backend route, preload bridge, filesystem permission, or Claude local storage reader was added.
- Markdown export is intentionally limited to messages already loaded and rendered in the page.
- Timeline markers are generated only from currently rendered user messages.

## Validation
- `npm run build` passed.
- `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 && cargo test claude_enhance::imp::tests --manifest-path src-tauri\Cargo.toml --lib'` passed: 22 passed, 2 ignored.

## Rollback
- Use the page enhancement cancel action for `markdown` or `timeline` to remove the feature marker.
- Restore the latest Claude Desktop resources backup if a real install needs full resource rollback.
- Revert the local commit for code rollback.
