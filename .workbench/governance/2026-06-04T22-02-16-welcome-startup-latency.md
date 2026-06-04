# Welcome Startup Latency

## Reason
The welcome page could show a Windows "not responding" title and delayed status cards on startup. Users mainly need the four environment cards quickly, while the app was also starting heavier status checks.

## Root Cause
Startup called welcome, localization, route mapping, and enhancement status refreshes together. Localization and enhancement status can scan Claude Desktop resources, frontend bundles, and app.asar files; route state can read CC Switch mappings. The welcome cards also treated missing status as false, so pending checks appeared as "not installed" before completion.

## Change
- Startup now runs only lightweight `welcome_status` and app version checks.
- Route, localization, and enhancement status refreshes run when their pages are opened or explicitly refreshed there.
- Welcome cards show "检测中" while `welcome_status` is pending.
- `welcome_status` now includes a lightweight Claude Desktop presence flag.
- Claude Code presence checks PATH entries directly instead of spawning `where claude`.

## Validation
- `npm run build` passed.
- `cargo test welcome --lib` passed.
- `cargo test --lib` passed.
- `git diff --check` passed.

## Rollback
Revert the local commit. No external files or user machine settings are changed by this optimization.
