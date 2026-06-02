# Conversation Title I18n Black Screen

## Symptom
- Enabling `对话列表中文化` made Claude Desktop start to a black window.

## Evidence
- Claude Desktop `claude.ai-web.log` showed `Uncaught SyntaxError: Invalid regular expression: /(^|\\/: Unterminated group` at startup.
- The error appeared after the conversation title i18n enhancement was enabled.

## Root Cause
- The injected script used a regex literal containing an over-escaped slash pattern.
- Claude Desktop parsed the regex literal as a broken expression during frontend bootstrap, which stopped the main webview from rendering.

## Change
- Replaced the fragile regex literal with a `new RegExp(...)` string pattern.
- Added a regression test that rejects the broken regex literal fragments.

## Verification
- Red test first: `conversation_title_i18n_avoids_regex_literal_slash_escape_crash` failed on the broken script.
- `cargo test conversation_title_i18n_avoids_regex_literal_slash_escape_crash --lib`
- `npm run build`
- `cargo test --lib`

## Rollback
- Revert the local commit for this change.
- If already installed, reinstall the previous enhancement bundle or cancel `对话列表中文化`.
