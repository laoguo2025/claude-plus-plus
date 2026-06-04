# P2 Atomic External Writes

## Scope
- Fixed non-atomic production writes for Claude Desktop external config, resource bundles, `app.asar`, and `Claude.exe` integrity marker updates.
- Left test fixture writes and internal app-state writes outside this scope.

## Change
- Added a shared same-directory atomic write helper in `claude_patch_common`.
- Windows replacement uses `MoveFileExW` with replace-existing and write-through flags after flushing the temporary file.
- Switched configLibrary entry/meta writes, page enhancement bundle and ASAR writes, localization resource/config/ASAR writes, developer settings writes, and `Claude.exe` integrity marker writes to the helper.

## Validation
- Added unit coverage for replacing an existing file without leaving the helper temp file behind.
- `cmd.exe /c build.bat test --lib`: passed, 126 passed and 5 real-resource verification tests ignored by existing guard.
- `npm run typecheck`: passed.
- `cmd.exe /c build.bat fmt --check`: command returned success; output still reports the existing `src-tauri/src/lib.rs` import-order diff, which this slice did not touch.
- `git diff --check`: passed.

## Rollback
- Revert the local commit for code rollback.
- Existing runtime backups still cover Claude Desktop resources changed before install-time patch writes.
