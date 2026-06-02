use crate::constants::DEFAULT_PROXY_PORT;
use serde::Deserialize;
use std::path::PathBuf;

const APP_STATE_DIR: &str = ".claude-plus-plus";
const SETTINGS_FILE: &str = "settings.json";
const PROXY_PORT_ENV: &str = "CLAUDE_PLUS_PROXY_PORT";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsFile {
    #[serde(alias = "proxy_port")]
    proxy_port: Option<u16>,
}

pub fn proxy_port() -> u16 {
    proxy_port_from_env()
        .or_else(|| proxy_port_from_file().ok().flatten())
        .unwrap_or(DEFAULT_PROXY_PORT)
}

pub fn settings_path() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)?;
    Some(home.join(APP_STATE_DIR).join(SETTINGS_FILE))
}

fn proxy_port_from_env() -> Option<u16> {
    std::env::var(PROXY_PORT_ENV)
        .ok()
        .and_then(|value| parse_port(&value))
}

fn proxy_port_from_file() -> Result<Option<u16>, String> {
    let Some(path) = settings_path() else {
        return Ok(None);
    };
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("读取 Claude++ 设置失败: {error}"))?;
    let settings = serde_json::from_str::<SettingsFile>(&text)
        .map_err(|error| format!("解析 Claude++ 设置失败: {error}"))?;
    Ok(settings.proxy_port.filter(|port| *port > 0))
}

fn parse_port(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok().filter(|port| *port > 0)
}

#[cfg(test)]
mod tests {
    use super::parse_port;

    #[test]
    fn parse_port_rejects_zero_and_invalid_values() {
        assert_eq!(parse_port("15722"), Some(15722));
        assert_eq!(parse_port(" 15723 "), Some(15723));
        assert_eq!(parse_port("0"), None);
        assert_eq!(parse_port("abc"), None);
    }
}
