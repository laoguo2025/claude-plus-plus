# Claude Enhance Reinstall Verification

## Reason
The packaged Claude++ executable contained the fixed skills popup script, but Claude Desktop still opened the native Customize page because its installed frontend resource still had the previous injected script.

## Action
- Stopped Claude Desktop processes.
- Backed up `resources/ion-dist/assets/v1/index-nw6xXXcG.js` under `.claude-plus-enhance-backups/20260602-075743`.
- Removed the old `__claudePlusEnhanceNavV2` injection from the installed Claude Desktop entry bundle.
- Preserved enabled menu markers and appended the current `INJECT_SCRIPT` from `src-tauri/src/claude_enhance.rs`.
- Restarted Claude++ and Claude Desktop.

## Verification
- Installed Claude Desktop entry bundle now has one `__claudePlusEnhanceNavV2` marker.
- Installed `function u(e,n)` contains the `open` branch with `j(t)`, role button, `stopImmediatePropagation`, and keyboard handling.
- `GET http://127.0.0.1:15722/claude-plus/skills` returned 27 skills and 4 project sources after restart.

## Lesson
Rebuilding Claude++ does not update an already-installed Claude Desktop page enhancement. After changing `INJECT_SCRIPT`, the enhancement must be reinstalled into the Claude Desktop resource bundle and verified there, not only in the Claude++ release binary.
