# Win virtual machine platform detection fix

## Reason
The first implementation used DISM/Get-WindowsOptionalFeature-style feature-info reads for status detection. On a normal non-elevated process this returns error 740, so Claude++ could misread an already enabled machine as disabled and auto-launch the elevated PowerShell path.

## Change
- Replaced status detection with non-elevated service marker checks: `LxssManager` for WSL and `vmcompute` for Virtual Machine Platform.
- Replaced nested `Start-Process ... -Command` quoting with a temporary `.ps1` script and `Start-Process powershell.exe -Verb RunAs -ArgumentList ... -File`.
- Kept the enable script limited to `Microsoft-Windows-Subsystem-Linux` and `VirtualMachinePlatform`, with `-All -NoRestart`.

## Evidence
- On the current machine, non-elevated DISM returned error 740 for both feature-info queries.
- On the current machine, `sc.exe query LxssManager` and `sc.exe query vmcompute` both succeeded and reported RUNNING.

## Verification
- `npm run test:rust -- welcome::imp::tests::virtual_machine_platform -- --nocapture`
- `npm run typecheck`

## Rollback
Revert the commit. If an elevated enable script was previously launched, Windows optional feature state is external OS state and must be changed through Windows Features or DISM followed by a restart.
