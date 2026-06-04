# Localization restore preserves page enhancement

## Context
User observed that after restoring English, all Claude Desktop page enhancement entries disappeared. The shared frontend bundle is modified by both localization and page enhancement.

## Root cause
The localization restore path restored the latest `.zh-cn-backups` set wholesale before removing Chinese language files. If that backup was created before page enhancements were installed, it overwrote the enhanced frontend bundle and removed enhancement markers/scripts.

## Changes
- Removed the whole-file `.zh-cn-backups` restore call from localization uninstall.
- Localization restore now removes Chinese language files, unregisters Chinese languages from the frontend language list, removes the localization display-name monkey patch, sets locale to `en-US`, and resyncs `Claude.exe` ASAR integrity.
- Added a regression test proving restore-English bundle patch keeps page enhancement markers.
- Updated UI copy and disabled state so restore English no longer depends on backup availability and states that page enhancement scripts are preserved.
- Removed the unused shared whole-backup restore helper to avoid future accidental use.

## Validation
- `cargo test restore_english_bundle_patch_keeps_page_enhance_markers --lib` passed.
- `cargo test unregister_languages_only_updates_language_list --lib` passed.
- `cargo test --lib` passed: 106 passed, 5 ignored.
- `npm run build` passed.

## Rollback
Revert the local commit containing this record and related source changes. No live Claude Desktop resources were modified during this task.
