# Repo Presentation, Release Workflow, and Labels

## Scope
- Improve the public repository README against the CodexPlusPlus reference without sponsor or donation content.
- Add GitHub issue templates.
- Add a Release workflow that uploads Windows NSIS and macOS arm64/x64 DMG installers to published GitHub Releases.
- Add repository topics and issue labels through GitHub CLI.

## Validation Plan
- Run frontend build.
- Check release workflow syntax by inspecting trigger, permissions, and upload paths.
- Confirm GitHub topics and labels through `gh`.

## Validation Results
- `npm run build` passed.
- Release workflow static checks passed for published-release trigger, `contents: write`, Windows NSIS build/upload, and macOS arm64/x64 DMG build/upload.
- README and `.github/` files contain no sponsor or donation keywords.
- GitHub topics and labels were applied and read back with `gh`.
- Screenshot capture was attempted through a temporary Playwright command, but the transient package was not available to the script; no screenshot asset was added.

## Rollback
- Revert the local commit that contains README, workflow, and template changes.
- Remove or edit GitHub topics/labels with `gh repo edit --remove-topic` and `gh label delete/edit` if needed.
