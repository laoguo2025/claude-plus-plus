# TokenUsage Badge Fix

## Root Cause
- Generic `cached_tokens` / `cachedTokens` was treated as cache-read input, so snapshots where `cached_tokens == input_tokens` rendered a false `100.0%` cache hit rate.
- The badge context fallback rendered raw token count when no context limit was known, which made the context format inconsistent with the requested `context ratio` field.
- The badge was inserted relative to the broad assistant container instead of the assistant footer/action row, leaving it too low in the conversation UI.

## Change
- Cache-read tokens now only trust explicit cache-read fields: `cache_read_input_tokens`, `cacheReadInputTokens`, and nested `prompt/input_tokens_details.cached_tokens` variants.
- Unknown context limit now renders `上下文占比 未知`; known limits render `上下文占比 used/limit (pct)`.
- Badge mounting now inserts after the detected assistant footer/action row when present.

## Verification
- `cargo test token_usage_ --lib`
- `cargo test --lib`
- `rustfmt --edition 2024 --check src\proxy.rs src\claude_enhance.rs`
- `npm run build`

## Rollback
- Revert the local commit containing this change and reinstall/apply the previous Claude Desktop page enhance resources.
