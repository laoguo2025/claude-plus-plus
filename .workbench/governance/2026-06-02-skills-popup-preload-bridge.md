# Skills Popup Preload Bridge

## Reason
The injected Claude Desktop "技能" popup still called `http://127.0.0.1:15722/claude-plus/skills`. After Claude++ was closed, the popup could not load skills and showed an incorrect connection-state failure.

## Change
- The installed skills popup now calls `window.claudePlusSkills` instead of the Claude++ local gateway.
- Installing the "技能" enhancement patches Claude Desktop `app.asar` preload with a narrow bridge that scans `%USERPROFILE%\.claude\skills` plus known project `.claude\skills` roots.
- Delete actions keep user confirmation in the popup and move the corresponding skill directory to the recycle bin through Electron `shell.trashItem`.
- The popup layout is a fixed skills dialog with title, global skills container, and project skills container.

## Verification
- Installed through the formal `claude_enhance::install("plugins")` path on the local Claude Desktop install.
- Confirmed the installed frontend bundle contains `__claudePlusEnhanceNavV2`, `__claudePlusEnhancePluginsV1`, and `window.claudePlusSkills`.
- Confirmed the installed frontend bundle no longer contains `127.0.0.1:15722/claude-plus/skills` or the old "无法连接 Claude++ 本地服务" error.
- Confirmed installed `app.asar` `.vite/build/index.js` contains `__claudePlusSkillsBridgeV1`, `contextBridge.exposeInMainWorld`, and `shell.trashItem`.
- Confirmed Claude.exe embedded `resources\\app.asar` SHA256 marker matches the current `app.asar` header hash.
- Ran the same bridge scanning logic locally and found 27 global skills, 0 project skills, and 5 known project paths.

## Rollback
Use the existing page-enhancement uninstall/backup path. Backups are kept under Claude Desktop `resources\.claude-plus-enhance-backups`.
