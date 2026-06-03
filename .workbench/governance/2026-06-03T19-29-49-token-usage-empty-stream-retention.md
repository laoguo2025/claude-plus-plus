# Token Usage Empty Stream Retention

## Reason
The token usage badge could remain absent even with the v0.3 script installed because `/claude-plus/token-usage` returned `usage: null`. The proxy cleared the stored usage at the start of every proxied stream; a later stream without usage metadata could erase the last valid snapshot.

## Change
- Keep the previous token usage snapshot when a proxied stream has no usage metadata.
- Publish new usage only when a stream produces a valid usage snapshot.
- Add endpoint diagnostics: `pending`, `lastEmptyAtMs`, and `lastError`.
- Reset TokenUsage enhance definition back to `v0.1` because the feature is not released yet.

## Validation
- Added regression coverage for empty streams preserving previous usage.
- Targeted token usage tests passed.

## Rollback
Revert the local commit containing this note and the related proxy/config changes.
