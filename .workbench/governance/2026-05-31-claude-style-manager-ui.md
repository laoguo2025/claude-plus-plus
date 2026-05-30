# Claude-style manager UI refresh

## Reason
The app needed to borrow Codex++'s desktop manager layout while keeping Claude++ visually closer to Claude Desktop / Claude Cowork. The local one-click Claude Desktop localization feature also needed to remain a first-class entry after the rename from `ccs2claude`.

## Change
- Kept existing Tauri command surface and backend behavior unchanged.
- Reworked the React app into a left navigation, topbar, and right-side workspace.
- Added pages for overview, model mappings, Claude Desktop integration, and one-click localization.
- Used Claude-like light theme tokens: warm off-white backgrounds, clay brand color, restrained borders, and compact typography.
- Kept one-click localization controls visible in both overview and the dedicated localization page.

## Verification Plan
- Run frontend build/typecheck.
- Start Vite and inspect the UI in a browser at desktop and narrow widths.
- Confirm no controls overlap and localization controls remain accessible.

## Rollback
Revert the UI commit to restore the previous single-column card interface.
