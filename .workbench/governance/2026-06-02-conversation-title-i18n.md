# Conversation Title I18n Enhance

## Request
- Add a `对话列表中文化` card above `导出对话为 Markdown`.
- When enabled, translate Claude Desktop's English conversation-list titles into Chinese.

## Change
- Added a `conversation_title_i18n` page-enhance feature with its own marker.
- Inserted the new card in the conversation-enhance group before Markdown export.
- Added an injected sidebar scanner that only targets likely chat/conversation list entries, skips existing Chinese titles, caches translations, and leaves the original title unchanged when translation fails.
- Added `POST /claude-plus/conversation-title-i18n` on the local proxy. It reuses the current CC Switch model route and API key, asks for a short Simplified Chinese title, and returns only the translated title to the injected script.

## Verification
- Red test first: `cargo test --lib` failed because `build_title_translation_request` was missing.
- `cargo fmt`
- `npm run build`
- `cargo test --lib`

## Rollback
- Revert the local commit for this change.
- If already installed into Claude Desktop resources, use the page-enhance cancel action for `对话列表中文化` or restore the latest Claude Desktop enhancement backup.
