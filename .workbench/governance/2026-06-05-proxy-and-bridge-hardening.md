# Proxy and Bridge Hardening

## Context

Static review found several boundary violations: main proxy forwarding could rely on stale inbound Authorization and a fixed upstream fallback, diagnostics status reads could restore the proxy, gateway-dependent page enhancements wrote Claude Desktop resources before checking the gateway, the Skills bridge could delete via an in-bridge filesystem fallback, and unused `zh-HK` / `zh-TW` resources remained despite `zh-CN`-only support.

## Changes

- Main proxy forwarding now requires a complete current CC Switch gateway profile and sets bearer auth from that profile while dropping inbound Authorization.
- Removed the production fixed upstream fallback constant.
- Diagnostics report generation uses a no-restore status mode and remains read-only with respect to proxy startup.
- Gateway-dependent page enhancements start the local gateway before writing Claude Desktop resources.
- Skills bridge deletion now goes through the authenticated local gateway only; local filesystem fallback remains only for read-only listing.
- Removed unused `zh-HK` and `zh-TW` bundled localization resource files while keeping legacy cleanup language names.
- Updated the project map with the tightened proxy and Skills bridge boundaries.

## Verification

- `npm run typecheck`
- `npm run check:rust`
- `npm run test:rust`
- `npm run audit:claude-zh`

## Rollback

Revert the local commit for this slice. If Claude Desktop resources were already written by a prior install, use the app's page enhancement uninstall/reinstall controls as needed; this change itself did not write external Claude Desktop resources during verification.
