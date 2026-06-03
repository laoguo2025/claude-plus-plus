# Release Asset Order and Notes Update

## Scope
- Retry pushing the local README redesign commit.
- Update GitHub Release `v1.0.0` notes by removing the installer explanation section and improving the main feature copy.
- Rename the Apple Silicon release asset from `Claude++_1.0.0_aarch64.dmg` to `Claude++_1.0.0_arm64.dmg`.
- Reorder visible Release assets so the two DMG files are adjacent and above the Windows installer.

## Validation Results
- Release notes now contain only the short release summary and main feature list.
- Release assets read back in order: `Claude++_1.0.0_arm64.dmg`, `Claude++_1.0.0_x64.dmg`, `Claude++_1.0.0_x64_setup.exe`.
- README installer table was synchronized with the visible Release asset names.

## Rollback
- Re-upload the previous asset names from the Release downloads if needed.
- Revert the README/governance commit locally before pushing, or revert it on GitHub after push.
