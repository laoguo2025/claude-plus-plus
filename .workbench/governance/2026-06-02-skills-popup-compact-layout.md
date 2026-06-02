# Skills Popup Compact Layout

## Request
- Simplify the Claude Desktop skills popup display to three pieces of information:
  - skill name
  - one-sentence Chinese functional summary
  - skill file path

## Change
- Updated the injected skills popup cards to render only name, brief summary, and file path.
- Removed the old long English description, project/global metadata text, and duplicated directory path from each card.
- Shortened generated summaries from `SKILL.md` descriptions by normalizing common English trigger prefixes into concise Chinese wording.
- Updated delete confirmation to read the skill name from the new compact card layout.
- Added a regression test for the compact layout markers and removal of the old fields.

## Verification
- `cargo fmt --manifest-path src-tauri\Cargo.toml`
- `npm run build`
- `cargo test claude_enhance::imp::tests --manifest-path src-tauri\Cargo.toml --lib`
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_plugins_enhance_writes_skills_bridges --manifest-path src-tauri\Cargo.toml --lib -- --ignored --nocapture`
- Resource check under Claude Desktop `ion-dist/assets/v1` confirmed `cps-name`, `cps-brief`, and `cps-file` were present, while old `cps-summary`, `cps-meta`, and `project_path?` markers were absent.
- `cargo test --manifest-path src-tauri\Cargo.toml --lib` passed with 18 tests passed and 1 ignored.

## Rollback
- Revert the local commit for this change.
- Or use the existing Claude++ enhancement backup/restore path, then reinstall the previous enhancement bundle into Claude Desktop resources.
