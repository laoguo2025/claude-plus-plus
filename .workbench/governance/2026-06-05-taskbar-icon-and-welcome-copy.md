# Taskbar Icon And Welcome Copy

## Scope
- Fix the Windows taskbar icon for the Claude++ Tauri main window.
- Preserve the welcome copy/layout adjustments requested in the same UI round.

## Cause
- The bundled executable already embeds `src-tauri/icons/icon.ico`, but Tauri `WebviewWindow::set_icon` maps to the small window icon path on Windows.
- The Windows taskbar uses the big icon (`ICON_BIG`) and may continue showing the default icon unless it is set explicitly.

## Change
- Added a Windows-only startup helper that loads the embedded application icon resource and writes it to `ICON_SMALL`, `ICON_BIG`, `GCLP_HICONSM`, and `GCLP_HICON`.
- Added the explicit `windows-sys` dependency needed for the Win32 API calls.
- Kept the welcome text shift and centered download hints.

## Validation
- `cargo check` in the MSVC environment passed.
- `npm run typecheck` passed.
- `npm run build` passed.
- `npm run tauri -- build` passed and generated the x64 NSIS installer.
- Runtime probe of the built exe reported `ICON_BIG`, `ICON_SMALL`, class big icon, and class small icon all set.

## Rollback
- Remove the Windows helper and its call from `src-tauri/src/lib.rs`.
- Remove the direct `windows-sys` dependency from `src-tauri/Cargo.toml`.
- Rebuild and copy release artifacts again.
