use crate::time_utils::now_ms;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

const APP_STATE_DIR: &str = ".claude-plus-plus";
const DIAGNOSTIC_LOG_FILE: &str = "claude-plus-plus.log";

#[derive(Debug, Clone, Serialize)]
pub struct LogsPayload {
    pub path: String,
    pub text: String,
    pub lines: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticsPayload {
    pub report: String,
}

#[derive(Debug, Clone, Serialize)]
struct DiagnosticRecord {
    timestamp_ms: u64,
    pid: u32,
    event: String,
    detail: Value,
}

pub fn append_event(event: &str, detail: impl Serialize) -> std::io::Result<()> {
    let path = diagnostic_log_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let detail = serde_json::to_value(detail).unwrap_or_else(|error| {
        json!({
            "serialization_error": error.to_string()
        })
    });
    let record = DiagnosticRecord {
        timestamp_ms: now_ms(),
        pid: std::process::id(),
        event: event.to_string(),
        detail,
    };
    let line = serde_json::to_string(&record).unwrap_or_else(|error| {
        json!({
            "timestamp_ms": now_ms(),
            "pid": std::process::id(),
            "event": "diagnostic_log.serialization_failed",
            "detail": {
                "message": error.to_string()
            }
        })
        .to_string()
    });

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub fn read_latest_logs(lines: usize) -> LogsPayload {
    let path = diagnostic_log_path();
    let text = read_tail(&path, lines).unwrap_or_default();
    LogsPayload {
        path: path.to_string_lossy().to_string(),
        text,
        lines,
    }
}

pub fn report(
    status: Value,
    mappings: Result<Value, String>,
    zh_status: Value,
    enhance_status: Value,
) -> DiagnosticsPayload {
    let mappings_value = match mappings {
        Ok(value) => value,
        Err(error) => json!({ "status": "error", "error": error }),
    };
    let report = json!({
        "generatedAtMs": now_ms(),
        "version": env!("CARGO_PKG_VERSION"),
        "overview": {
            "app": "Claude++",
            "pid": std::process::id(),
            "status": status,
            "mappings": mappings_value,
            "claude_zh": zh_status,
            "claude_enhance": enhance_status
        },
        "paths": {
            "ccSwitchDb": crate::server::default_db_path(),
            "diagnosticLog": diagnostic_log_path()
        }
    });

    DiagnosticsPayload {
        report: serde_json::to_string_pretty(&report)
            .unwrap_or_else(|error| format!("诊断报告序列化失败: {error}")),
    }
}

fn diagnostic_log_path() -> PathBuf {
    default_app_state_dir().join(DIAGNOSTIC_LOG_FILE)
}

fn default_app_state_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        return PathBuf::from(home).join(APP_STATE_DIR);
    }
    PathBuf::from(APP_STATE_DIR)
}

fn read_tail(path: &PathBuf, lines: usize) -> std::io::Result<String> {
    if lines == 0 {
        return Ok(String::new());
    }

    let mut file = fs::File::open(path)?;
    let mut position = file.seek(SeekFrom::End(0))?;
    if position == 0 {
        return Ok(String::new());
    }

    const BLOCK_SIZE: usize = 8192;
    let mut chunks: Vec<u8> = Vec::new();
    let mut newline_count = 0usize;

    while position > 0 && newline_count <= lines {
        let read_size = BLOCK_SIZE.min(position as usize);
        position -= read_size as u64;
        file.seek(SeekFrom::Start(position))?;

        let mut buffer = vec![0u8; read_size];
        file.read_exact(&mut buffer)?;
        newline_count += buffer.iter().filter(|byte| **byte == b'\n').count();

        buffer.extend_from_slice(&chunks);
        chunks = buffer;
    }

    let text = String::from_utf8_lossy(&chunks);
    let mut selected: Vec<&str> = text.lines().rev().take(lines).collect();
    selected.reverse();
    Ok(selected.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_tail_returns_requested_last_lines() {
        let path = std::env::temp_dir().join(format!(
            "claude-plus-plus-read-tail-{}.log",
            std::process::id()
        ));
        fs::write(&path, "one\ntwo\nthree\nfour\n").unwrap();

        assert_eq!(read_tail(&path, 2).unwrap(), "three\nfour");
        assert_eq!(read_tail(&path, 0).unwrap(), "");

        let _ = fs::remove_file(path);
    }
}
