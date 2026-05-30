# 2026-05-31 Claude Desktop Restart Required

## Finding
After CC Switch provider or model changes, `ccs2claude` updates `/claude-desktop/v1/models` immediately, but Claude Desktop does not re-run model discovery while the app is already running.

## Evidence
- Direct requests to `http://127.0.0.1:15722/claude-desktop/v1/models?limit=1000` returned the current role-unique models.
- Claude Desktop `main.log` showed `[custom-3p] Model discovery` only during app startup.
- Claude Desktop packaged code creates model discovery as a startup-time promise and caches the enterprise config in process memory.

## Change
- Added an app command and UI button to restart Claude Desktop.
- Added an in-app notice that CC Switch changes are synced in `ccs2claude`, but Claude Desktop must restart to refresh the model picker.
- Kept the background config entry rewrite for API key/config freshness, but clarified logs and docs so it is not treated as a live picker refresh.

## Rollback
Revert the local commit for this slice to remove the restart command and UI notice. The gateway and configLibrary integration can still be reverted through the app's existing revert action.
