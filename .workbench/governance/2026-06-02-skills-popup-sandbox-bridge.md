# Skills Popup Sandbox Bridge

## Symptom
- Claude Desktop skills popup showed `本地 skills 桥未安装或尚未生效。`
- Local global skills existed under `%USERPROFILE%\.claude\skills`.

## Root Cause
- The injected popup UI was present and the old bridge marker existed in `mainView.js`.
- Claude Desktop creates the main view with sandboxed preload.
- The old injected preload bridge called `require("fs")`, `require("path")`, and `require("crypto")` directly.
- Runtime log confirmed: `[Claude++] skills bridge failed Error: module not found: fs`.

## Change
- Split the skills bridge into two injected scripts.
- Main-process bridge in `.vite/build/index.js` handles filesystem scanning and recycle-bin deletion.
- Sandboxed preload bridge in `.vite/build/mainView.js` only exposes `window.claudePlusSkills` through IPC.
- Status detection now requires both bridge markers.

## Verification
- `npm run build`
- `cargo test claude_enhance::imp::tests --manifest-path src-tauri\Cargo.toml --lib`
- `cargo test --manifest-path src-tauri\Cargo.toml --lib`
- Ignored install verification with `CLAUDE_PLUS_VERIFY_INSTALL=1` passed and rewrote Claude Desktop resources.
- Current `app.asar` check:
  - `index.js` contains `__claudePlusSkillsMainBridgeV1`.
  - `mainView.js` contains `__claudePlusSkillsBridgeV1`.
  - `mainView.js` no longer contains `require("fs")`, `require("path")`, or `require("crypto")`.
- After launching Claude Desktop at 2026-06-02 10:28, logs no longer added the old `module not found: fs` bridge error.

## Rollback
- Use Claude++ page enhancement uninstall for `plugins`, or restore from the latest `.claude-plus-enhance-backups` set under Claude Desktop resources.
