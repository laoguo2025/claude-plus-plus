@echo off
REM Build helper for Windows. Loads MSVC env (to avoid MSYS link.exe shadowing
REM the MSVC linker) then runs cargo from the src-tauri directory.
REM Usage: build.bat build   /   build.bat run   /   build.bat build --release
setlocal

REM Locate Visual Studio via vswhere; fall back to the common BuildTools path.
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "VCVARS="
if exist "%VSWHERE%" (
  for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do (
    set "VCVARS=%%i\VC\Auxiliary\Build\vcvars64.bat"
  )
)
if not defined VCVARS set "VCVARS=%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"

if not exist "%VCVARS%" (
  echo [build.bat] Could not find vcvars64.bat. Install Visual Studio Build Tools with the C++ workload.
  exit /b 1
)
call "%VCVARS%" >nul 2>&1

REM Run cargo relative to this script's location.
cd /d "%~dp0src-tauri"
cargo %*
endlocal
