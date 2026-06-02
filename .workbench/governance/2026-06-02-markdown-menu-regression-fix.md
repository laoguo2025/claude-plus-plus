# Markdown Menu Regression Fix

## Reason
The Markdown export menu injection affected sidebar title translation and duplicated the conversation dropdown menu. The conversation title i18n card also needed the requested token usage notice.

## Scope
- Excluded menu and popover surfaces from conversation title translation scans.
- Narrowed Markdown export menu injection to real menu items only.
- Removed broad `div` scanning and duplicate inserted menu items.
- Added the `会消耗少量 token` notice on the conversation title i18n card.

## Boundaries
- Markdown export still uses the currently loaded and rendered conversation DOM.
- Timeline behavior was not changed.
- No new backend route, preload bridge, or local storage reader was added.

## Validation
- `npm run build` passed.
- `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 && cargo test claude_enhance::imp::tests --manifest-path src-tauri\Cargo.toml --lib'` passed: 23 passed, 2 ignored.

## Rollback
- Use the page enhancement cancel action for `markdown` or `conversation_title_i18n` to remove the installed feature markers.
- Revert the local commit for code rollback.
