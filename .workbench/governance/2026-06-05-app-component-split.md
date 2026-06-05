# App Component Split

## Scope
- Authorized item: split the large React `App.tsx` page/display components into separate files.
- Non-change constraints: keep UI copy, CSS classes, route behavior, command calls, polling intervals, and runtime state semantics unchanged.

## Change
- `src/App.tsx` now keeps the app shell, state, command orchestration, routing, and polling.
- Page components moved to `src/pages/`.
- Shared presentational components moved to `src/components/`.

## Validation
- `npm run typecheck`
- `npm run build`
- `git diff --check`

## Rollback
- Revert the local commit for this change. No external Claude Desktop, CC Switch, gateway, or settings state is modified by this refactor.
