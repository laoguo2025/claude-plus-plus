# GitHub Release auto-fetch

## Scope
- User asked to make the repository/about information read the latest GitHub content automatically, then commit, build Windows 1.0.1 installer, and upload only the Windows exe.
- macOS packages are intentionally out of scope for this pass.

## Change
- Added a Tauri command that reads `laoguo2025/claude-plus-plus` latest GitHub Release through the GitHub Releases API.
- About page now checks the latest Release on load, supports manual refresh, shows version/published time/assets, and opens installer asset links.
- Browser preview has a matching mock command response.

## Validation
- `npm run build`
- `npm run test:rust -- github_release`
- `npm run check:rust`
- `git diff --check`

## Rollback
- Revert the local commit for this feature and rebuild/reupload the previous installer if needed.
