# Claude Skills Popup

## Reason
The injected Claude Desktop `技能` navigation item opened Claude's built-in plugin page. The required behavior is a Claude++ popup listing local Claude skills from global and project sources, with confirmed deletion through the Windows Recycle Bin.

## Change
- Added local skill discovery for `%USERPROFILE%\.claude\skills`.
- Added project skill discovery from `%USERPROFILE%\.claude.json`, `%USERPROFILE%\.claude\projects` cache directories, and cached session JSONL `cwd` values so old or hidden Claude Desktop/Claude CLI project conversations can still contribute project paths.
- Added local gateway endpoints:
  - `GET /claude-plus/skills`
  - `POST /claude-plus/skills/:id/trash`
- Updated the injected `技能` button to open a modal, render Chinese template summaries, and require confirmation before deletion.
- Deletion re-scans current skills, accepts only known skill ids, validates the directory still contains `SKILL.md`, and sends the directory to the Windows Recycle Bin.

## Verification
- `cargo fmt --manifest-path src-tauri\Cargo.toml -- --check`
- `npm run build`
- `cargo test --manifest-path src-tauri\Cargo.toml claude_skills::tests --lib`
- `cargo check --manifest-path src-tauri\Cargo.toml`
- `npm run tauri -- build`
- Runtime smoke with `src-tauri\target\release\claude-plus-plus.exe`:
  - one `claude-plus-plus` process running
  - `GET /claude-plus/skills` returned 27 real skills and 4 project sources after cleanup
  - temporary global smoke skill was listed, `POST /trash` returned `ok: true`, and the temporary directory no longer existed

## Rollback
Revert the local commit for this change. If a release exe is already running, stop it before rebuilding or reverting because Windows locks `src-tauri\target\release\claude-plus-plus.exe`.
