# Token Usage Cache Known Fix

## Reason
The Claude Desktop token usage badge could show cache creation/read as `0` and a misleading hit rate when upstream usage did not include cache fields. The page script could not distinguish an explicit zero from a missing field, and the local proxy serialized default zeros without field-presence metadata.

## Change
- Added proxy-side `cacheReadKnown` and `cacheCreationKnown` metadata to token usage snapshots.
- Kept `cached_input_tokens` excluded from cache hit-rate calculation, while allowing proxy `cachedTokens` only when `cacheReadKnown` is true.
- Updated the badge copy to show cache creation/read as `未知` when the source did not provide cache fields.
- Tightened badge placement/render timing for final turn rendering and bumped the token usage enhance version to `v0.3` so enabled old installs are recognized as upgrade candidates.

## Validation
- `cargo test token_usage_ --lib`
- `cargo test --lib`
- `rustfmt --edition 2024 --check src\claude_enhance.rs src\proxy.rs`
- `npm run build`

## Rollback
Revert the local commit that contains this governance note and the related source changes. Claude Desktop installed resources were not modified in this step.
