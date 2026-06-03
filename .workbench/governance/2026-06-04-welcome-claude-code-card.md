# 2026-06-04 Welcome Claude Code Card

## Scope
- Add a Claude Code status card before the existing Welcome environment cards.
- Detect Claude Code through the local `claude` command.
- Launch the official Anthropic command-line installer in a visible shell when the card action is clicked.
- Keep four Welcome status cards in one desktop row.

## Evidence
- `anthropics/claude-code` currently recommends `irm https://claude.ai/install.ps1 | iex` for Windows and `curl -fsSL https://claude.ai/install.sh | bash` for macOS/Linux.
- This Windows machine has `claude` available through npm shims under `%APPDATA%\npm`; Windows detection uses `where claude` to recognize shim-based installs.

## Validation
- `npm run build`: passed.
- `git diff --check` for the touched files: passed.
- Rust library verification is currently blocked by unrelated dirty `proxy.rs`/`ccswitch_db.rs` token-usage changes in the working tree: `TokenUsageSnapshot` initializers are missing the pre-existing `call_count` and `source` fields.

## Rollback
Revert the local commit for this slice. If a user has already clicked install, remove Claude Code using Anthropic's official uninstall guidance or remove the installed `claude` command from the user's shell path.
