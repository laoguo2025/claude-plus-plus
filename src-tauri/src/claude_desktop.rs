use std::process::Command;

#[cfg(target_os = "windows")]
pub fn restart() -> Result<(), String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            restart_script(),
        ])
        .output()
        .map_err(|e| format!("启动 PowerShell 失败: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        Err(if detail.is_empty() {
            "重启 Claude Desktop 失败".to_string()
        } else {
            format!("重启 Claude Desktop 失败: {detail}")
        })
    }
}

#[cfg(not(target_os = "windows"))]
pub fn restart() -> Result<(), String> {
    Err("当前只支持在 Windows 上重启 Claude Desktop".to_string())
}

#[cfg(target_os = "windows")]
fn restart_script() -> &'static str {
    r#"
$ErrorActionPreference = 'Stop'
$procs = @(Get-Process -Name Claude -ErrorAction SilentlyContinue)
$exe = $null

foreach ($p in $procs) {
  try {
    if ($p.Path) {
      $exe = $p.Path
      break
    }
  } catch {}
}

if ($procs.Count -gt 0) {
  foreach ($p in $procs) {
    try {
      if ($p.MainWindowHandle -ne 0) {
        [void]$p.CloseMainWindow()
      }
    } catch {}
  }

  $deadline = (Get-Date).AddSeconds(12)
  while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 250
    if (-not (Get-Process -Name Claude -ErrorAction SilentlyContinue)) {
      break
    }
  }

  $left = @(Get-Process -Name Claude -ErrorAction SilentlyContinue)
  if ($left.Count -gt 0) {
    $left | Stop-Process -Force
  }
}

if ($exe -and (Test-Path -LiteralPath $exe)) {
  Start-Process -FilePath $exe
  exit 0
}

$pkg = Get-AppxPackage -Name Claude -ErrorAction SilentlyContinue | Select-Object -First 1
if ($pkg) {
  Start-Process "shell:AppsFolder\$($pkg.PackageFamilyName)!App"
  exit 0
}

$alias = Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps\Claude.exe'
if (Test-Path -LiteralPath $alias) {
  Start-Process -FilePath $alias
  exit 0
}

throw '未找到 Claude Desktop 启动入口'
"#
}
