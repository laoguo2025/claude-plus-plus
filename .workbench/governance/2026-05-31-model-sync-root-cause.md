# 2026-05-31 Model Sync Root Cause

## Symptom
After changing models or switching provider in CC Switch, Claude Desktop could not show models from `ccs2claude`.

## Evidence
- Claude Desktop configLibrary `_meta.json` had `appliedId` set to the `ccs2claude` entry.
- The `ccs2claude` entry pointed to `http://127.0.0.1:15722/claude-desktop` and omitted `inferenceModels`, so discovery mode was configured.
- No process was listening on `127.0.0.1:15722`; direct request to `/claude-desktop/v1/models?limit=1000` failed to connect.
- Starting the built `ccs2claude.exe` made `GET /claude-desktop/v1/models` return the current CC Switch model list.

## Root Cause
The discovery and DB mapping logic worked once the process was running. The failure was that Claude Desktop had been configured to use the gateway while the gateway process was not resident.

## Change Rationale
- Keep the app resident by hiding the window on close instead of exiting.
- Add tray controls so the hidden window can be restored and the process can be intentionally quit.
- Make proxy startup report bind failures before marking the server running, preventing false "running" status if the port cannot bind.

## Verification Plan
- Build with `npm run build` and `npm run tauri build`.
- Start release executable and confirm `127.0.0.1:15722` listens.
- Request `/claude-desktop/v1/models?limit=1000` and confirm models return.
- Close the app window and confirm the process and port remain active.

## Verification Result
- `npm run build` passed.
- `cmd /c build.bat check` passed after loading the MSVC environment.
- `cmd /c build-release.bat` passed and produced the NSIS installer.
- Started the release executable; `127.0.0.1:15722` listened under the `ccs2claude` process.
- `GET /claude-desktop/v1/models?limit=1000` returned the current CC Switch model list.
- After a normal window close request, the process and port remained active and `/v1/models` still returned models.

## Rollback
Use the app's revert action to switch Claude Desktop back to the CC Switch entry, then revert the local commit if this change must be removed.
