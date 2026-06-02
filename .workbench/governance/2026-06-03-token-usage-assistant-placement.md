# 2026-06-03 Token Usage Assistant Placement

## Scope
User reported the Token usage badge was inserted near the chat input box. The intended placement is below the latest assistant reply, matching the referenced Codex token usage script behavior.

## Reference
- Reviewed `kokotao/codex-token-usage-script`, especially the assistant-container/action-button targeting strategy in `scripts/codex-token-usage.js`.

## Changes
- Removed the token usage insertion anchor that searched for `textarea` / `contenteditable` / form nodes.
- Added assistant-message targeting for token usage rendering:
  - Prefer visible assistant response action buttons such as copy/like/dislike/branch.
  - Fall back to the existing assistant message scan and assistant/message selectors.
  - Reject containers that contain input/editable controls.
- Removed the waiting badge near the input area; if no assistant reply exists, no token badge is rendered.
- Restyled the token usage element as a compact inline badge below the assistant message.

## Validation
- `npm run build` passed.
- `cargo fmt --check` passed.
- `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo test --lib'` passed: 47 passed, 4 ignored.
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_token_usage_enhance_writes_bridge --lib -- --ignored --nocapture` passed and rewrote the local Claude Desktop page-enhance resources through the formal install path.

## External Effects
- Local Claude Desktop resources were updated by the ignored install-verification test. A running Claude Desktop instance may need restart to load the corrected injected script.

## Rollback
- Revert this commit for source changes.
- For installed Claude Desktop page-enhance resources, use the Claude++ page-enhance uninstall/restore flow or restore the latest `.claude-plus-enhance-backups` set.
