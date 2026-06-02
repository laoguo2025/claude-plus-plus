# Markdown Export Menu Item

## Reason
The fixed Markdown export button worked, but it occupied the conversation viewport. The requested behavior is to expose export from each conversation's existing dropdown control list.

## Scope
- Removed the fixed Markdown export button behavior from the renderer injection.
- Added scanner logic for conversation dropdown menu surfaces.
- Injected a `导出 Markdown` menu item near archive/delete entries.

## Boundaries
- The export data source remains the current loaded and rendered conversation DOM.
- No backend route, preload bridge, or local storage reader was added.
- Timeline behavior was not changed.

## Validation
- `npm run build` passed.
- `cmd.exe /d /s /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 && cargo test claude_enhance::imp::tests --manifest-path src-tauri\Cargo.toml --lib'` passed: 23 passed, 2 ignored.

## Rollback
- Use the page enhancement cancel action for `markdown` to remove the feature marker.
- Revert the local commit for code rollback.
