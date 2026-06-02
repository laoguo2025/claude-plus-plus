# MCP Nav Custom3p Connectors

## Symptom
- The Claude++ sidebar `MCP` entry opened Claude Desktop's `/customize/connectors` page.
- The expected target is the `连接器与扩展` page inside the same native `配置第三方推理` setup window used by `第三方API`.

## Root Cause
- The injected MCP nav item was configured with `path: "/customize/connectors"` and no custom open mode.
- `第三方API` already used Claude Desktop's `Custom3pSetup.openSetupWindow()` bridge.
- `Custom3pSetup.openSetupWindow()` accepts no pane argument, so selecting the connectors pane must happen after the setup window loads.

## Change
- Changed the MCP nav item to use `/setup-desktop-3p` with `open: "custom3p_connectors"`.
- Before opening the setup window, the script writes a one-time `localStorage` pane marker.
- When `/setup-desktop-3p` loads, the injected script reads the marker and clicks the `连接器与扩展` / `Connectors` / `MCP servers` navigation item, then clears the marker.
- Added a regression test that rejects the old MCP `/customize/connectors` route and requires the new setup-window mode.

## Verification
- Red test first: `mcp_nav_opens_custom3p_connectors_dialog` failed before implementation because `open:"custom3p_connectors"` was missing.
- `cargo fmt --manifest-path src-tauri\Cargo.toml`
- `cargo test claude_enhance::imp::tests::mcp_nav_opens_custom3p_connectors_dialog --manifest-path src-tauri\Cargo.toml --lib`
- `npm run build`
- `cargo test claude_enhance::imp::tests --manifest-path src-tauri\Cargo.toml --lib`
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_plugins_enhance_writes_skills_bridges --manifest-path src-tauri\Cargo.toml --lib -- --ignored --nocapture`
- Resource check under Claude Desktop `ion-dist/assets/v1` confirmed `open:"custom3p_connectors"` and `claudePlusCustom3pPane` were present, while the old MCP `/customize/connectors` nav route was absent.
- `cargo test --manifest-path src-tauri\Cargo.toml --lib` passed with 19 tests passed and 1 ignored.

## Rollback
- Revert the local commit for this change.
- Or restore the previous Claude Desktop enhancement backup and reinstall the earlier enhancement bundle.
