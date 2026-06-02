# Conversation Title I18n Sidebar Filter

## Reason
The conversation title i18n enhancement was installed and its preload/main bridge was present, but visible sidebar titles were still not translated.

## Evidence
- Local gateway endpoint `POST /claude-plus/conversation-title-i18n` translated sample English titles successfully.
- Installed Claude Desktop resources contained the frontend feature marker, preload bridge, and main bridge.
- Temporary runtime diagnostics after restart showed `bridge: true` but `candidates: 0` with the old filter.
- After relaxing candidate detection to use actual text nodes, diagnostics showed request/response/applied events for sidebar titles.

## Change
Candidate filtering now allows sidebar row elements that contain a translatable English title text node, instead of requiring URL or aria metadata. The final script excludes shortcut/function entries such as `Ctrl+B` and known navigation labels, and does not keep runtime diagnostics.

## Verification
- `npm run build`
- `cargo test --lib`
- Ignored install verification test is used to write the final resource patch before runtime use.

## Rollback
Disable the `conversation_title_i18n` enhancement from Claude++ or restore Claude Desktop resources from the `.zh-cn-backups` backup set.
