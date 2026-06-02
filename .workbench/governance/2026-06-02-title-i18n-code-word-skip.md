# Conversation Title I18n Code Word Skip

## Scope
- Fix missed translation for conversation titles that contain the English word `code`.
- Keep the change limited to the conversation title i18n injected script.

## Root Cause
- The sidebar candidate filter excluded navigation labels with a broad substring regex.
- Because `Code` was included in that regex, a real title such as `Comprehensive alma project code audit` was treated as a navigation shortcut and skipped.

## Change
- Replaced the broad navigation exclusion with an exact-label helper for sidebar shortcuts.
- `Code` still excludes the standalone Code mode button, but no longer excludes conversation titles containing `code`.
- Added a regression test for the old broad skip pattern.

## Verification
- `cargo test conversation_title_i18n_ --lib -- --nocapture`: passed.
- `npm run build`: passed.
- `cargo test --lib`: passed.
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_title_i18n_enhance_writes_bridge --lib -- --ignored --nocapture`: passed.
- Installed resource check confirmed the new exact-label helper is present and the old broad `...Code...test(r)` skip pattern is absent.

## Rollback
- Use the Claude++ enhance uninstall/reinstall flow for `conversation_title_i18n`, or revert the local commit containing this record and the `claude_enhance.rs` change.
