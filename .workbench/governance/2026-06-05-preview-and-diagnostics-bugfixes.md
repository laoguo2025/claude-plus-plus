# Preview and Diagnostics Bugfixes

## Scope
User-reported potential bugs:
- `refreshAll` depended on `refreshDiagnostics` but did not call it.
- Preview noop commands returned `undefined as T`.
- Diagnostics log line keys included content slices.
- Preview command state used independent `if` checks for mutually exclusive commands.
- `/claude-desktop/*` missing-Origin security was reviewed and already covered by commit `7319b04`.

## Change
- Removed the unused `refreshDiagnostics` dependency from `refreshAll`.
- Changed diagnostics log rows to use stable row-number keys.
- Added a `PreviewNoopCommand` type guard/overload and made preview noop commands return plain `void`.
- Replaced preview state update `if` chain with a `switch`.
- Added `.workbench/tools/audit-preview-command-contract.cjs` to prevent reintroducing `undefined as T` and non-exclusive preview state branches.

## Validation
- `node .workbench/tools/audit-preview-command-contract.cjs`: failed before implementation, passed after.
- `npm run typecheck`: passed.
- `npm run build`: passed.
- `npm run test:rust`: passed, 149 passed, 5 ignored.

## Rollback
Revert the local commit. No external Claude Desktop, CC Switch, or Claude Code state was written during this task.
