# 2026-06-03 Remove Reademe Compat

## Scope
User approved removing the `.workbench/reademe.md` compatibility pointer.

## Changes
- Removed `.workbench/reademe.md`.
- Kept `.workbench/readme.md` as the only active workbench navigation entry.

## Validation
- Repository search found no active source or navigation references to `.workbench/reademe.md`; only historical governance notes mention the old path.

## Rollback
- Revert this commit to restore the compatibility pointer.
