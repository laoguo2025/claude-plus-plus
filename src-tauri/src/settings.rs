use crate::constants::DEFAULT_PROXY_PORT;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

const SETTINGS_FILE: &str = "settings.json";
const PROXY_PORT_ENV: &str = "CLAUDE_PLUS_PROXY_PORT";
const DEFAULT_TITLE_I18N_RATE_LIMIT_WINDOW_SECS: u64 = 60;
const DEFAULT_TITLE_I18N_RATE_LIMIT_MAX: u32 = 90;
const DEFAULT_TOKEN_USAGE_MAX_PENDING_LINE: usize = 64 * 1024;
const DEFAULT_TOKEN_USAGE_DB_FRESH_WINDOW_MS: u64 = 15_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyRuntimeTuning {
    pub title_i18n_rate_limit_window_secs: u64,
    pub title_i18n_rate_limit_max: u32,
    pub token_usage_max_pending_line: usize,
    pub token_usage_db_fresh_window_ms: u64,
}

impl Default for ProxyRuntimeTuning {
    fn default() -> Self {
        Self {
            title_i18n_rate_limit_window_secs: DEFAULT_TITLE_I18N_RATE_LIMIT_WINDOW_SECS,
            title_i18n_rate_limit_max: DEFAULT_TITLE_I18N_RATE_LIMIT_MAX,
            token_usage_max_pending_line: DEFAULT_TOKEN_USAGE_MAX_PENDING_LINE,
            token_usage_db_fresh_window_ms: DEFAULT_TOKEN_USAGE_DB_FRESH_WINDOW_MS,
        }
    }
}

impl ProxyRuntimeTuning {
    pub fn title_i18n_rate_limit_window(&self) -> Duration {
        Duration::from_secs(self.title_i18n_rate_limit_window_secs)
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SettingsFile {
    #[serde(alias = "proxy_port")]
    proxy_port: Option<u16>,
    #[serde(alias = "claude_desktop_path")]
    claude_desktop_path: Option<PathBuf>,
    #[serde(alias = "claude_desktop_resources_path")]
    claude_desktop_resources_path: Option<PathBuf>,
    #[serde(alias = "title_i18n_rate_limit_window_secs")]
    title_i18n_rate_limit_window_secs: Option<u64>,
    #[serde(alias = "title_i18n_rate_limit_max")]
    title_i18n_rate_limit_max: Option<u32>,
    #[serde(alias = "token_usage_max_pending_line")]
    token_usage_max_pending_line: Option<usize>,
    #[serde(alias = "token_usage_db_fresh_window_ms")]
    token_usage_db_fresh_window_ms: Option<u64>,
}

pub fn proxy_port() -> u16 {
    proxy_port_from_env()
        .or_else(|| proxy_port_from_file().ok().flatten())
        .unwrap_or(DEFAULT_PROXY_PORT)
}

pub fn settings_path() -> PathBuf {
    crate::paths::app_state_dir().join(SETTINGS_FILE)
}

pub fn claude_desktop_path_overrides() -> Vec<PathBuf> {
    read_settings_file()
        .map(|settings| {
            [
                settings.claude_desktop_path,
                settings.claude_desktop_resources_path,
            ]
            .into_iter()
            .flatten()
            .collect()
        })
        .unwrap_or_default()
}

pub fn proxy_runtime_tuning() -> ProxyRuntimeTuning {
    proxy_runtime_tuning_from_path().unwrap_or_else(|_| ProxyRuntimeTuning::default())
}

fn proxy_port_from_env() -> Option<u16> {
    std::env::var(PROXY_PORT_ENV)
        .ok()
        .and_then(|value| crate::net_utils::parse_port(&value))
}

fn proxy_port_from_file() -> Result<Option<u16>, String> {
    let settings = read_settings_file()?;
    Ok(settings.proxy_port.filter(|port| *port > 0))
}

fn proxy_runtime_tuning_from_path() -> Result<ProxyRuntimeTuning, String> {
    let settings = read_settings_file()?;
    Ok(proxy_runtime_tuning_from_settings(settings))
}

fn read_settings_file() -> Result<SettingsFile, String> {
    let path = settings_path();
    if !path.is_file() {
        return Ok(SettingsFile::default());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("读取 Claude++ 设置失败: {error}"))?;
    serde_json::from_str::<SettingsFile>(&text)
        .map_err(|error| format!("解析 Claude++ 设置失败: {error}"))
}

fn proxy_runtime_tuning_from_settings(settings: SettingsFile) -> ProxyRuntimeTuning {
    let defaults = ProxyRuntimeTuning::default();
    ProxyRuntimeTuning {
        title_i18n_rate_limit_window_secs: settings
            .title_i18n_rate_limit_window_secs
            .filter(|value| *value > 0)
            .unwrap_or(defaults.title_i18n_rate_limit_window_secs),
        title_i18n_rate_limit_max: settings
            .title_i18n_rate_limit_max
            .filter(|value| *value > 0)
            .unwrap_or(defaults.title_i18n_rate_limit_max),
        token_usage_max_pending_line: settings
            .token_usage_max_pending_line
            .filter(|value| *value > 0)
            .unwrap_or(defaults.token_usage_max_pending_line),
        token_usage_db_fresh_window_ms: settings
            .token_usage_db_fresh_window_ms
            .filter(|value| *value > 0)
            .unwrap_or(defaults.token_usage_db_fresh_window_ms),
    }
}

#[cfg(test)]
fn proxy_runtime_tuning_from_file(text: &str) -> Result<ProxyRuntimeTuning, String> {
    serde_json::from_str::<SettingsFile>(text)
        .map(proxy_runtime_tuning_from_settings)
        .map_err(|error| format!("解析 Claude++ 设置失败: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{proxy_runtime_tuning_from_file, SettingsFile};
    use crate::net_utils::parse_port;
    use std::path::PathBuf;

    #[test]
    fn parse_port_rejects_zero_and_invalid_values() {
        assert_eq!(parse_port("15722"), Some(15722));
        assert_eq!(parse_port(" 15723 "), Some(15723));
        assert_eq!(parse_port("0"), None);
        assert_eq!(parse_port("abc"), None);
    }

    #[test]
    fn proxy_runtime_tuning_reads_camel_and_snake_case_values() {
        let camel = proxy_runtime_tuning_from_file(
            r#"{
                "titleI18nRateLimitWindowSecs": 30,
                "titleI18nRateLimitMax": 12,
                "tokenUsageMaxPendingLine": 4096,
                "tokenUsageDbFreshWindowMs": 30000
            }"#,
        )
        .expect("camel config");
        assert_eq!(camel.title_i18n_rate_limit_window_secs, 30);
        assert_eq!(camel.title_i18n_rate_limit_max, 12);
        assert_eq!(camel.token_usage_max_pending_line, 4096);
        assert_eq!(camel.token_usage_db_fresh_window_ms, 30_000);

        let snake = proxy_runtime_tuning_from_file(
            r#"{
                "title_i18n_rate_limit_window_secs": 45,
                "title_i18n_rate_limit_max": 20,
                "token_usage_max_pending_line": 8192,
                "token_usage_db_fresh_window_ms": 25000
            }"#,
        )
        .expect("snake config");
        assert_eq!(snake.title_i18n_rate_limit_window_secs, 45);
        assert_eq!(snake.title_i18n_rate_limit_max, 20);
        assert_eq!(snake.token_usage_max_pending_line, 8192);
        assert_eq!(snake.token_usage_db_fresh_window_ms, 25_000);
    }

    #[test]
    fn proxy_runtime_tuning_rejects_zero_values_to_defaults() {
        let tuning = proxy_runtime_tuning_from_file(
            r#"{
                "titleI18nRateLimitWindowSecs": 0,
                "titleI18nRateLimitMax": 0,
                "tokenUsageMaxPendingLine": 0,
                "tokenUsageDbFreshWindowMs": 0
            }"#,
        )
        .expect("zero config");
        assert_eq!(tuning, Default::default());
    }

    #[test]
    fn settings_reads_claude_desktop_path_overrides() {
        let settings = serde_json::from_str::<SettingsFile>(
            r#"{
                "claudeDesktopPath": "D:\\Apps\\Claude",
                "claude_desktop_resources_path": "E:\\Portable\\Claude\\resources"
            }"#,
        )
        .expect("settings");

        assert_eq!(
            settings.claude_desktop_path,
            Some(PathBuf::from(r"D:\Apps\Claude"))
        );
        assert_eq!(
            settings.claude_desktop_resources_path,
            Some(PathBuf::from(r"E:\Portable\Claude\resources"))
        );
    }
}
