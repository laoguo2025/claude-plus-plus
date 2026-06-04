# Claude Enhance Structure Split

## Scope
- Addressed the P3 structure debt concentrated in `src-tauri/src/claude_enhance.rs`.
- Kept behavior, feature IDs, markers, versions, injected strings, install/uninstall semantics, ASAR writes, local gateway token handling, and runtime port resolution unchanged.
- Did not write Claude Desktop resources.

## Change
- Kept the command/status state machine in `src-tauri/src/claude_enhance.rs`.
- Split page injected script generation into `src-tauri/src/claude_enhance/enhance_injected.rs`.
- Split main/preload bridge script generation into `src-tauri/src/claude_enhance/enhance_bridge_scripts.rs`.
- Split ASAR patch/read helpers into `src-tauri/src/claude_enhance/enhance_asar.rs`.
- Split bridge install/remove/status operations into `src-tauri/src/claude_enhance/enhance_bridge_ops.rs`.
- Moved the page-enhancement tests to `src-tauri/src/claude_enhance/imp/enhance_tests.rs` so the main file no longer carries the large test block.
- Updated the project map to keep future page-enhancement internals in the split module layout.

## Validation
- `cmd.exe /c build.bat test --lib`: passed, 126 passed and 5 real-resource verification tests ignored by existing guard.
- `npm run typecheck`: passed.
- `cmd.exe /c build.bat fmt --check`: command returned success; output still reports the existing `src-tauri/src/lib.rs` import-order diff, which this slice did not touch.
- `git diff --check`: passed.

## Rollback
- Revert the local commit for this source-only split.
- No external Claude Desktop resource rollback is needed because this slice does not install or modify page-enhancement resources.
