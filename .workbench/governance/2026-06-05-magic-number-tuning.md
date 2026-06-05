# Magic Number Tuning

## Scope
- Replaced frontend polling intervals and backend gateway thresholds with named constants.
- Made token usage injected capture limits configurable through `settings.json`.
- Made the frontend default proxy port derive from the Rust constant at Vite config time.

## Non-change Constraints
- Default timings, limits, title translation behavior, token usage scoring, and injected script behavior remain unchanged when no settings file overrides are present.
- Runtime proxy port precedence remains environment, settings file, then Rust default.

## Validation
- `npm run test:rust`: 154 passed, 5 ignored.
- `npm run typecheck`: passed.
- `npm run build`: passed.

## Rollback
- Revert the local commit for this change set. No external state migration is required because new settings keys are optional.
