# Skills bridge gateway-first scan

## Context
The P2 review found that Skills discovery was implemented twice: Rust owns `claude_skills::list_skills`, while the installed Claude Desktop main-process bridge embedded a second synchronous JavaScript scanner. The bridge ran `fs.readdirSync` / `fs.readFileSync` recursively inside `ipcMain.handle`, so large Claude project/session caches could block the Claude Desktop main process.

## Change
- Skills main-process bridge now calls the Claude++ local gateway first for list and trash operations.
- The gateway calls carry the local gateway token header and resolve the proxy port at bridge runtime.
- The old in-bridge filesystem scanner remains as a fallback when Claude++ is unavailable.
- Installing the `plugins` page enhancement now starts the local gateway after install succeeds.
- Bumped only the `plugins` enhancement definition to `v0.3` so installed Skills bridges can migrate.

## Validation
- Targeted bridge and feature-version tests passed.
- `cmd.exe /c build.bat test --lib`: passed, 125 passed / 5 ignored.
- `npm run typecheck`: passed.
- `cmd.exe /c build.bat fmt --check`: still fails on the pre-existing `src-tauri/src/lib.rs` import ordering diff; this unrelated hunk was not included in the final change.

## Rollback
Revert the local commit for source, feature-definition, and project-map changes. Existing installed Claude Desktop resources can be refreshed through the Claude++ page enhancement install/restore flow after rollback.
