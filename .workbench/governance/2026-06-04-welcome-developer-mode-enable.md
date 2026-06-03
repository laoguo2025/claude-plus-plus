# 2026-06-04 Welcome Developer Mode Enable

## Scope
- Add a Welcome page action for enabling Claude Desktop developer mode.
- Keep the existing status detection and add a write path for `developer_settings.json`.
- Restart Claude Desktop only if it was already running when the action was triggered.

## Evidence
- Claude Desktop 1.4758 reads `app.getPath("userData")/developer_settings.json`.
- Its developer settings schema includes `allowDevTools`.
- Its own Enable Developer Mode action sets `allowDevTools=true`, writes the JSON with a private-file helper, then relaunches/quits the app.
- On this machine the live path is `%APPDATA%\Claude\developer_settings.json`.

## Validation
- Pending for this slice: `cargo test --lib`, `npm run build`, and `git diff --check`.

## Rollback
Revert the local commit for this slice. If a user has already clicked the new action, restore the generated `developer_settings.json.bak-*` sibling backup or set `allowDevTools` back to `false`, then restart Claude Desktop.
