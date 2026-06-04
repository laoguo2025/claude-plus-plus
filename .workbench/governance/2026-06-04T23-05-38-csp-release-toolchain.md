# CSP, Release Links, and Toolchains

## Scope
- Tightened the Tauri app-shell CSP while preserving required local IPC, asset, and Vite development endpoints.
- Replaced stale About-page repository/release wording with real GitHub repository and Release actions.
- Declared Node/npm and Rust toolchains.
- Added production dependency audit to CI.

## Reason
The second review batch found that the app shell still used a null CSP, the About page described release discovery inconsistently with the public repository workflow, and project toolchains / production dependency audit gates were not explicit enough for repeatable validation.

## Changes
- Added explicit production `csp` and development `devCsp` in the Tauri config.
- Kept `opener:default` because the app uses it for user-triggered external URLs, including download, repository, and release buttons.
- Added `GITHUB_REPOSITORY_URL` and `GITHUB_RELEASES_URL`, and wired the About page buttons through the existing opener wrapper.
- Declared `packageManager`, Node/npm engine bounds, and `rust-toolchain.toml`.
- Added `npm audit --audit-level=high --omit=dev` to the CI frontend job.

## Validation
- `npm run typecheck` passed.
- `npm run build` passed.
- `npm run audit:claude-zh` passed with `count: 0`.
- `npm audit --audit-level=high --omit=dev` passed with 0 vulnerabilities.
- `npm run check:rust` passed.
- `npm run test:rust` passed: 118 passed, 5 ignored.

## Rollback
Revert the local commit. These changes only modify repository files and do not write Claude Desktop, CC Switch, or external services.
