# Shared Claude Patch Common

## Basis
- User requested stopping small batches and prioritizing important document fixes.
- This batch targets duplicated Claude Desktop patching helpers in Chinese localization and page enhancement modules.

## Changes
- Added a shared Windows patch helper module for Claude path discovery, write permission setup, backup sets, ASAR header parsing/encoding, ASAR entry helpers, ASAR integrity calculation, executable integrity sync, and file/path utilities.
- Kept Chinese localization backup directory `.zh-cn-backups` and enhancement backup directory `.claude-plus-enhance-backups`.
- Added backup retention pruning with `MAX_BACKUP_SETS = 10`.
- Unified ASAR header encoding into `encode_asar_header(header_string, expected_header_size)`.
  - Chinese localization passes `Some(parsed.header_size)` to preserve fixed-size patching.
  - Enhancement passes `None` to preserve variable-size ASAR patching.
- Replaced direct backup-set field access with `BackupContext::has_backup()`.

## Non-Changes
- Install, backup, uninstall, locale restore, injected scripts, feature marker behavior, bridge scripts, and language pack contents are not intentionally changed.
- No push was performed.

## Rollback
- Revert the local commit for this batch.

## Verification
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check` passed; Git only emitted line-ending normalization warnings.
- Static residual search found no duplicate helper definitions or invalid `fn patch::...`/`restore_patch::...` leftovers in the two old modules.
- `npm run build` passed.
- `cargo check` could not reach project code because this shell cannot find MSVC `cl.exe`; native dependency build scripts for `libsqlite3-sys`, `vswhom-sys`, and `ring` fail first.
- `where.exe cl` found no `cl.exe`.
- `vswhere` returned no Visual Studio installation with `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`.
