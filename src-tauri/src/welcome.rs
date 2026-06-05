#[cfg(target_os = "windows")]
mod imp {
    use crate::{claude_patch_common as patch, paths, server};
    use serde::Serialize;
    use serde_json::{Map, Value};
    use std::{
        env, fs, io,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        time::{SystemTime, UNIX_EPOCH},
    };

    const WINDOWS_CLAUDE_CODE_INSTALL_COMMAND: &str =
        "$ErrorActionPreference = 'Stop'; $installUrl = 'https://claude.ai/install.ps1'; $installScript = Join-Path $env:TEMP ('claude-ai-install-' + [guid]::NewGuid().ToString('N') + '.ps1'); Invoke-WebRequest -Uri $installUrl -OutFile $installScript -UseBasicParsing; if (!(Test-Path -LiteralPath $installScript)) { throw 'Claude Code 安装脚本下载失败。' }; if ((Get-Item -LiteralPath $installScript).Length -lt 100) { throw 'Claude Code 安装脚本内容异常。' }; & $installScript; $claudeDir = Join-Path $HOME '.claude'; New-Item -ItemType Directory -Force -Path $claudeDir | Out-Null; $settingsPath = Join-Path $claudeDir 'settings.json'; $settings = @{}; if (Test-Path -LiteralPath $settingsPath) { try { $json = Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json; if ($json -is [pscustomobject]) { $json.PSObject.Properties | ForEach-Object { $settings[$_.Name] = $_.Value } } } catch { $settings = @{} } }; $envMap = @{}; if ($settings.ContainsKey('env') -and $null -ne $settings['env']) { if ($settings['env'] -is [System.Collections.IDictionary]) { $envMap = $settings['env'] } elseif ($settings['env'] -is [pscustomobject]) { $settings['env'].PSObject.Properties | ForEach-Object { $envMap[$_.Name] = $_.Value } } }; $envMap['ANTHROPIC_AUTH_TOKEN'] = 'PROXY_MANAGED'; $settings['env'] = $envMap; $settings | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $settingsPath -Encoding UTF8; Write-Host ''; Write-Host 'Claude Code 安装脚本已结束，代理配置已写入。可关闭此窗口。'";
    const WINDOWS_ENABLE_VIRTUAL_MACHINE_PLATFORM_COMMAND: &str =
        "Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux,VirtualMachinePlatform -All -NoRestart; Write-Host ''; Write-Host 'Win虚拟机平台已发起开启。请重启电脑后再继续使用。'; Read-Host '按回车关闭窗口'";

    const CLAUDE_CODE_AUTH_TOKEN_ENV: &str = "ANTHROPIC_AUTH_TOKEN";
    const CLAUDE_CODE_PROXY_MANAGED_TOKEN: &str = "PROXY_MANAGED";

    #[derive(Serialize)]
    pub struct WelcomeStatus {
        pub claude_code_installed: bool,
        pub virtual_machine_platform_supported: bool,
        pub virtual_machine_platform_enabled: bool,
        pub claude_desktop_found: bool,
        pub developer_mode_enabled: bool,
        pub cc_switch_installed: bool,
    }

    pub fn status() -> WelcomeStatus {
        let claude_code_installed = detect_claude_code_installed();
        if claude_code_installed {
            let _ = ensure_claude_code_proxy_settings();
        }
        WelcomeStatus {
            claude_code_installed,
            virtual_machine_platform_supported: true,
            virtual_machine_platform_enabled: detect_virtual_machine_platform_enabled(),
            claude_desktop_found: detect_claude_desktop_found(),
            developer_mode_enabled: read_developer_mode_enabled(),
            cc_switch_installed: detect_cc_switch_installed(),
        }
    }

    pub fn install_claude_code() -> Result<(), String> {
        let mut command = Command::new("powershell.exe");
        command
            .args([
                "-NoExit",
                "-NoProfile",
                "-Command",
                WINDOWS_CLAUDE_CODE_INSTALL_COMMAND,
            ])
            .stdin(Stdio::null());
        command
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("启动 Claude Code 安装命令失败: {e}"))
    }

    pub fn enable_developer_mode() -> Result<(), String> {
        let path = developer_settings_write_path()
            .ok_or_else(|| "无法定位 Claude Desktop 开发者配置路径".to_string())?;
        enable_developer_mode_at_path(&path)
    }

    pub fn enable_virtual_machine_platform() -> Result<(), String> {
        if detect_virtual_machine_platform_enabled() {
            return Ok(());
        }
        let script = write_virtual_machine_platform_enable_script()
            .map_err(|e| format!("写入 Win 虚拟机平台开启脚本失败: {e}"))?;
        let mut command = Command::new("powershell.exe");
        command
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &enable_virtual_machine_platform_command(&script),
            ])
            .stdin(Stdio::null());
        command
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("启动 Win 虚拟机平台开启命令失败: {e}"))
    }

    fn detect_claude_code_installed() -> bool {
        path_has_windows_command("claude")
    }

    fn detect_virtual_machine_platform_enabled() -> bool {
        is_virtual_machine_platform_enabled_marker(
            windows_service_exists("LxssManager"),
            windows_service_exists("vmcompute"),
        )
    }

    fn is_virtual_machine_platform_enabled_marker(
        wsl_service_exists: bool,
        vmcompute_service_exists: bool,
    ) -> bool {
        wsl_service_exists && vmcompute_service_exists
    }

    fn windows_service_exists(service: &str) -> bool {
        let output = Command::new("sc.exe")
            .args(["query", service])
            .stdin(Stdio::null())
            .output();
        let Ok(output) = output else {
            return false;
        };
        output.status.success()
    }

    fn write_virtual_machine_platform_enable_script() -> io::Result<PathBuf> {
        let path = env::temp_dir().join(format!(
            "claude-plus-plus-enable-vm-platform-{}.ps1",
            std::process::id()
        ));
        fs::write(&path, WINDOWS_ENABLE_VIRTUAL_MACHINE_PLATFORM_COMMAND)?;
        Ok(path)
    }

    fn enable_virtual_machine_platform_command(script: &Path) -> String {
        let script = script.to_string_lossy().replace('\'', "''");
        format!(
            "Start-Process powershell.exe -Verb RunAs -ArgumentList @('-NoExit','-ExecutionPolicy','Bypass','-File','{script}')"
        )
    }

    fn ensure_claude_code_proxy_settings() -> Result<(), String> {
        let home = paths::home_dir().ok_or_else(|| "无法定位用户目录".to_string())?;
        ensure_claude_code_proxy_settings_at_path(&claude_code_settings_path_from_home(&home))
    }

    fn claude_code_settings_path_from_home(home: &Path) -> PathBuf {
        home.join(".claude").join("settings.json")
    }

    fn claude_code_settings_has_proxy_managed_env(text: &str) -> bool {
        serde_json::from_str::<Value>(text)
            .ok()
            .and_then(|value| {
                value
                    .get("env")
                    .and_then(|env| env.get(CLAUDE_CODE_AUTH_TOKEN_ENV))
                    .and_then(Value::as_str)
                    .map(|token| token == CLAUDE_CODE_PROXY_MANAGED_TOKEN)
            })
            .unwrap_or(false)
    }

    fn ensure_claude_code_proxy_settings_at_path(path: &Path) -> Result<(), String> {
        let mut settings = if path.is_file() {
            let text =
                fs::read_to_string(path).map_err(|e| format!("读取 Claude Code 配置失败: {e}"))?;
            if claude_code_settings_has_proxy_managed_env(&text) {
                return Ok(());
            }
            serde_json::from_str::<Value>(&text)
                .map_err(|e| format!("解析 Claude Code 配置失败: {e}"))?
        } else {
            Value::Object(Map::new())
        };

        let Value::Object(ref mut object) = settings else {
            return Err("Claude Code 配置不是 JSON 对象".to_string());
        };
        let env_value = object
            .entry("env".to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !env_value.is_object() {
            *env_value = Value::Object(Map::new());
        }
        let Value::Object(env_object) = env_value else {
            return Err("Claude Code env 配置不是 JSON 对象".to_string());
        };
        env_object.insert(
            CLAUDE_CODE_AUTH_TOKEN_ENV.to_string(),
            Value::String(CLAUDE_CODE_PROXY_MANAGED_TOKEN.to_string()),
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建 Claude Code 配置目录失败: {e}"))?;
        }
        let text = format!(
            "{}\n",
            serde_json::to_string_pretty(&settings)
                .map_err(|e| format!("生成 Claude Code 配置失败: {e}"))?
        );
        patch::atomic_write(path, text.as_bytes())
            .map_err(|e| format!("写入 Claude Code 配置失败: {e}"))?;

        let updated =
            fs::read_to_string(path).map_err(|e| format!("读回 Claude Code 配置失败: {e}"))?;
        if claude_code_settings_has_proxy_managed_env(&updated) {
            Ok(())
        } else {
            Err("Claude Code 配置已写入，但未读回代理托管认证配置".to_string())
        }
    }

    fn detect_claude_desktop_found() -> bool {
        patch::find_claude_path().is_some()
    }

    fn path_has_windows_command(command: &str) -> bool {
        let Some(path) = env::var_os("PATH") else {
            return false;
        };
        let extensions = windows_command_extensions();
        env::split_paths(&path).any(|dir| {
            extensions
                .iter()
                .map(|extension| dir.join(format!("{command}{extension}")))
                .any(|candidate| candidate.is_file())
        })
    }

    fn windows_command_extensions() -> Vec<String> {
        let mut extensions = env::var_os("PATHEXT")
            .map(|value| {
                value
                    .to_string_lossy()
                    .split(';')
                    .map(str::trim)
                    .filter(|extension| !extension.is_empty())
                    .map(|extension| extension.to_ascii_lowercase())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        extensions.extend([".exe", ".cmd", ".bat"].into_iter().map(String::from));
        extensions.sort();
        extensions.dedup();
        extensions
    }

    fn read_developer_mode_enabled() -> bool {
        crate::developer_settings::developer_settings_candidates()
            .into_iter()
            .any(|path| read_allow_dev_tools(&path))
    }

    fn detect_cc_switch_installed() -> bool {
        let db_path = server::default_db_path();
        let state_dir_exists = paths::home_dir()
            .map(|home| cc_switch_state_dir_from_home(&home).is_dir())
            .unwrap_or(false);
        is_cc_switch_install_marker(db_path.is_file(), state_dir_exists)
    }

    fn cc_switch_state_dir_from_home(home: &Path) -> PathBuf {
        home.join(".cc-switch")
    }

    fn is_cc_switch_install_marker(db_file_exists: bool, state_dir_exists: bool) -> bool {
        db_file_exists || state_dir_exists
    }

    fn read_allow_dev_tools(path: &Path) -> bool {
        let Ok(text) = fs::read_to_string(path) else {
            return false;
        };
        developer_mode_from_json(&text)
    }

    fn developer_settings_write_path() -> Option<PathBuf> {
        let candidates = crate::developer_settings::developer_settings_candidates();
        candidates
            .iter()
            .find(|path| path.is_file())
            .cloned()
            .or_else(|| candidates.into_iter().next())
    }

    fn developer_mode_from_json(text: &str) -> bool {
        serde_json::from_str::<Value>(text)
            .ok()
            .and_then(|value| value.get("allowDevTools").and_then(Value::as_bool))
            .unwrap_or(false)
    }

    fn enable_developer_mode_at_path(path: &Path) -> Result<(), String> {
        let (mut settings, original_text) = if path.is_file() {
            let text = fs::read_to_string(path)
                .map_err(|e| format!("读取 Claude Desktop 开发者配置失败: {e}"))?;
            let value = serde_json::from_str::<Value>(&text)
                .map_err(|e| format!("解析 Claude Desktop 开发者配置失败: {e}"))?;
            (value, Some(text))
        } else {
            (Value::Object(Map::new()), None)
        };

        if developer_mode_from_json(&serde_json::to_string(&settings).unwrap_or_default()) {
            return Ok(());
        }

        let Value::Object(ref mut object) = settings else {
            return Err("Claude Desktop 开发者配置不是 JSON 对象".to_string());
        };
        object.insert("allowDevTools".to_string(), Value::Bool(true));

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建 Claude Desktop 配置目录失败: {e}"))?;
        }

        if let Some(text) = original_text {
            let backup = backup_path(path);
            patch::atomic_write(&backup, text.as_bytes())
                .map_err(|e| format!("备份开发者配置失败: {e}"))?;
        }

        let text = format!(
            "{}\n",
            serde_json::to_string_pretty(&settings)
                .map_err(|e| format!("生成开发者配置失败: {e}"))?
        );
        patch::atomic_write(path, text.as_bytes())
            .map_err(|e| format!("写入开发者配置失败: {e}"))?;

        if read_allow_dev_tools(path) {
            Ok(())
        } else {
            Err("开发者配置已写入，但未读回 allowDevTools=true".to_string())
        }
    }

    fn backup_path(path: &Path) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("developer_settings.json");
        path.with_file_name(format!("{name}.bak-{}-{stamp}", std::process::id()))
    }

    #[cfg(test)]
    mod tests {
        use super::{
            cc_switch_state_dir_from_home, claude_code_settings_has_proxy_managed_env,
            claude_code_settings_path_from_home, developer_mode_from_json,
            enable_developer_mode_at_path, enable_virtual_machine_platform_command,
            ensure_claude_code_proxy_settings_at_path, is_cc_switch_install_marker,
            is_virtual_machine_platform_enabled_marker, windows_command_extensions,
            WINDOWS_CLAUDE_CODE_INSTALL_COMMAND, WINDOWS_ENABLE_VIRTUAL_MACHINE_PLATFORM_COMMAND,
        };
        use serde_json::Value;
        use std::{
            fs,
            path::PathBuf,
            time::{SystemTime, UNIX_EPOCH},
        };

        #[test]
        fn developer_mode_detects_enabled_allow_dev_tools() {
            assert!(developer_mode_from_json(r#"{"allowDevTools":true}"#));
        }

        #[test]
        fn developer_mode_treats_missing_or_invalid_as_disabled() {
            assert!(!developer_mode_from_json(r#"{"allowDevTools":false}"#));
            assert!(!developer_mode_from_json(r#"{"other":true}"#));
            assert!(!developer_mode_from_json("not json"));
        }

        #[test]
        fn claude_code_windows_install_command_uses_official_script() {
            assert!(WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains("https://claude.ai/install.ps1"));
            assert!(WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains("Invoke-WebRequest"));
            assert!(WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains("-OutFile"));
            assert!(WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains("& $installScript"));
            assert!(!WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains(" | iex"));
            assert!(!WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains("irm "));
            assert!(!WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains("Invoke-Expression"));
        }

        #[test]
        fn claude_code_settings_path_uses_user_claude_dir() {
            let home = PathBuf::from(r"C:\Users\Ada");

            assert_eq!(
                claude_code_settings_path_from_home(&home),
                PathBuf::from(r"C:\Users\Ada\.claude\settings.json")
            );
        }

        #[test]
        fn claude_code_settings_detects_proxy_managed_env() {
            assert!(claude_code_settings_has_proxy_managed_env(
                r#"{"env":{"ANTHROPIC_AUTH_TOKEN":"PROXY_MANAGED"}}"#
            ));
            assert!(!claude_code_settings_has_proxy_managed_env(
                r#"{"env":{"ANTHROPIC_AUTH_TOKEN":"other"}}"#
            ));
            assert!(!claude_code_settings_has_proxy_managed_env("not json"));
        }

        #[test]
        fn ensure_claude_code_proxy_settings_creates_missing_file() {
            let dir = test_dir("claude-code-settings-missing");
            let path = dir.join(".claude").join("settings.json");

            ensure_claude_code_proxy_settings_at_path(&path).unwrap();

            let updated: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
            assert_eq!(
                updated
                    .get("env")
                    .and_then(|env| env.get("ANTHROPIC_AUTH_TOKEN"))
                    .and_then(Value::as_str),
                Some("PROXY_MANAGED")
            );
        }

        #[test]
        fn ensure_claude_code_proxy_settings_preserves_existing_fields() {
            let dir = test_dir("claude-code-settings-preserve");
            fs::create_dir_all(dir.join(".claude")).unwrap();
            let path = dir.join(".claude").join("settings.json");
            fs::write(
                &path,
                r#"{"permissions":{"allow":["Bash(ls:*)"]},"env":{"KEEP":"yes"}}"#,
            )
            .unwrap();

            ensure_claude_code_proxy_settings_at_path(&path).unwrap();

            let updated: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
            assert_eq!(
                updated
                    .get("permissions")
                    .and_then(|permissions| permissions.get("allow"))
                    .and_then(Value::as_array)
                    .map(Vec::len),
                Some(1)
            );
            assert_eq!(
                updated
                    .get("env")
                    .and_then(|env| env.get("KEEP"))
                    .and_then(Value::as_str),
                Some("yes")
            );
            assert_eq!(
                updated
                    .get("env")
                    .and_then(|env| env.get("ANTHROPIC_AUTH_TOKEN"))
                    .and_then(Value::as_str),
                Some("PROXY_MANAGED")
            );
        }

        #[test]
        fn claude_code_windows_install_command_writes_proxy_settings_after_install() {
            assert!(WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains(".claude"));
            assert!(WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains("settings.json"));
            assert!(WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains("ANTHROPIC_AUTH_TOKEN"));
            assert!(WINDOWS_CLAUDE_CODE_INSTALL_COMMAND.contains("PROXY_MANAGED"));
        }

        #[test]
        fn cc_switch_install_marker_accepts_db_file_or_state_dir() {
            assert!(is_cc_switch_install_marker(true, false));
            assert!(is_cc_switch_install_marker(false, true));
            assert!(!is_cc_switch_install_marker(false, false));
        }

        #[test]
        fn cc_switch_state_dir_uses_home_path() {
            let home = PathBuf::from(r"C:\Users\Ada");

            assert_eq!(
                cc_switch_state_dir_from_home(&home),
                PathBuf::from(r"C:\Users\Ada\.cc-switch")
            );
        }

        #[test]
        fn windows_command_extensions_include_common_shell_commands() {
            let extensions = windows_command_extensions();

            assert!(extensions.contains(&".exe".to_string()));
            assert!(extensions.contains(&".cmd".to_string()));
            assert!(extensions.contains(&".bat".to_string()));
        }

        #[test]
        fn virtual_machine_platform_requires_wsl_and_vm_platform_service_markers() {
            assert!(is_virtual_machine_platform_enabled_marker(true, true));
            assert!(!is_virtual_machine_platform_enabled_marker(false, true));
            assert!(!is_virtual_machine_platform_enabled_marker(true, false));
            assert!(!is_virtual_machine_platform_enabled_marker(false, false));
        }

        #[test]
        fn virtual_machine_platform_enable_script_targets_only_required_features() {
            assert!(WINDOWS_ENABLE_VIRTUAL_MACHINE_PLATFORM_COMMAND
                .contains("Microsoft-Windows-Subsystem-Linux"));
            assert!(
                WINDOWS_ENABLE_VIRTUAL_MACHINE_PLATFORM_COMMAND.contains("VirtualMachinePlatform")
            );
            assert!(WINDOWS_ENABLE_VIRTUAL_MACHINE_PLATFORM_COMMAND.contains("-All"));
            assert!(WINDOWS_ENABLE_VIRTUAL_MACHINE_PLATFORM_COMMAND.contains("-NoRestart"));
            assert!(!WINDOWS_ENABLE_VIRTUAL_MACHINE_PLATFORM_COMMAND.contains("HypervisorPlatform"));
        }

        #[test]
        fn virtual_machine_platform_elevated_command_runs_temp_script_without_nested_command() {
            let script = PathBuf::from(r"C:\Users\Ada\AppData\Local\Temp\cpp-enable-vm.ps1");
            let command = enable_virtual_machine_platform_command(&script);

            assert!(command.contains("Start-Process powershell.exe -Verb RunAs"));
            assert!(command.contains("-File"));
            assert!(command.contains(r"C:\Users\Ada\AppData\Local\Temp\cpp-enable-vm.ps1"));
            assert!(!command.contains("-Command ''"));
            assert!(!command.contains("Win虚拟机平台已发起开启。请重启电脑后再继续使用。'"));
        }

        #[test]
        fn enable_developer_mode_preserves_fields_and_backs_up_existing_file() {
            let dir = test_dir("preserve");
            fs::create_dir_all(&dir).unwrap();
            let path = dir.join("developer_settings.json");
            fs::write(&path, r#"{"allowDevTools":false,"theme":"dark"}"#).unwrap();

            enable_developer_mode_at_path(&path).unwrap();

            let updated: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
            assert_eq!(
                updated.get("allowDevTools").and_then(Value::as_bool),
                Some(true)
            );
            assert_eq!(updated.get("theme").and_then(Value::as_str), Some("dark"));

            let backup = fs::read_dir(&dir)
                .unwrap()
                .flatten()
                .map(|entry| entry.path())
                .find(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("developer_settings.json.bak-"))
                })
                .expect("backup file");
            let backup_json: Value =
                serde_json::from_str(&fs::read_to_string(backup).unwrap()).unwrap();
            assert_eq!(
                backup_json.get("allowDevTools").and_then(Value::as_bool),
                Some(false)
            );
        }

        #[test]
        fn enable_developer_mode_creates_missing_file() {
            let dir = test_dir("missing");
            fs::create_dir_all(&dir).unwrap();
            let path = dir.join("developer_settings.json");

            enable_developer_mode_at_path(&path).unwrap();

            let updated: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
            assert_eq!(
                updated.get("allowDevTools").and_then(Value::as_bool),
                Some(true)
            );
        }

        fn test_dir(name: &str) -> PathBuf {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            std::env::temp_dir().join(format!(
                "claude-plus-plus-welcome-{name}-{}-{stamp}",
                std::process::id()
            ))
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use std::{
        process::{Command, Stdio},
        sync::LazyLock,
    };

    use serde::Serialize;

    #[cfg(target_os = "macos")]
    const CLAUDE_CODE_INSTALL_COMMAND: &str = "curl -fsSL https://claude.ai/install.sh | bash";

    #[cfg(all(unix, not(target_os = "macos")))]
    const CLAUDE_CODE_INSTALL_COMMAND: &str = "curl -fsSL https://claude.ai/install.sh | bash";

    #[cfg(target_os = "macos")]
    static MACOS_TERMINAL_SCRIPT: LazyLock<String> = LazyLock::new(|| {
        format!(
            "tell application \"Terminal\" to do script \"{}\"",
            CLAUDE_CODE_INSTALL_COMMAND
        )
    });

    #[derive(Serialize)]
    pub struct WelcomeStatus {
        pub claude_code_installed: bool,
        pub virtual_machine_platform_supported: bool,
        pub virtual_machine_platform_enabled: bool,
        pub claude_desktop_found: bool,
        pub developer_mode_enabled: bool,
        pub cc_switch_installed: bool,
    }

    pub fn status() -> WelcomeStatus {
        WelcomeStatus {
            claude_code_installed: detect_claude_code_installed(),
            virtual_machine_platform_supported: false,
            virtual_machine_platform_enabled: false,
            claude_desktop_found: false,
            developer_mode_enabled: false,
            cc_switch_installed: false,
        }
    }

    pub fn install_claude_code() -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            Command::new("osascript")
                .args(["-e", MACOS_TERMINAL_SCRIPT.as_str()])
                .spawn()
                .map(|_| ())
                .map_err(|e| format!("启动 Claude Code 安装命令失败: {e}"))
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            launch_linux_terminal()
        }
    }

    pub fn enable_developer_mode() -> Result<(), String> {
        Err("当前只支持在 Windows 上开启 Claude Desktop 开发者模式".to_string())
    }

    pub fn enable_virtual_machine_platform() -> Result<(), String> {
        Err("当前只支持在 Windows 上开启 Win 虚拟机平台".to_string())
    }

    fn detect_claude_code_installed() -> bool {
        Command::new("claude")
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn launch_linux_terminal() -> Result<(), String> {
        let candidates: &[(&str, &[&str])] = &[
            (
                "x-terminal-emulator",
                &["-e", "sh", "-lc", CLAUDE_CODE_INSTALL_COMMAND],
            ),
            (
                "gnome-terminal",
                &["--", "sh", "-lc", CLAUDE_CODE_INSTALL_COMMAND],
            ),
            ("konsole", &["-e", "sh", "-lc", CLAUDE_CODE_INSTALL_COMMAND]),
            (
                "xfce4-terminal",
                &["-e", "sh", "-lc", CLAUDE_CODE_INSTALL_COMMAND],
            ),
            ("xterm", &["-e", "sh", "-lc", CLAUDE_CODE_INSTALL_COMMAND]),
        ];
        for (program, args) in candidates {
            if Command::new(program)
                .args(*args)
                .spawn()
                .map(|_| ())
                .is_ok()
            {
                return Ok(());
            }
        }
        Err("未找到可用的 Linux 终端程序来启动 Claude Code 安装命令".to_string())
    }
}

pub use imp::{
    enable_developer_mode, enable_virtual_machine_platform, install_claude_code, status,
    WelcomeStatus,
};
