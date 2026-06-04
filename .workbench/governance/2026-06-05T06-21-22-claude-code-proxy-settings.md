# Claude Code proxy settings bootstrap

## Reason
The welcome page Claude Code card already detects whether the `claude` command is installed and can launch the official installer when missing. The requested behavior is to ensure Claude Code also receives the proxy-managed auth setting in `%USERPROFILE%\.claude\settings.json` after install, and to create or repair the same file when Claude Code is already installed.

## Change
- Added a shared Rust settings writer for `%USERPROFILE%\.claude\settings.json`.
- The writer preserves existing JSON fields, normalizes `env` to an object, and sets `ANTHROPIC_AUTH_TOKEN` to `PROXY_MANAGED`.
- Existing installed-status checks now call the writer after detecting `claude` on PATH.
- The visible PowerShell install command still runs the official Anthropic installer, then writes the same settings file before telling the user the script has ended.

## Verification
- `npm run test:rust -- welcome::imp::tests::claude_code_settings -- --nocapture`
- `npm run test:rust -- welcome::imp::tests -- --nocapture`

## Rollback
Revert the commit for code and document changes. Runtime-created user settings can be reverted by removing `%USERPROFILE%\.claude\settings.json` or deleting only `env.ANTHROPIC_AUTH_TOKEN` if the file had other settings.
