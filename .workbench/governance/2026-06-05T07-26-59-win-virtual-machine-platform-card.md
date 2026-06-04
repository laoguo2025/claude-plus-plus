# Win virtual machine platform welcome card

## Reason
The welcome page needs a preflight card before Claude Code that checks whether the Windows features required by the user are enabled. The requested scope excludes Windows Hypervisor Platform and requires only WSL plus Virtual Machine Platform.

## Change
- Added Windows-only status detection for `Microsoft-Windows-Subsystem-Linux` and `VirtualMachinePlatform`.
- Added a Tauri command that launches an elevated visible PowerShell process to run `Enable-WindowsOptionalFeature` with `-All -NoRestart`.
- Added a welcome page card before Claude Code. If the features are missing, the page automatically requests enablement once and tells the user to restart Windows.
- Updated preview state and TypeScript status shape.

## Verification
- `npm run test:rust -- welcome::imp::tests::virtual_machine_platform -- --nocapture`
- `npm run typecheck`

## Rollback
Revert the commit for code and document changes. If the elevated command was run on a machine, Windows optional features can be disabled manually through "Turn Windows features on or off" or with DISM, followed by a restart.
