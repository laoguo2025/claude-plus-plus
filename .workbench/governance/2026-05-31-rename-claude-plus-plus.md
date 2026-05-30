# Rename to Claude++

## Context
The project is being renamed to match the Codex++ / CodexPlusPlus style: user-visible product name is `Claude++`, while repository/package slug is `claude-plus-plus`.

## Change reason
The previous name `ccs2claude` was implementation-oriented. The new name is clearer as a product name and matches the requested repository slug.

## Non-change constraints
- Do not change the local gateway port `15722`.
- Do not change the fixed Claude Desktop configLibrary UUID, so existing installed integration can be updated in place.
- Do not keep the old `ccs2claude-` model ID prefix fallback.
- Do not push or rewrite git history.

## Verification plan
- Search current source for `claude-plus-plus-` and current product surfaces for stale `ccs2claude`.
- `npm run build`
- `.\build.bat check`
- `.\build.bat test`

## Rollback
Revert the local rename commit. If the local folder is renamed, rename it back outside git.
