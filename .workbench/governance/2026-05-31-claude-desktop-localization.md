# Claude Desktop localization slice

## Context
User approved adding a one-click localization feature to `ccs2claude` based on `D:\claude-desktop-zh-cn-1.1.0`, with the reference package improved because the existing Windows install path copied a fixed translation pack and could miss current-version keys.

## Change reason
`ccs2claude` already had a disabled "one-click localization" placeholder. The new feature embeds the reference Chinese resources and adds app-level install/status/restore commands so users can localize Claude Desktop without running the external script directly.

## Non-change constraints
- CC Switch provider data and `ccs2claude` proxy routing are not changed.
- Claude Desktop 3P config apply/revert behavior is not changed.
- The localization flow is Windows-only in this slice.
- Install must create backups before changing Claude frontend bundles, `app.asar`, or `Claude.exe`.

## Verification plan
- `npm run build`
- `.\build.bat check`
- `.\build.bat test`
- No live install is performed unless explicitly requested; read-only status detection is acceptable for this slice.

## Rollback
- App-level restore uses the latest Claude `resources\.zh-cn-backups` set, removes Chinese language files, writes `en-US` locale, and restarts Claude.
- Code rollback is a normal local commit revert; do not rewrite history.
