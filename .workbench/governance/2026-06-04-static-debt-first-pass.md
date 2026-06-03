# Static Debt First Pass

## Context

The review identified small static-debt items around duplicated Windows hidden command helpers, redundant CC Switch install detection, preview default proxy port duplication, and coexisting `.workbench/reademe.md` / `.workbench/readme.md` entries.

## Changes

- Reused the shared Windows hidden command helper where no caller-specific output behavior was required.
- Simplified welcome-page CC Switch install detection to file/directory markers instead of re-reading proxy config after checking the same DB path.
- Moved the preview default proxy port to a Vite build constant so browser preview code no longer owns a separate literal.
- Made `.workbench/readme.md` the canonical navigation entry and kept `.workbench/reademe.md` as a compatibility pointer.

## Validation

- `npm run build` passed.
- `cargo test --lib cc_switch_install_marker_accepts_db_file_or_state_dir` failed before the helper implementation and passed after the fix.
- `cargo test --lib` passed: 91 passed, 0 failed, 5 ignored.
- Full `cargo fmt --check` still reports pre-existing formatting changes in `src-tauri/src/claude_enhance.rs` and `src-tauri/src/lib.rs`, which were outside this slice.

## Rollback

Revert the local commit for this slice. The misspelled workbench compatibility entry can remain harmlessly if only code changes need rollback.
