# 2026-06-02 route and parser tightening

## Basis
- User approved the second batch of fixes from the documented review.
- Scope is limited to route lifecycle cleanup, log tail reading, token usage parsing, and model role fallback matching.

## Changes
- `use_ccs_route` stops the local proxy only after Claude Desktop config revert succeeds.
- Diagnostic log tail reading scans backward from file end instead of loading the whole file.
- Token usage extraction skips whole-buffer JSON parsing and only parses SSE `data:` fragments.
- Model role fallback now requires a role-kind token boundary instead of arbitrary substring containment.

## Non-changes
- Did not add proxy health checks or restart policy.
- Did not change the configured gateway route, CC Switch DB reads, or injected JavaScript URLs.
- Did not alter exact model id, display name, or raw display matching precedence.

## Rollback
- Revert this local commit; no external writes or push are required.

## Verification
- `npm run build` passed.
- `cargo fmt --check` passed.
- `git diff --check` passed.
- Targeted `cargo test role_kind_fallback_requires_token_boundary` and `cargo test read_tail_returns_requested_last_lines` could not reach project tests because this shell cannot find MSVC `cl.exe`; native dependency build scripts for `libsqlite3-sys`, `vswhom-sys`, and `ring` fail first.
- `cargo check` has the same `cl.exe` environment blocker.
