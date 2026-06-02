# 2026-06-02 static debt cleanup

## Basis
- User approved the fourth batch of fixes from the documented review.
- Scope is limited to unused CSS, unused scaffold assets, and HTML scaffold title cleanup.

## Changes
- Removed unused `App.css` blocks for stale classes that no current component references.
- Removed unused scaffold SVGs: `public/tauri.svg` and `src/assets/react.svg`.
- Updated `index.html` title to `Claude++`.

## Non-changes
- Kept `public/vite.svg` because `index.html` still references it as favicon.
- Did not change any React component structure or runtime behavior.
- Did not alter Tauri app icons.

## Rollback
- Revert this local commit; no external writes or push are required.

## Verification
- `npm run build` passed.
- `cargo fmt --check` passed.
- `git diff --check` passed.
- `rg` confirms the removed CSS class selectors and deleted scaffold SVG references are no longer used by app source; remaining `badge` matches are TypeScript prop names backed by `.stateBadge`.
