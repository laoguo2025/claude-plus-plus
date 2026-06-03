# 2026-06-03 Welcome And Github UI

## Scope
- Rename the old About page/navigation entry to Github repository.
- Add a new Welcome page before CCS transfer.
- Move Claude Desktop installation status from CCS transfer into Welcome.
- Add read-only Welcome status checks for Claude Desktop developer mode and CC Switch installation.
- Show the two user-provided desktop QR images through the Tauri asset protocol.

## Reason
The app needed a first-run welcome surface that carries the existing product copy, community/support QR codes, and basic environment checks. The CCS transfer page should stay focused on route transfer status.

## Validation
- `npm run build`
- `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo test welcome --lib'`
- Local Vite preview browser check for navigation order, Welcome title/content, two QR image nodes, three Welcome status cards, Github repository title cleanup, and CCS transfer status cards.
- `git diff --check`

## Rollback
Revert the commit for this slice. This removes the new Welcome route, the Github repository rename, the read-only status command, and the Tauri asset protocol QR image scope.
