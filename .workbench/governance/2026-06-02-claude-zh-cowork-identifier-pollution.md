# Claude Desktop Cowork Identifier Pollution

## Scope
- Fix Claude Desktop Cowork crash after Claude++ Chinese localization is applied.
- Keep the change limited to the localization hardcoded frontend replacement path.

## Root Cause
- Claude Desktop logs reported `TypeError: e.shader来源 is not a function` and fatal error code `0QG8NES` on `/task/new`.
- Installed frontend bundles contained polluted JavaScript identifiers such as `shader来源`, `supported扩展`, `set来源Branch`, and `trust来源s`.
- The previous hardcoded localization pass used global text replacement for short entries such as `Source -> 来源` and `Extensions -> 扩展`, which changed identifier fragments inside minified JS code.

## Change
- Hardcoded frontend replacements now check JavaScript identifier boundaries before replacing English source text.
- Before applying new replacements, the installer repairs prior pollution by restoring `来源` and `扩展` only when the Chinese fragment is inside an ASCII JavaScript identifier.
- Literal visible Chinese strings such as `title:"来源"` and `label:"扩展"` are intentionally preserved.

## Verification
- `cargo test hardcoded_frontend_ --lib -- --nocapture`: passed.
- `npm run build`: passed.
- `cargo test --lib`: passed.
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_zh_cn_keeps_cowork_identifiers --lib -- --ignored --nocapture`: passed.
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_plugins_enhance_writes_skills_bridges --lib -- --ignored --nocapture`: passed.
- `CLAUDE_PLUS_VERIFY_INSTALL=1 cargo test verify_install_title_i18n_enhance_writes_bridge --lib -- --ignored --nocapture`: passed.
- Installed resource check found no `shader来源`, `supported扩展`, `set来源Branch`, or `trust来源s`; restored identifiers such as `shaderSource`, `supportedExtensions`, `setSourceBranch`, and `trustSources` are present.

## Rollback
- Use the Claude++ localization restore action to restore Claude Desktop resources from `.zh-cn-backups`.
- For code rollback, revert the local commit that contains this record and the `claude_zh.rs` change.
