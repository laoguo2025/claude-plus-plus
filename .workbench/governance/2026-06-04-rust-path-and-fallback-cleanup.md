# Rust Path And Fallback Cleanup

## Context

The review identified repeated home-directory resolution across Rust modules and an obsolete `"Role - display"` model-name fallback in proxy mapping. The upstream URL fallback was intentionally left unchanged for a later request-path slice because it affects live proxy failure behavior.

## Changes

- Added `src-tauri/src/paths.rs` for shared `home_dir`, app state directory, and CC Switch database path helpers.
- Reused shared path helpers from CC Switch DB path resolution, settings path resolution, and diagnostics log path resolution.
- Removed the obsolete `"Opus - mimo-v2.5-pro"` display-name fallback from model-to-role mapping.
- Kept current model ID matching, raw CC Switch display matching, and role-kind token fallback.

## Validation

- `cargo test --lib unknown_model_names_fallback_by_role_kind` failed before removing the legacy fallback and passed after the fix.
- `cargo test --lib paths::tests` passed.
- `rustfmt --edition 2024 --check` passed for the Rust files touched in this slice.
- `cargo test --lib` passed: 93 passed, 0 failed, 5 ignored.

## Rollback

Revert the local commit for this slice. No upstream URL fallback or proxy request forwarding behavior was changed.
