# Conversation Title I18n No Effect

## Symptom
- `对话列表中文化` was enabled, but the red-boxed Claude Desktop conversation list still showed English titles.

## Evidence
- `POST /claude-plus/conversation-title-i18n` was reachable but returned `{ ok: false, title: "" }`.
- The injected script only scanned sidebar anchors/buttons/role links/buttons, while the visible recent conversation rows can render as plain sidebar list containers.

## Root Cause
- The translation response extractor only handled Anthropic-style `content[].text`; CC Switch/third-party gateways can return OpenAI-style `choices[].message.content`.
- The DOM scanner missed plain `div`, `li`, and `role=listitem` sidebar rows.

## Change
- Added OpenAI-style response extraction for translated titles.
- Expanded the sidebar scanner to include plain sidebar list containers while still limiting scope to `aside`/`nav`.
- Added regression tests for both response extraction and sidebar list-item scanning.

## Verification
- Red test first: `extracts_openai_style_title_translation_response` and `conversation_title_i18n_scans_plain_sidebar_list_items` failed before implementation.
- `cargo test --lib`
- `npm run build`

## Rollback
- Revert the local commit for this change.
- Reinstall or cancel the `对话列表中文化` enhancement if already applied to Claude Desktop resources.
