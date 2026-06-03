# Route Status Semantics

## Context

The review identified route status copy that treated `cd_applied` as enough to say Claude Desktop was connected to Claude++, even when the local proxy was not running. It also noted that the CC Switch route detail expression mixed the disabled and unreachable cases.

## Changes

- Added pure route status text helpers in `src/routeStatus.ts`.
- Route summary now distinguishes not applied, applied with proxy stopped, and applied with proxy running.
- CC Switch route detail now distinguishes unreadable config, disabled route, enabled-but-unreachable route, and healthy route.
- `refreshRouteState` no longer clears the global error before every poll, so repeated mapping reads do not erase unrelated user-visible errors. Mapping failures remain scoped to the mapping card through `mappingError`.

## Validation

- `npm run build` failed before `src/routeStatus.ts` existed, proving the new route status module was required.
- `npm run build` passed after implementing the route status helpers and UI integration.

## Rollback

Revert the local commit for this slice. No Rust route detection or Tauri command behavior was changed.
