# 2026-05-31 Claude Desktop Model Cache

## Symptom
After switching the claude-desktop provider in CC Switch, `ccs2claude` immediately returned the new model list, but Claude Desktop kept showing the previous model list.

## Evidence
- `GET http://127.0.0.1:15722/claude-desktop/v1/models?limit=1000` returned the current CC Switch provider models.
- Claude Desktop configLibrary `_meta.json` still had `appliedId` set to the `ccs2claude` entry.
- Claude Desktop had active local TCP connections to `ccs2claude`.
- The stale behavior was therefore on the Claude Desktop discovery/cache side, not in CC Switch DB reading.

## Change Rationale
- Add no-cache headers to `/v1/models` responses.
- Add a background mapping monitor that fingerprints the current CC Switch claude-desktop provider mappings.
- On startup and whenever the fingerprint changes, refresh the `ccs2claude` Claude Desktop configLibrary entry and `_meta.json` to update mtimes and trigger Claude Desktop's config watcher.
- Read CC Switch-generated gateway fields from the newest `157210` entry across known Claude-3p config paths, because provider switching can update a different path than the Store runtime path.

## Verification Result
- `cmd /c build.bat check` passed.
- `cmd /c build-release.bat` passed after stopping the previously running executable that locked the release binary.
- Starting the new release executable updated the Store configLibrary `ccs2claude` entry and `_meta.json` mtimes.
- `GET /claude-desktop/v1/models?limit=1000` returned the current model list with `Cache-Control`, `Pragma`, and `Expires` no-cache headers.
- Full live validation of a new CC Switch provider switch still requires a real switch action in CC Switch while Claude Desktop is open.

## Rollback
Use the app's revert action to switch Claude Desktop back to CC Switch, then revert the local commit if this behavior must be removed.
