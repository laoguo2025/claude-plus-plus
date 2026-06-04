# Enhance Locale Sync

## Reason
Restoring Claude Desktop to English restored most built-in Claude menus, but installed Claude++ enhancement entries and dialogs still showed Chinese. The enhancement install path used one fixed injected script instead of matching the current Claude Desktop locale.

## Root Cause
The page enhancement script and Skills main-process bridge had Chinese visible copy baked into the bundled script. Localization install/restore changed Claude Desktop locale files but did not rewrite already enabled enhancement scripts or bridge copy.

## Change
- Added an enhancement script locale switch: only `zh-CN` emits Chinese copy; missing, `en-US`, or any other locale emits English copy.
- Made install, upgrade, one-click localization, and restore-English flows refresh already enabled enhancement scripts without changing enabled feature markers.
- Made the Skills main-process bridge visible defaults and errors follow the enhancement locale.
- Limited supported localization languages to `zh-CN`; legacy `zh-TW` and `zh-HK` remain only for cleanup of previously written files.
- Bumped enhancement definitions to `v0.2` so existing enabled installs can be upgraded to the locale-aware bundle.

## Validation
- Targeted Rust tests passed for injected-script English/Chinese copy, Skills bridge copy, locale refresh marker preservation, and zh-CN-only language validation.
- Full validation is completed before commit in the task transcript.

## Rollback
Revert the local commit for code/config/documentation changes. In the app, rerun the relevant enhancement install or localization restore action to rewrite Claude Desktop resource files from the previous bundled behavior.
