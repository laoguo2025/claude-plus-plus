# UI Freeze Diagnostics

## Context
User reported Claude++ page switches causing brief Windows "not responding" freezes, and diagnostics "generate report" causing a 3-4 second freeze.

## Root Cause
Tauri commands for status and diagnostics ran blocking filesystem, PowerShell, database, TCP, bundle, and ASAR checks synchronously on the command path. The diagnostics command also chained several status collectors in one call. Enhancement status reads also attempted automatic feature-version migration, which made a status read capable of stopping Claude Desktop and writing external resources.

## Change
- Moved heavy status/report/log commands to `spawn_blocking` while keeping existing command names.
- Kept enhancement status reads read-only by removing automatic migration from `claude_enhance::status`.
- Added regression coverage that outdated enhancement payloads are reported without being upgraded during status reads.
- Added a diagnostics button busy state to prevent repeated report generation clicks and show progress.

## Verification
- `npm run typecheck`
- `npm run build`
- `npm run check:rust`
- `npm run test:rust`
- Targeted Rust tests for status-read-only behavior and explicit marker migration.

## Rollback
Revert the local commit for this slice. No external Claude Desktop or CC Switch state is required to roll back.
