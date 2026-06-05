# Robustness Hardening

## Scope
- Removed panic paths from ASAR integer reads, embedded enhancement feature definition parsing, and bridge patch UTF-8 conversion.
- Removed an unnecessary proxy request body clone.
- Split manager status from diagnostics status so routine UI polling avoids CC Switch DB and TCP reachability checks.

## Non-change Constraints
- Diagnostics still performs the complete CC Switch DB and TCP reachability checks.
- Proxy request rewriting and bridge patch output remain unchanged for valid input.

## Validation
- `npm run test:rust`: 156 passed, 5 ignored.
- `npm run typecheck`: passed.

## Rollback
- Revert the local commit for this change set. No external state migration is required.
