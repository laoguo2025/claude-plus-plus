# Proxy Runtime Tuning

## Context

The review identified hardcoded proxy tuning values in `proxy.rs` and noted that title translation could fall back to the first mapping, including a haiku route, when no sonnet mapping existed.

## Changes

- Added `ProxyRuntimeTuning` in `settings.rs`.
- `settings.json` now supports camelCase and snake_case keys for title translation rate limit window, title translation rate limit max, token usage pending-line size, and token usage DB freshness window.
- Kept existing default values: 60 seconds, 90 requests, 64 KiB pending line, and 15000 ms DB freshness.
- Routed title i18n rate limiting and token usage freshness through the tuning config.
- Title translation model selection now prefers sonnet, then opus, and returns no model when only haiku mappings are available.

## Validation

- `cargo test --lib` failed after adding tests and before implementing the tuning parser/freshness signature.
- `cargo test --lib proxy_runtime_tuning` passed.
- `cargo test --lib title_translation_model_prefers_sonnet_then_opus_and_skips_haiku_only` passed.
- `cargo test --lib token_usage_requires_fresh_snapshot_for_since_query` passed.
- `cargo test --lib` passed: 96 passed, 0 failed, 5 ignored.
- `rustfmt --edition 2024 --check src-tauri\src\settings.rs src-tauri\src\proxy.rs` passed.

## Rollback

Revert the local commit for this slice. Request body rewrite behavior, upstream fallback behavior, and token usage parsing semantics were not changed.
