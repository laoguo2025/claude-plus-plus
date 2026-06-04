# Claude Desktop install discovery

## Context
User reported that Claude++ showed Claude Desktop as missing even after reinstalling Claude Desktop, and that prior Chinese localization disappeared after reinstall. The diagnostic report had `claude_found=false`, null install/resource paths, and no language files.

## Change reason
Claude++ is open-source and must not depend on one user's local path. Detection should cover common Windows installation surfaces and distinguish "not located" from "not installed".

## Changes
- Added generic Claude Desktop discovery sources: Claude++ settings overrides, running `Claude.exe` process path, uninstall registry entries, Start Menu shortcuts, WindowsApps package directories, Program Files, and `%LOCALAPPDATA%\Programs\Claude`.
- Kept resource layout validation as the final gate before localization or page enhancement writes.
- Added settings overrides `claudeDesktopPath` / `claudeDesktopResourcesPath` for users whose install location cannot be auto-discovered.
- Updated UI copy from "未安装/未检测到" to "未定位" for Claude Desktop resource discovery failures.
- Added the Claude++ settings path to diagnostics.

## Rollback
Revert the local commit containing this record and the related changes. No external Claude Desktop resources were written in this task.
