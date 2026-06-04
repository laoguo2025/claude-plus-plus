# CC Switch Route Status Split

## Scope
Updated CCS transfer status semantics after confirming CC Switch separates `proxy_config.enabled` and `proxy_config.proxy_enabled`.

## Change
- `app_type='claude'.enabled` is now treated as the Claude route/app takeover switch.
- `proxy_enabled` is now treated as the CC Switch route service master switch and reachability gate.
- The overview route cards now show Claude route switch, CC Switch route master switch, and Claude++ takeover.
- Model provider configuration status moved into the current provider/model mapping panel.

## Validation
- `cmd.exe /c build.bat test --lib ccswitch_db::tests::proxy_config_keeps_claude_route_separate_when_proxy_enabled_is_off`
- `npm run typecheck`
- `cmd.exe /c build.bat test --lib`
- `npm run build`

## Rollback
Revert the scoped commit for this change.
