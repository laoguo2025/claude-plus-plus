# Duplicate Code Shared Utils

## Scope
- Deduplicated TCP reachability checks, proxy port parsing, Claude Desktop developer settings candidates, local gateway token validation, and frontend `Icon` type ownership.
- No route behavior, token format, diagnostics schema, or UI copy changes were intended.

## Changes
- Added shared Rust utility modules for TCP/port helpers and developer settings path discovery.
- Kept local gateway token validation owned by the server module and reused it from diagnostics.
- Reused the exported frontend `Icon` type from `appConstants.ts`.
- Registered the new stable utility entry points in the project map.

## Validation
- `npm run test:rust`: 153 passed, 5 ignored.
- `npm run typecheck`: passed.
- `npm run build`: passed.

## Rollback
- Revert the local commit for this change set. This restores the previous per-file helper implementations without external state migration.
