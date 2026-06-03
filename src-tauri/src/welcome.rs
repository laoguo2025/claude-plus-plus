#[cfg(target_os = "windows")]
mod imp {
    use crate::{ccswitch_db, server};
    use serde::Serialize;
    use serde_json::{Map, Value};
    use std::{
        env, fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[derive(Serialize)]
    pub struct WelcomeStatus {
        pub developer_mode_enabled: bool,
        pub cc_switch_installed: bool,
    }

    pub fn status() -> WelcomeStatus {
        WelcomeStatus {
            developer_mode_enabled: read_developer_mode_enabled(),
            cc_switch_installed: detect_cc_switch_installed(),
        }
    }

    pub fn enable_developer_mode() -> Result<(), String> {
        let path = developer_settings_write_path()
            .ok_or_else(|| "无法定位 Claude Desktop 开发者配置路径".to_string())?;
        enable_developer_mode_at_path(&path)
    }

    fn read_developer_mode_enabled() -> bool {
        developer_settings_candidates()
            .into_iter()
            .any(|path| read_allow_dev_tools(&path))
    }

    fn detect_cc_switch_installed() -> bool {
        if server::default_db_path().is_file() {
            return true;
        }
        if ccswitch_db::load_proxy_config(&server::default_db_path()).is_ok() {
            return true;
        }
        env::var_os("USERPROFILE")
            .map(|home| Path::new(&home).join(".cc-switch").is_dir())
            .unwrap_or(false)
    }

    fn read_allow_dev_tools(path: &Path) -> bool {
        let Ok(text) = fs::read_to_string(path) else {
            return false;
        };
        developer_mode_from_json(&text)
    }

    fn developer_settings_candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Some(appdata) = env::var_os("APPDATA").map(PathBuf::from) {
            candidates.push(appdata.join("Claude").join("developer_settings.json"));
            candidates.push(appdata.join("Claude-3p").join("developer_settings.json"));
        }
        if let Some(local_appdata) = env::var_os("LOCALAPPDATA").map(PathBuf::from) {
            candidates.push(
                local_appdata
                    .join("Packages")
                    .join(crate::constants::CLAUDE_STORE_PACKAGE_NAME)
                    .join("LocalCache")
                    .join("Roaming")
                    .join("Claude")
                    .join("developer_settings.json"),
            );
        }
        candidates
    }

    fn developer_settings_write_path() -> Option<PathBuf> {
        let candidates = developer_settings_candidates();
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
            fs::write(&backup, text).map_err(|e| format!("备份开发者配置失败: {e}"))?;
        }

        let text = format!(
            "{}\n",
            serde_json::to_string_pretty(&settings)
                .map_err(|e| format!("生成开发者配置失败: {e}"))?
        );
        fs::write(path, text).map_err(|e| format!("写入开发者配置失败: {e}"))?;

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
        use super::{developer_mode_from_json, enable_developer_mode_at_path};
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
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct WelcomeStatus {
        pub developer_mode_enabled: bool,
        pub cc_switch_installed: bool,
    }

    pub fn status() -> WelcomeStatus {
        WelcomeStatus {
            developer_mode_enabled: false,
            cc_switch_installed: false,
        }
    }

    pub fn enable_developer_mode() -> Result<(), String> {
        Err("当前只支持在 Windows 上开启 Claude Desktop 开发者模式".to_string())
    }
}

pub use imp::{enable_developer_mode, status, WelcomeStatus};
