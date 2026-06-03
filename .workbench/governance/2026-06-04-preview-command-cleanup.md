# Preview Command Cleanup

## Context

The review identified that browser preview command mocks lived inside `App.tsx`, shared mutable preview objects with the component, returned `undefined` for unknown commands, and duplicated the busy action pattern across button handlers.

## Changes

- Moved preview command handling and mock payloads into `src/previewCommands.ts`.
- Kept preview state module-local and returned cloned status payloads to avoid sharing mutable objects with callers.
- Changed unsupported preview commands to throw a clear browser-preview error.
- Added preview no-op handling for the app commands currently used by the UI, including localization install/uninstall and Claude Desktop restart.
- Added a local `runBusy` helper in `App.tsx` for repeated button command flows.

## Validation

- `npm run build` failed before `src/previewCommands.ts` existed, proving the new module path was required.
- `npm run build` passed after implementing the preview module.
- `npm run build` passed after replacing repeated busy handlers with `runBusy`.

## Rollback

Revert the local commit for this slice. No Rust or runtime Tauri command behavior was changed.
