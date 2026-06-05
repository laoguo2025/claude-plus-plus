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
const DEFAULT_TOKEN_USAGE_RECENT_LIMIT: usize = 20;
const DEFAULT_TOKEN_USAGE_DEBUG_LIMIT: usize = 50;
const DEFAULT_TOKEN_USAGE_LEDGER_LIMIT: usize = 500;
const DEFAULT_TOKEN_USAGE_CONTEXT_POLL_INTERVAL_MS: u64 = 1_000;
const DEFAULT_TOKEN_USAGE_TURN_IDLE_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_TOKEN_USAGE_CONTEXT_MERGE_WINDOW_MS: u64 = 30_000;
const DEFAULT_TOKEN_USAGE_CROSS_SOURCE_DEDUPE_WINDOW_MS: u64 = 3_000;
const DEFAULT_TOKEN_USAGE_FINAL_RENDER_DELAY_MS: u64 = 900;
const DEFAULT_TOKEN_USAGE_MAX_CAPTURE_TEXT_LENGTH: usize = 524_288;
const DEFAULT_TOKEN_USAGE_MAX_CAPTURE_BLOB_BYTES: usize = 512_000;
const DEFAULT_TOKEN_USAGE_MAX_COLLECT_DEPTH: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyRuntimeTuning {
    pub title_i18n_rate_limit_window_secs: u64,
    pub title_i18n_rate_limit_max: u32,
    pub token_usage_max_pending_line: usize,
    pub token_usage_db_fresh_window_ms: u64,
    pub token_usage_capture: TokenUsageCaptureTuning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenUsageCaptureTuning {
    pub recent_limit: usize,
    pub debug_limit: usize,
    pub ledger_limit: usize,
    pub context_poll_interval_ms: u64,
    pub turn_idle_timeout_ms: u64,
    pub context_merge_window_ms: u64,
    pub cross_source_dedupe_window_ms: u64,
    pub final_render_delay_ms: u64,
    pub max_capture_text_length: usize,
    pub max_capture_blob_bytes: usize,
    pub max_collect_depth: usize,
}

impl Default for TokenUsageCaptureTuning {
    fn default() -> Self {
        Self {
            recent_limit: DEFAULT_TOKEN_USAGE_RECENT_LIMIT,
            debug_limit: DEFAULT_TOKEN_USAGE_DEBUG_LIMIT,
            ledger_limit: DEFAULT_TOKEN_USAGE_LEDGER_LIMIT,
            context_poll_interval_ms: DEFAULT_TOKEN_USAGE_CONTEXT_POLL_INTERVAL_MS,
            turn_idle_timeout_ms: DEFAULT_TOKEN_USAGE_TURN_IDLE_TIMEOUT_MS,
            context_merge_window_ms: DEFAULT_TOKEN_USAGE_CONTEXT_MERGE_WINDOW_MS,
            cross_source_dedupe_window_ms: DEFAULT_TOKEN_USAGE_CROSS_SOURCE_DEDUPE_WINDOW_MS,
            final_render_delay_ms: DEFAULT_TOKEN_USAGE_FINAL_RENDER_DELAY_MS,
            max_capture_text_length: DEFAULT_TOKEN_USAGE_MAX_CAPTURE_TEXT_LENGTH,
            max_capture_blob_bytes: DEFAULT_TOKEN_USAGE_MAX_CAPTURE_BLOB_BYTES,
            max_collect_depth: DEFAULT_TOKEN_USAGE_MAX_COLLECT_DEPTH,
        }
    }
}

impl Default for ProxyRuntimeTuning {
    fn default() -> Self {
        Self {
            title_i18n_rate_limit_window_secs: DEFAULT_TITLE_I18N_RATE_LIMIT_WINDOW_SECS,
            title_i18n_rate_limit_max: DEFAULT_TITLE_I18N_RATE_LIMIT_MAX,
            token_usage_max_pending_line: DEFAULT_TOKEN_USAGE_MAX_PENDING_LINE,
            token_usage_db_fresh_window_ms: DEFAULT_TOKEN_USAGE_DB_FRESH_WINDOW_MS,
            token_usage_capture: TokenUsageCaptureTuning::default(),
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
    #[cfg(target_os = "windows")]
    #[serde(alias = "claude_desktop_path")]
    claude_desktop_path: Option<PathBuf>,
    #[cfg(target_os = "windows")]
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
    #[serde(default, alias = "token_usage_capture")]
    token_usage_capture: TokenUsageCaptureFile,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TokenUsageCaptureFile {
    #[serde(alias = "recent_limit")]
    recent_limit: Option<usize>,
    #[serde(alias = "debug_limit")]
    debug_limit: Option<usize>,
    #[serde(alias = "ledger_limit")]
    ledger_limit: Option<usize>,
    #[serde(alias = "context_poll_interval_ms")]
    context_poll_interval_ms: Option<u64>,
    #[serde(alias = "turn_idle_timeout_ms")]
    turn_idle_timeout_ms: Option<u64>,
    #[serde(alias = "context_merge_window_ms")]
    context_merge_window_ms: Option<u64>,
    #[serde(alias = "cross_source_dedupe_window_ms")]
    cross_source_dedupe_window_ms: Option<u64>,
    #[serde(alias = "final_render_delay_ms")]
    final_render_delay_ms: Option<u64>,
    #[serde(alias = "max_capture_text_length")]
    max_capture_text_length: Option<usize>,
    #[serde(alias = "max_capture_blob_bytes")]
    max_capture_blob_bytes: Option<usize>,
    #[serde(alias = "max_collect_depth")]
    max_collect_depth: Option<usize>,
}

pub fn proxy_port() -> u16 {
    proxy_port_from_env()
        .or_else(|| proxy_port_from_file().ok().flatten())
        .unwrap_or(DEFAULT_PROXY_PORT)
}

pub fn settings_path() -> PathBuf {
    crate::paths::app_state_dir().join(SETTINGS_FILE)
}

#[cfg(target_os = "windows")]
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
        token_usage_capture: token_usage_capture_tuning_from_settings(
            settings.token_usage_capture,
            defaults.token_usage_capture,
        ),
    }
}

fn token_usage_capture_tuning_from_settings(
    settings: TokenUsageCaptureFile,
    defaults: TokenUsageCaptureTuning,
) -> TokenUsageCaptureTuning {
    TokenUsageCaptureTuning {
        recent_limit: settings
            .recent_limit
            .filter(|value| *value > 0)
            .unwrap_or(defaults.recent_limit),
        debug_limit: settings
            .debug_limit
            .filter(|value| *value > 0)
            .unwrap_or(defaults.debug_limit),
        ledger_limit: settings
            .ledger_limit
            .filter(|value| *value > 0)
            .unwrap_or(defaults.ledger_limit),
        context_poll_interval_ms: settings
            .context_poll_interval_ms
            .filter(|value| *value > 0)
            .unwrap_or(defaults.context_poll_interval_ms),
        turn_idle_timeout_ms: settings
            .turn_idle_timeout_ms
            .filter(|value| *value > 0)
            .unwrap_or(defaults.turn_idle_timeout_ms),
        context_merge_window_ms: settings
            .context_merge_window_ms
            .filter(|value| *value > 0)
            .unwrap_or(defaults.context_merge_window_ms),
        cross_source_dedupe_window_ms: settings
            .cross_source_dedupe_window_ms
            .filter(|value| *value > 0)
            .unwrap_or(defaults.cross_source_dedupe_window_ms),
        final_render_delay_ms: settings
            .final_render_delay_ms
            .filter(|value| *value > 0)
            .unwrap_or(defaults.final_render_delay_ms),
        max_capture_text_length: settings
            .max_capture_text_length
            .filter(|value| *value > 0)
            .unwrap_or(defaults.max_capture_text_length),
        max_capture_blob_bytes: settings
            .max_capture_blob_bytes
            .filter(|value| *value > 0)
            .unwrap_or(defaults.max_capture_blob_bytes),
        max_collect_depth: settings
            .max_collect_depth
            .filter(|value| *value > 0)
            .unwrap_or(defaults.max_collect_depth),
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
                "tokenUsageDbFreshWindowMs": 30000,
                "tokenUsageCapture": {
                    "recentLimit": 7,
                    "debugLimit": 8,
                    "ledgerLimit": 9,
                    "contextPollIntervalMs": 100,
                    "turnIdleTimeoutMs": 200,
                    "contextMergeWindowMs": 300,
                    "crossSourceDedupeWindowMs": 400,
                    "finalRenderDelayMs": 500,
                    "maxCaptureTextLength": 600,
                    "maxCaptureBlobBytes": 700,
                    "maxCollectDepth": 3
                }
            }"#,
        )
        .expect("camel config");
        assert_eq!(camel.title_i18n_rate_limit_window_secs, 30);
        assert_eq!(camel.title_i18n_rate_limit_max, 12);
        assert_eq!(camel.token_usage_max_pending_line, 4096);
        assert_eq!(camel.token_usage_db_fresh_window_ms, 30_000);
        assert_eq!(camel.token_usage_capture.recent_limit, 7);
        assert_eq!(camel.token_usage_capture.debug_limit, 8);
        assert_eq!(camel.token_usage_capture.ledger_limit, 9);
        assert_eq!(camel.token_usage_capture.context_poll_interval_ms, 100);
        assert_eq!(camel.token_usage_capture.turn_idle_timeout_ms, 200);
        assert_eq!(camel.token_usage_capture.context_merge_window_ms, 300);
        assert_eq!(camel.token_usage_capture.cross_source_dedupe_window_ms, 400);
        assert_eq!(camel.token_usage_capture.final_render_delay_ms, 500);
        assert_eq!(camel.token_usage_capture.max_capture_text_length, 600);
        assert_eq!(camel.token_usage_capture.max_capture_blob_bytes, 700);
        assert_eq!(camel.token_usage_capture.max_collect_depth, 3);

        let snake = proxy_runtime_tuning_from_file(
            r#"{
                "title_i18n_rate_limit_window_secs": 45,
                "title_i18n_rate_limit_max": 20,
                "token_usage_max_pending_line": 8192,
                "token_usage_db_fresh_window_ms": 25000,
                "token_usage_capture": {
                    "recent_limit": 11,
                    "debug_limit": 12,
                    "ledger_limit": 13,
                    "context_poll_interval_ms": 1100,
                    "turn_idle_timeout_ms": 1200,
                    "context_merge_window_ms": 1300,
                    "cross_source_dedupe_window_ms": 1400,
                    "final_render_delay_ms": 1500,
                    "max_capture_text_length": 1600,
                    "max_capture_blob_bytes": 1700,
                    "max_collect_depth": 4
                }
            }"#,
        )
        .expect("snake config");
        assert_eq!(snake.title_i18n_rate_limit_window_secs, 45);
        assert_eq!(snake.title_i18n_rate_limit_max, 20);
        assert_eq!(snake.token_usage_max_pending_line, 8192);
        assert_eq!(snake.token_usage_db_fresh_window_ms, 25_000);
        assert_eq!(snake.token_usage_capture.recent_limit, 11);
        assert_eq!(snake.token_usage_capture.debug_limit, 12);
        assert_eq!(snake.token_usage_capture.ledger_limit, 13);
        assert_eq!(snake.token_usage_capture.context_poll_interval_ms, 1100);
        assert_eq!(snake.token_usage_capture.turn_idle_timeout_ms, 1200);
        assert_eq!(snake.token_usage_capture.context_merge_window_ms, 1300);
        assert_eq!(
            snake.token_usage_capture.cross_source_dedupe_window_ms,
            1400
        );
        assert_eq!(snake.token_usage_capture.final_render_delay_ms, 1500);
        assert_eq!(snake.token_usage_capture.max_capture_text_length, 1600);
        assert_eq!(snake.token_usage_capture.max_capture_blob_bytes, 1700);
        assert_eq!(snake.token_usage_capture.max_collect_depth, 4);
    }

    #[test]
    fn proxy_runtime_tuning_rejects_zero_values_to_defaults() {
        let tuning = proxy_runtime_tuning_from_file(
            r#"{
                "titleI18nRateLimitWindowSecs": 0,
                "titleI18nRateLimitMax": 0,
                "tokenUsageMaxPendingLine": 0,
                "tokenUsageDbFreshWindowMs": 0,
                "tokenUsageCapture": {
                    "recentLimit": 0,
                    "debugLimit": 0,
                    "ledgerLimit": 0,
                    "contextPollIntervalMs": 0,
                    "turnIdleTimeoutMs": 0,
                    "contextMergeWindowMs": 0,
                    "crossSourceDedupeWindowMs": 0,
                    "finalRenderDelayMs": 0,
                    "maxCaptureTextLength": 0,
                    "maxCaptureBlobBytes": 0,
                    "maxCollectDepth": 0
                }
            }"#,
        )
        .expect("zero config");
        assert_eq!(tuning, Default::default());
    }

    #[test]
    #[cfg(target_os = "windows")]
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
