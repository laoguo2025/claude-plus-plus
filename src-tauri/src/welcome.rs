#[cfg(target_os = "windows")]
mod imp {
    use crate::{ccswitch_db, server};
    use serde::Serialize;
    use serde_json::Value;
    use std::{env, fs, path::{Path, PathBuf}};

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

    fn developer_mode_from_json(text: &str) -> bool {
        serde_json::from_str::<Value>(text)
            .ok()
            .and_then(|value| value.get("allowDevTools").and_then(Value::as_bool))
            .unwrap_or(false)
    }

    #[cfg(test)]
    mod tests {
        use super::developer_mode_from_json;

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
}

pub use imp::{status, WelcomeStatus};
