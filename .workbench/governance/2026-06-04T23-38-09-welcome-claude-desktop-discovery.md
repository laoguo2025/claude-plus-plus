# Welcome Claude Desktop Discovery

## Scope
- Fixed the Welcome page Claude Desktop card text.
- Reused the shared Claude Desktop install discovery for the Welcome page presence check.

## Reason
The Welcome page could show Claude Desktop as "未定位" even when the Windows Store app existed under `C:\Program Files\WindowsApps`. Its lightweight status check had diverged from the generic discovery path used by localization and page enhancement flows. The card also told users to specify a resources directory, which is unrealistic for most users.

## Changes
- Changed the missing Claude Desktop card hint to `点击后从网盘下载`.
- Changed `welcome_status` to call the shared Claude Desktop discovery path instead of maintaining a separate fast directory scan.
- Updated the project map to record that Welcome should reuse generic install discovery while still avoiding page-enhancement bundle/asar reads.

## Validation
- `npm run typecheck` passed.
- `npm run build` passed.
- `npm run check:rust` passed.
- `npm run test:rust` passed: 119 passed, 5 ignored.
- `npm run audit:claude-zh` passed with `count: 0`.
- `npm audit --audit-level=high --omit=dev` passed with 0 vulnerabilities.

## Rollback
Revert the local commit. This change only modifies repository files and does not write Claude Desktop, CC Switch, or external services during validation.
