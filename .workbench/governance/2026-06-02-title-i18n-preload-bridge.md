# Title I18n Preload Bridge

## Symptom
- The red-boxed Claude Desktop conversation list still showed English titles after `对话列表中文化` was enabled.

## Evidence
- The local translation endpoint returned Chinese when called from PowerShell.
- Claude Desktop web logs showed prior CSP blocks for page-side `fetch` to `http://127.0.0.1:15722/...`.
- The page script still called `fetch("http://127.0.0.1:15722/claude-plus/conversation-title-i18n")` directly.

## Root Cause
- Claude Desktop's frontend CSP can block page-side local gateway calls.
- Plain sidebar conversation rows did not reliably expose route or aria metadata.

## Change
- Added `window.claudePlusTitleI18n.translate(title)` through the preload bridge.
- Added a main-process bridge handler that fetches the Claude++ local title translation endpoint outside the page CSP.
- Installing or cancelling `conversation_title_i18n` now also writes/removes the title i18n bridge in `app.asar`.
- Relaxed plain sidebar row matching while still limiting scanning to `aside` and `nav`.

## Verification
- Red tests first failed because the bridge functions did not exist.
- `cargo test --lib`
- `npm run build`

## Rollback
- Revert the local commit for this change.
- Reinstall or cancel the `对话列表中文化` enhancement to update Claude Desktop resources.
