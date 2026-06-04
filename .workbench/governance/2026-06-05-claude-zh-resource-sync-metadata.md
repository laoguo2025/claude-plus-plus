# Claude Zh Resource Sync Metadata

## Scope
- Synced bundled simplified Chinese localization resources from `javaht/claude-desktop-zh-cn` release `1.2.0`, commit `8505555ef344df5a26a0a17c9d6fac2a7c235d93`.
- Added bundled resource metadata to localization status and the localization page.
- Did not write Claude Desktop, CC Switch, ASAR, or external configuration.

## Merge rule
Only imported upstream `zh-CN` entries when Claude++ was missing the key or still fell back to English. Existing Claude++ visible overrides and hardcoded translations stayed authoritative.

## Result
- `frontend-zh-CN.json`: added 29 upstream keys and replaced 41 local English fallbacks with upstream Chinese.
- `frontend-hardcoded-zh-CN.json`: appended 177 upstream-only hardcoded replacements.
- `desktop-zh-CN.json` and `statsig-zh-CN.json` matched upstream for current key coverage and were not changed.
- `metadata-zh-CN.json` now records source repository, commit, release, sync date, resource scope, and merge policy.

## Validation
- `cmd.exe /c build.bat test --lib status_exposes_bundled_resource_metadata` passed after the expected initial missing-field failure.
- `npm run audit:claude-zh` passed with `count: 0`.
- `npm run typecheck` passed.
- `cmd.exe /c build.bat test --lib claude_zh::imp::tests` passed: 7 passed, 1 ignored.

## Rollback
Revert the local commit for source, resource, and workbench documentation changes. No live Claude Desktop resources were modified during validation.
