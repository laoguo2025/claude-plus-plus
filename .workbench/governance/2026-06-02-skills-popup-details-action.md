# Skills Popup Details Action

## Request
- Stop translating skill descriptions into Chinese summaries.
- Add a `详情` button to the left of each skill card's `删除` button.

## Change
- Kept compact skill cards but changed the brief line back to normalized original description text.
- Added a per-card `详情` action before `删除`.
- `详情` toggles an inline detail block with original description, skill file path, and skill directory path.
- Kept the delete flow unchanged except for continuing to read the skill name from the compact layout.
- Updated the regression test to require detail controls and reject the old Chinese translation prefixes.

## Verification
- Red test first: `skills_popup_cards_use_compact_layout_with_details_action` failed before implementation because `data-cps-detail` was missing.
- `cargo fmt --manifest-path src-tauri\Cargo.toml`
- `cargo test claude_enhance::imp::tests::skills_popup_cards_use_compact_layout_with_details_action --manifest-path src-tauri\Cargo.toml --lib`
- `npm run build`
- `cargo test claude_enhance::imp::tests --manifest-path src-tauri\Cargo.toml --lib`
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_plugins_enhance_writes_skills_bridges --manifest-path src-tauri\Cargo.toml --lib -- --ignored --nocapture`
- Resource check under Claude Desktop `ion-dist/assets/v1` confirmed `data-cps-detail` and `原始描述` were present, while `适用于` and `该技能用于` were absent.

## Rollback
- Revert the local commit for this change.
- Or restore the previous Claude Desktop enhancement backup and reinstall the earlier enhancement bundle.
