# CC Switch gateway profile atomic read

## Context
The P1 review found that Claude++ read CC Switch `inferenceGatewayApiKey` and `inferenceGatewayBaseUrl` through two independent scans of Claude Desktop `configLibrary` candidates. If different candidate libraries had different mtimes or partial entries, the key could come from one `157210` entry while the base URL came from another.

## Change
- Added a single `CcSwitchGatewayProfile` read path for the CC Switch `157210` entry.
- A candidate profile is accepted only when the same entry file contains both non-empty `inferenceGatewayApiKey` and `inferenceGatewayBaseUrl`.
- Title translation, proxy upstream selection, mapping-refresh fingerprinting, and Claude++ config refresh now use the unified profile path.
- Removed the old split key/baseUrl getter entry points to avoid new call sites reintroducing mixed reads.

## Validation
- Added a regression test proving orphan key-only/baseUrl-only entries are not cross-paired.
- `npm run typecheck`: passed.
- `cmd.exe /c build.bat test --lib`: passed, 124 passed / 5 ignored.
- `cmd.exe /c build.bat fmt --check`: still fails on the pre-existing `src-tauri/src/lib.rs` import ordering diff; this unrelated hunk was not included in the final change.

## Rollback
Revert the local commit for source and project-map changes. The app rollback path remains reverting the latest local commit; no external Claude Desktop resources are written by this source-only change.
