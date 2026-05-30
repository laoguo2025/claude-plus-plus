# 2026-05-31 Model Label Uniqueness

## Symptom
Claude Desktop showed duplicate-looking `Mim* 2` entries in new sessions, while old sessions still showed a stale provider model such as `now-opus-4-6`.

## Evidence
- `ccs2claude` returned only two discovered IDs: `mimo-v2.5-pro` and `mimo-v2.5`.
- Claude Desktop formatted those raw IDs into lossy friendly names.
- Two CC Switch roles can share the same labelOverride, so display-label dedupe removed one role from discovery.
- Old sessions can keep their previously selected model ID in the session UI.

## Change Rationale
- Discovery model IDs are now generated as `ccs2claude-{role_kind}-{display}` so role entries remain unique.
- Discovery `display_name` now includes role and model label, e.g. `Opus - mimo-v2.5-pro`.
- Request rewrite accepts the generated ID, the generated display name, the old raw display label, and stale role-like IDs containing `opus`, `sonnet`, or `haiku`.

## Verification Result
- First ran `cmd /c build.bat test proxy::tests -- --nocapture` and observed compile failures for missing mapping helpers.
- Implemented the helpers and reran the same command: 3 tests passed.
- `cmd /c build.bat check` passed.
- `cmd /c build-release.bat` passed.
- Started the release executable and verified `/v1/models` returned three unique entries: `Opus - mimo-v2.5-pro`, `Haiku - mimo-v2.5`, and `Sonnet - mimo-v2.5`.

## Rollback
Revert the local commit to restore raw display-label discovery and old dedupe behavior.
