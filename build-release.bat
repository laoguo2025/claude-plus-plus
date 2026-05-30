@echo off
REM Release build: loads MSVC env (to avoid MSYS link.exe shadowing the MSVC
REM linker), then runs `tauri build` which bundles the frontend into the exe
REM and produces installers under src-tauri\target\release\bundle.
setlocal

set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "VCVARS="
if exist "%VSWHERE%" (
  for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do (
    set "VCVARS=%%i\VC\Auxiliary\Build\vcvars64.bat"
  )
)
if not defined VCVARS set "VCVARS=%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"

if not exist "%VCVARS%" (
  echo [build-release.bat] Could not find vcvars64.bat. Install Visual Studio Build Tools with the C++ workload.
  exit /b 1
)
call "%VCVARS%" >nul 2>&1

cd /d "%~dp0"
call npx tauri build %*
endlocal
