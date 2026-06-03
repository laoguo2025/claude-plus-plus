# App structure first split

## Scope

- Round 6 frontend-only slice.
- Split pure React app types into `src/appTypes.ts`.
- Split app constants, navigation metadata, and download/QR URLs into `src/appConstants.ts`.
- Split Tauri runtime command/open helpers into `src/tauriClient.ts`.
- Reuse the shared types from preview and route-status helpers.

## Non-changes

- No UI layout, copy, CSS, command names, backend behavior, proxy behavior, or enhance injection behavior changes.
- No Rust files touched.
- Existing browser preview semantics are preserved; this slice only moves boundaries.

## Validation

- Red check: `npm run build` failed after `App.tsx` was pointed at missing split modules, proving the build covers the new import boundary.
- Green check: `npm run build` passed after the split and import cleanup.

## Rollback

- Revert the local commit for this round.
- The split has no external writes or runtime migration; rollback is source-only.
