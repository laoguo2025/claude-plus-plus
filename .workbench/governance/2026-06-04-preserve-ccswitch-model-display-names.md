# Preserve CC Switch model display names

## Scope
User clarified that Claude++ must not rewrite Claude Desktop model menu labels. Duplicate names are a user configuration choice in CC Switch.

## Change
- `/claude-desktop/v1/models` now returns `display_name` exactly from the CC Switch menu display label.
- Internal model IDs remain role-unique so forwarding can still distinguish Opus, Sonnet, and Haiku.
- Request forwarding keeps legacy `Role - label` names compatible for existing sessions.
- CCS transfer page explanatory copy now states that Claude Desktop displays CC Switch labels as-is.

## Rollback
Revert this commit to restore role-prefixed display names.
