# 2026-06-02 doc review targeted fixes

## Basis
- User approved fixing the confirmed documentation-review issues in a narrow scope.
- Verified the attached review report against the current tree before editing.

## Changes
- Fixed non-Windows `claude_enhance::install` and `uninstall` signatures so they match the command callers.
- Reused the frontend preview enhance feature list from one helper, including `token_usage`.
- Added shared backend constants for the default proxy port, CC Switch Claude Desktop entry id, and Claude Store package/app ids.
- Added a shared `now_ms` helper for diagnostics and proxy token usage timestamps.

## Non-changes
- Did not refactor the large duplicated ASAR/backup helper blocks in `claude_zh.rs` and `claude_enhance.rs`.
- Did not change compressed injected JavaScript gateway URLs or runtime behavior.
- Did not rename `.workbench/reademe.md` because it is the current documented navigation entry.

## Rollback
- Revert this local commit; no external writes or push are required.

## Verification
- `npm run build` passed.
- `cargo fmt --check` passed after formatting.
- `cargo check` could not reach project compilation because this shell cannot find MSVC `cl.exe`; native dependency build scripts for `libsqlite3-sys`, `vswhom-sys`, and `ring` fail before crate code is checked.
- Confirmed only `x86_64-pc-windows-msvc` Rust target is installed in this environment; `gcc` exists but `cl.exe` does not.
