use crate::time_utils::now_ms;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

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
    let mappings_value = result_value(mappings);
    let generated_at_ms = now_ms();
    let paths = collect_paths();
    let settings = collect_settings();
    let gateway = collect_gateway(&status);
    let ccswitch = collect_ccswitch(&mappings_value);
    let config_library =
        collect_config_library(gateway.get("expectedPort").and_then(Value::as_u64));
    let claude_desktop = collect_claude_desktop();
    let developer_mode = collect_developer_mode();
    let logs = collect_log_summary(200);
    let mut findings = Vec::new();
    collect_findings(
        &mut findings,
        &status,
        &gateway,
        &ccswitch,
        &config_library,
        &claude_desktop,
        &zh_status,
        &enhance_status,
        &developer_mode,
    );
    let summary = diagnostic_summary(&findings);

    let report = json!({
        "schemaVersion": 2,
        "generatedAtMs": generated_at_ms,
        "version": env!("CARGO_PKG_VERSION"),
        "summary": summary,
        "findings": findings,
        "overview": {
            "app": "Claude++",
            "pid": std::process::id(),
            "status": status,
            "mappings": mappings_value,
            "claude_zh": zh_status,
            "claude_enhance": enhance_status,
            "developer_mode": developer_mode
        },
        "checks": {
            "settings": settings,
            "gateway": gateway,
            "ccSwitch": ccswitch,
            "configLibrary": config_library,
            "claudeDesktop": claude_desktop,
            "logs": logs
        },
        "paths": paths
    });

    DiagnosticsPayload {
        report: serde_json::to_string_pretty(&report)
            .unwrap_or_else(|error| format!("诊断报告序列化失败: {error}")),
    }
}

fn collect_paths() -> Value {
    json!({
        "home": crate::paths::home_dir(),
        "appStateDir": crate::paths::app_state_dir(),
        "ccSwitchDb": crate::server::default_db_path(),
        "settings": crate::settings::settings_path(),
        "diagnosticLog": diagnostic_log_path(),
        "localGatewayToken": crate::server::local_gateway_token_path()
    })
}

fn collect_settings() -> Value {
    let path = crate::settings::settings_path();
    let env_port = std::env::var("CLAUDE_PLUS_PROXY_PORT").ok();
    let file = read_json_file_status(&path);
    let file_proxy_port = file
        .get("json")
        .and_then(|value| value.get("proxyPort").or_else(|| value.get("proxy_port")))
        .and_then(Value::as_u64);
    let env_proxy_port = env_port.as_deref().and_then(parse_port);
    let source = if env_proxy_port.is_some() {
        "env"
    } else if file_proxy_port
        .and_then(|value| u16::try_from(value).ok())
        .is_some()
    {
        "settings"
    } else {
        "default"
    };

    json!({
        "path": path,
        "exists": path.is_file(),
        "envProxyPortSet": env_port.is_some(),
        "envProxyPortValid": env_proxy_port.is_some(),
        "settingsProxyPort": file_proxy_port,
        "effectiveProxyPort": crate::settings::proxy_port(),
        "effectiveProxyPortSource": source,
        "file": strip_json_value(file, "json")
    })
}

fn collect_gateway(status: &Value) -> Value {
    let expected_port = crate::settings::proxy_port();
    let token_path = crate::server::local_gateway_token_path();
    let token_status = match fs::read_to_string(&token_path) {
        Ok(text) => {
            let token = text.trim();
            json!({
                "exists": true,
                "validFormat": valid_local_gateway_token(token),
                "length": token.len()
            })
        }
        Err(error) => json!({
            "exists": false,
            "validFormat": false,
            "error": error.to_string()
        }),
    };

    json!({
        "expectedPort": expected_port,
        "statusRunning": status.get("running").and_then(Value::as_bool),
        "statusPort": status.get("port").and_then(Value::as_u64),
        "tcpAcceptsExpectedPort": tcp_accepts("127.0.0.1", expected_port),
        "localGatewayToken": token_status
    })
}

fn collect_ccswitch(mappings_value: &Value) -> Value {
    let db_path = crate::server::default_db_path();
    let db = file_brief(&db_path);
    let proxy_config = result_value(crate::ccswitch_db::load_proxy_config(&db_path).map(
        |config| {
            json!({
                "proxyEnabled": config.proxy_enabled,
                "claudeRouteEnabled": config.claude_route_enabled,
                "listenAddress": config.listen_address,
                "listenPort": config.listen_port,
                "reachable": config.proxy_enabled
                    && tcp_accepts(&config.listen_address, config.listen_port)
            })
        },
    ));
    let profile = match crate::server::read_ccswitch_gateway_profile() {
        Some(profile) => json!({
            "status": "ok",
            "baseUrl": sanitize_url(&profile.base_url),
            "apiKeyPresent": !profile.api_key.trim().is_empty(),
            "apiKeyLength": profile.api_key.len()
        }),
        None => json!({
            "status": "error",
            "error": "无法从同一个 CC Switch configLibrary entry 读取 baseUrl 和 apiKey"
        }),
    };
    let mapping_count = mappings_value
        .get("mappings")
        .and_then(Value::as_array)
        .map(Vec::len);

    json!({
        "database": db,
        "proxyConfig": proxy_config,
        "gatewayProfile": profile,
        "mappings": {
            "status": mappings_value.get("status").and_then(Value::as_str).unwrap_or("ok"),
            "count": mapping_count,
            "providerName": mappings_value.get("provider_name"),
            "providerId": mappings_value.get("provider_id"),
            "error": mappings_value.get("error")
        }
    })
}

fn collect_config_library(expected_port: Option<u64>) -> Value {
    let dirs = crate::cd_config::candidate_dirs();
    let resolved = crate::cd_config::resolve_config_library_dir()
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|error| error.to_string());
    let candidates = dirs
        .iter()
        .map(|dir| inspect_config_library_dir(dir, expected_port))
        .collect::<Vec<_>>();

    json!({
        "resolved": result_string(resolved),
        "isApplied": crate::cd_config::is_applied(),
        "candidates": candidates
    })
}

fn inspect_config_library_dir(dir: &std::path::Path, expected_port: Option<u64>) -> Value {
    use crate::constants::{CC_SWITCH_CLAUDE_DESKTOP_ENTRY_ID, CLAUDE_PLUS_PLUS_ENTRY_ID};

    let meta_path = dir.join("_meta.json");
    let meta_status = read_json_file_status(&meta_path);
    let applied_id = meta_status
        .get("json")
        .and_then(|value| value.get("appliedId"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let entry_count = meta_status
        .get("json")
        .and_then(|value| value.get("entries"))
        .and_then(Value::as_array)
        .map(Vec::len);
    let cpp_entry_path = dir.join(format!("{CLAUDE_PLUS_PLUS_ENTRY_ID}.json"));
    let ccs_entry_path = dir.join(format!("{CC_SWITCH_CLAUDE_DESKTOP_ENTRY_ID}.json"));
    let cpp_entry = inspect_gateway_entry(&cpp_entry_path, expected_port);
    let ccs_entry = inspect_gateway_entry(&ccs_entry_path, None);

    json!({
        "path": dir,
        "exists": dir.is_dir(),
        "meta": strip_json_value(meta_status, "json"),
        "appliedId": applied_id,
        "entryCount": entry_count,
        "claudePlusEntry": cpp_entry,
        "ccSwitchEntry": ccs_entry
    })
}

fn inspect_gateway_entry(path: &std::path::Path, expected_port: Option<u64>) -> Value {
    let status = read_json_file_status(path);
    let base_url = status
        .get("json")
        .and_then(|value| value.get("inferenceGatewayBaseUrl"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let api_key = status
        .get("json")
        .and_then(|value| value.get("inferenceGatewayApiKey"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let url = sanitize_url(base_url);
    let port_matches = expected_port.and_then(|port| {
        url.get("port")
            .and_then(Value::as_u64)
            .map(|actual| actual == port)
    });
    let path_ok = url
        .get("path")
        .and_then(Value::as_str)
        .map(|path| path == "/claude-desktop");

    json!({
        "path": path,
        "exists": path.is_file(),
        "readable": status.get("readable").and_then(Value::as_bool).unwrap_or(false),
        "parseOk": status.get("parseOk").and_then(Value::as_bool).unwrap_or(false),
        "baseUrl": url,
        "apiKeyPresent": !api_key.trim().is_empty(),
        "apiKeyLength": api_key.len(),
        "authScheme": status.get("json")
            .and_then(|value| value.get("inferenceGatewayAuthScheme"))
            .and_then(Value::as_str),
        "portMatchesExpected": port_matches,
        "pathIsClaudeDesktop": path_ok,
        "error": status.get("error")
    })
}

fn collect_claude_desktop() -> Value {
    #[cfg(target_os = "windows")]
    {
        let resolved = crate::claude_patch_common::resolve_claude_paths();
        match resolved {
            Ok(paths) => {
                let asar_path = paths.resources.join("app.asar");
                let asar_readable = fs::read(&asar_path)
                    .map_err(|error| error.to_string())
                    .and_then(|data| {
                        crate::claude_patch_common::read_asar_header(&data, &asar_path).map(
                            |header| {
                                json!({
                                    "readable": true,
                                    "headerSize": header.header_size,
                                    "sizeBytes": data.len()
                                })
                            },
                        )
                    });
                json!({
                    "supported": true,
                    "found": true,
                    "running": crate::claude_desktop::is_running(),
                    "appPath": paths.app,
                    "resourcesPath": paths.resources,
                    "files": {
                        "appAsar": result_value(asar_readable),
                        "claudeExe": [
                            file_brief(&paths.app.join("Claude.exe")),
                            file_brief(&paths.app.join("claude.exe"))
                        ],
                        "resourcesDir": dir_brief(&paths.resources)
                    }
                })
            }
            Err(error) => json!({
                "supported": true,
                "found": false,
                "running": crate::claude_desktop::is_running(),
                "error": error
            }),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        json!({
            "supported": false,
            "found": false,
            "running": false
        })
    }
}

fn collect_developer_mode() -> Value {
    let candidates = developer_settings_candidates();
    let inspected = candidates
        .iter()
        .map(|path| {
            let status = read_json_file_status(path);
            let enabled = status
                .get("json")
                .and_then(|value| value.get("allowDevTools"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            json!({
                "path": path,
                "exists": path.is_file(),
                "parseOk": status.get("parseOk").and_then(Value::as_bool).unwrap_or(false),
                "allowDevTools": enabled,
                "error": status.get("error")
            })
        })
        .collect::<Vec<_>>();
    let enabled = inspected.iter().any(|item| {
        item.get("allowDevTools")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });
    json!({
        "enabled": enabled,
        "candidates": inspected
    })
}

fn collect_log_summary(lines: usize) -> Value {
    let path = diagnostic_log_path();
    let text = read_tail(&path, lines).unwrap_or_default();
    let mut error_events = Vec::new();
    let mut event_counts = Map::new();
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let event = value
            .get("event")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let next = event_counts
            .get(&event)
            .and_then(Value::as_u64)
            .unwrap_or(0)
            + 1;
        event_counts.insert(event.clone(), Value::Number(next.into()));
        if event.contains("failed") || event.contains("error") {
            error_events.push(redact_sensitive_value(value));
        }
    }
    if error_events.len() > 20 {
        error_events = error_events.split_off(error_events.len() - 20);
    }
    json!({
        "path": path,
        "exists": path.is_file(),
        "sizeBytes": fs::metadata(&path).map(|meta| meta.len()).ok(),
        "tailLineCount": text.lines().count(),
        "eventCounts": event_counts,
        "recentErrors": error_events
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_findings(
    findings: &mut Vec<Value>,
    status: &Value,
    gateway: &Value,
    ccswitch: &Value,
    config_library: &Value,
    claude_desktop: &Value,
    zh_status: &Value,
    enhance_status: &Value,
    developer_mode: &Value,
) {
    if !ccswitch
        .pointer("/database/exists")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        push_finding(
            findings,
            "error",
            "ccswitch_db_missing",
            "未找到 CC Switch 数据库",
            "Claude++ 无法读取模型映射和 CC Switch 路由状态。",
            "确认 CC Switch 已安装并至少启动过一次。",
            json!({"path": crate::server::default_db_path()}),
        );
    }

    if ccswitch.pointer("/mappings/status").and_then(Value::as_str) == Some("error") {
        push_finding(
            findings,
            "error",
            "ccswitch_mapping_unreadable",
            "无法读取当前 claude-desktop 模型映射",
            "Claude Desktop 模型列表可能为空或无法转发。",
            "在 CC Switch 中为 Claude Desktop 选择服务商，并确认 providers 表有当前记录。",
            json!({"error": ccswitch.pointer("/mappings/error")}),
        );
    } else if ccswitch.pointer("/mappings/count").and_then(Value::as_u64) == Some(0) {
        push_finding(
            findings,
            "error",
            "ccswitch_mapping_empty",
            "当前 CC Switch 服务商没有 Claude Desktop 模型映射",
            "Claude Desktop 无可用模型。",
            "在 CC Switch 当前服务商中配置 claudeDesktopModelRoutes。",
            json!({}),
        );
    }

    if ccswitch
        .pointer("/proxyConfig/status")
        .and_then(Value::as_str)
        == Some("ok")
    {
        let enabled = ccswitch
            .pointer("/proxyConfig/proxyEnabled")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let reachable = ccswitch
            .pointer("/proxyConfig/reachable")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !enabled {
            push_finding(
                findings,
                "warning",
                "ccswitch_route_disabled",
                "CC Switch 路由开关未开启",
                "Claude++ 转发到 CC Switch 后可能无法到达上游代理。",
                "在 CC Switch 中开启代理/路由开关。",
                json!({"proxyConfig": ccswitch.get("proxyConfig")}),
            );
        } else if !reachable {
            push_finding(
                findings,
                "error",
                "ccswitch_route_unreachable",
                "CC Switch 监听地址不可连接",
                "Claude++ 即使启动也无法把请求转发给 CC Switch。",
                "启动 CC Switch 或检查其监听地址和端口配置。",
                json!({"proxyConfig": ccswitch.get("proxyConfig")}),
            );
        }
    }

    if ccswitch
        .pointer("/gatewayProfile/status")
        .and_then(Value::as_str)
        != Some("ok")
    {
        push_finding(
            findings,
            "error",
            "ccswitch_gateway_profile_missing",
            "无法从同一个 CC Switch 配置读取 baseUrl 和 apiKey",
            "Claude++ 无法生成可靠的 Claude Desktop 配置。",
            "在 CC Switch 中重新保存 Claude Desktop 当前服务商，确保 baseUrl 和 key 同时存在。",
            json!({"gatewayProfile": ccswitch.get("gatewayProfile")}),
        );
    }

    let cd_applied = status
        .get("cd_applied")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let gateway_running = gateway
        .get("statusRunning")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let gateway_accepts = gateway
        .get("tcpAcceptsExpectedPort")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if cd_applied && (!gateway_running || !gateway_accepts) {
        push_finding(
            findings,
            "error",
            "claude_plus_route_without_gateway",
            "Claude Desktop 已指向 Claude++，但本地网关不可用",
            "Claude Desktop 无法发现模型或发送请求。",
            "打开 Claude++ 并点击使用 Claude++ 路由，或恢复为 CC Switch 路由。",
            json!({"status": status, "gateway": gateway}),
        );
    }

    if !gateway
        .pointer("/localGatewayToken/validFormat")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        push_finding(
            findings,
            "warning",
            "local_gateway_token_invalid",
            "本地网关 token 缺失或格式无效",
            "页面增强的 Skills、标题翻译或 Token 使用信息可能无法调用本地辅助接口。",
            "重启 Claude++，让它重新生成本地网关 token；之后重新安装相关页面增强。",
            json!({"token": gateway.get("localGatewayToken")}),
        );
    }

    if !claude_desktop
        .get("found")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        push_finding(
            findings,
            "error",
            "claude_desktop_not_found",
            "未找到 Claude Desktop 安装目录",
            "汉化、页面增强、重启 Claude Desktop 和安装检测都无法工作。",
            "安装 Claude Desktop，或在 Claude++ settings.json 中配置 claudeDesktopPath / claudeDesktopResourcesPath。",
            json!({"claudeDesktop": claude_desktop}),
        );
    } else if claude_desktop
        .pointer("/files/appAsar/status")
        .and_then(Value::as_str)
        == Some("error")
    {
        push_finding(
            findings,
            "error",
            "claude_asar_unreadable",
            "Claude Desktop app.asar 不可读或结构异常",
            "页面增强和部分汉化补丁无法安装或验证。",
            "确认 Claude Desktop 安装完整，必要时重新安装 Claude Desktop。",
            json!({"appAsar": claude_desktop.pointer("/files/appAsar")}),
        );
    }

    for candidate in config_library
        .get("candidates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let applied = candidate.get("appliedId").and_then(Value::as_str);
        if applied == Some(crate::constants::CLAUDE_PLUS_PLUS_ENTRY_ID) {
            let entry = candidate.get("claudePlusEntry").unwrap_or(&Value::Null);
            if !entry
                .get("exists")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                push_finding(
                    findings,
                    "error",
                    "config_library_applied_entry_missing",
                    "configLibrary appliedId 指向 Claude++，但 Claude++ entry 文件缺失",
                    "Claude Desktop 当前配置损坏，可能无法加载 3P 网关。",
                    "重新点击使用 Claude++ 路由，或恢复为 CC Switch 路由后再切换。",
                    json!({"candidate": candidate}),
                );
            }
            if entry.get("portMatchesExpected").and_then(Value::as_bool) == Some(false) {
                push_finding(
                    findings,
                    "error",
                    "config_library_port_mismatch",
                    "Claude++ entry 中的端口和当前 Claude++ 网关端口不一致",
                    "Claude Desktop 会连接旧端口，导致模型发现或请求失败。",
                    "重新点击使用 Claude++ 路由以刷新 configLibrary entry。",
                    json!({"entry": entry}),
                );
            }
            if entry.get("apiKeyPresent").and_then(Value::as_bool) == Some(false) {
                push_finding(
                    findings,
                    "error",
                    "config_library_api_key_missing",
                    "Claude++ entry 缺少 inferenceGatewayApiKey",
                    "Claude Desktop 调用本地网关时认证信息不完整。",
                    "重新点击使用 Claude++ 路由，确保 CC Switch 当前配置有 key。",
                    json!({"entry": entry}),
                );
            }
        }
    }

    if enhance_status.get("claude_found").and_then(Value::as_bool) == Some(true) {
        for feature in enhance_status
            .get("features")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if feature.get("enabled").and_then(Value::as_bool) == Some(true)
                && feature.get("available").and_then(Value::as_bool) == Some(false)
            {
                push_finding(
                    findings,
                    "warning",
                    "enhance_feature_unavailable",
                    "存在已启用但当前不可用的页面增强",
                    "对应增强可能不会在 Claude Desktop 中生效。",
                    "重新安装该页面增强，或取消后再启用。",
                    json!({"feature": feature}),
                );
            }
        }
    }

    if zh_status.get("installed").and_then(Value::as_bool) == Some(true)
        && zh_status.get("locale").and_then(Value::as_str) != Some("zh-CN")
    {
        push_finding(
            findings,
            "warning",
            "zh_files_installed_but_locale_not_zh_cn",
            "检测到中文资源文件，但 Claude locale 不是 zh-CN",
            "Claude Desktop 可能仍显示英文或页面增强文案语言不一致。",
            "在一键汉化页重新安装简体中文，或恢复英文后再安装。",
            json!({"claude_zh": zh_status}),
        );
    }

    if !developer_mode
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        push_finding(
            findings,
            "info",
            "developer_mode_disabled",
            "Claude Desktop 开发者模式未开启",
            "这不影响基础路由，但会影响用户排查 Claude Desktop 页面脚本问题。",
            "如需打开 DevTools 排查页面增强，先在欢迎页启用开发者模式。",
            json!({"developerMode": developer_mode}),
        );
    }
}

fn diagnostic_summary(findings: &[Value]) -> Value {
    let error_count = findings
        .iter()
        .filter(|finding| finding.get("severity").and_then(Value::as_str) == Some("error"))
        .count();
    let warning_count = findings
        .iter()
        .filter(|finding| finding.get("severity").and_then(Value::as_str) == Some("warning"))
        .count();
    let top = findings.first().cloned();
    let health = if error_count > 0 {
        "error"
    } else if warning_count > 0 {
        "warning"
    } else {
        "ok"
    };
    json!({
        "health": health,
        "errorCount": error_count,
        "warningCount": warning_count,
        "findingCount": findings.len(),
        "topFinding": top
    })
}

fn push_finding(
    findings: &mut Vec<Value>,
    severity: &str,
    code: &str,
    title: &str,
    impact: &str,
    fix_hint: &str,
    evidence: Value,
) {
    findings.push(json!({
        "severity": severity,
        "code": code,
        "title": title,
        "impact": impact,
        "fixHint": fix_hint,
        "evidence": evidence
    }));
}

fn result_value<T: Serialize>(result: Result<T, String>) -> Value {
    match result {
        Ok(value) => {
            let value = serde_json::to_value(value).unwrap_or_else(|error| {
                json!({
                    "serializationError": error.to_string()
                })
            });
            match value {
                Value::Object(mut object) => {
                    object
                        .entry("status".to_string())
                        .or_insert_with(|| Value::String("ok".to_string()));
                    Value::Object(object)
                }
                other => json!({
                    "status": "ok",
                    "value": other
                }),
            }
        }
        Err(error) => json!({
            "status": "error",
            "error": error
        }),
    }
}

fn result_string(result: Result<String, String>) -> Value {
    match result {
        Ok(value) => json!({
            "status": "ok",
            "value": value
        }),
        Err(error) => json!({
            "status": "error",
            "error": error
        }),
    }
}

fn read_json_file_status(path: &Path) -> Value {
    let meta = fs::metadata(path).ok();
    let exists = meta.as_ref().is_some_and(|meta| meta.is_file());
    let size_bytes = meta.as_ref().map(|meta| meta.len());
    let modified_ms = meta
        .as_ref()
        .and_then(|meta| meta.modified().ok())
        .and_then(system_time_ms);

    if !exists {
        return json!({
            "path": path,
            "exists": false,
            "readable": false,
            "parseOk": false,
            "sizeBytes": size_bytes,
            "modifiedMs": modified_ms
        });
    }

    match fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value) => json!({
                "path": path,
                "exists": true,
                "readable": true,
                "parseOk": true,
                "sizeBytes": size_bytes,
                "modifiedMs": modified_ms,
                "json": value
            }),
            Err(error) => json!({
                "path": path,
                "exists": true,
                "readable": true,
                "parseOk": false,
                "sizeBytes": size_bytes,
                "modifiedMs": modified_ms,
                "error": error.to_string()
            }),
        },
        Err(error) => json!({
            "path": path,
            "exists": true,
            "readable": false,
            "parseOk": false,
            "sizeBytes": size_bytes,
            "modifiedMs": modified_ms,
            "error": error.to_string()
        }),
    }
}

fn strip_json_value(mut value: Value, key: &str) -> Value {
    if let Value::Object(object) = &mut value {
        object.remove(key);
    }
    value
}

fn redact_sensitive_value(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| {
                    if is_sensitive_key(&key) {
                        (key, redacted_value_summary(value))
                    } else {
                        (key, redact_sensitive_value(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(redact_sensitive_value).collect())
        }
        Value::String(text) => Value::String(redact_sensitive_text(&text)),
        other => other,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("key")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("authorization")
}

fn redacted_value_summary(value: Value) -> Value {
    match value {
        Value::String(text) => json!({
            "redacted": true,
            "present": !text.trim().is_empty(),
            "length": text.len()
        }),
        Value::Null => json!({
            "redacted": true,
            "present": false
        }),
        other => json!({
            "redacted": true,
            "present": true,
            "type": value_type_name(&other)
        }),
    }
}

fn redact_sensitive_text(text: &str) -> String {
    let bearer_pattern = regex::Regex::new(r"(?i)Bearer\s+[A-Za-z0-9._~+/=-]+")
        .expect("bearer redaction regex is valid");
    let redacted = bearer_pattern.replace_all(text, "Bearer [redacted]");
    redact_url_queries(&redacted)
}

fn redact_url_queries(text: &str) -> String {
    let url_pattern =
        regex::Regex::new(r#"https?://[^\s"'<>()]+"#).expect("URL redaction regex is valid");
    url_pattern
        .replace_all(text, |captures: &regex::Captures<'_>| {
            let raw = captures.get(0).map(|match_| match_.as_str()).unwrap_or("");
            let Ok(mut url) = reqwest::Url::parse(raw) else {
                return raw.to_string();
            };
            if url.query().is_some() || url.fragment().is_some() {
                url.set_query(None);
                url.set_fragment(None);
                url.to_string()
            } else {
                raw.to_string()
            }
        })
        .into_owned()
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn file_brief(path: &Path) -> Value {
    match fs::metadata(path) {
        Ok(meta) => json!({
            "path": path,
            "exists": meta.is_file(),
            "isFile": meta.is_file(),
            "sizeBytes": meta.len(),
            "modifiedMs": meta.modified().ok().and_then(system_time_ms)
        }),
        Err(error) => json!({
            "path": path,
            "exists": false,
            "isFile": false,
            "error": error.to_string()
        }),
    }
}

fn dir_brief(path: &Path) -> Value {
    match fs::metadata(path) {
        Ok(meta) => json!({
            "path": path,
            "exists": meta.is_dir(),
            "isDir": meta.is_dir(),
            "modifiedMs": meta.modified().ok().and_then(system_time_ms)
        }),
        Err(error) => json!({
            "path": path,
            "exists": false,
            "isDir": false,
            "error": error.to_string()
        }),
    }
}

fn system_time_ms(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
}

fn parse_port(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok().filter(|port| *port > 0)
}

fn tcp_accepts(host: &str, port: u16) -> bool {
    let host = host.trim();
    if host.is_empty() || port == 0 {
        return false;
    }

    let connect_host = match host {
        "0.0.0.0" | "::" | "[::]" => "127.0.0.1",
        other => other,
    };
    let connect_host = connect_host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(connect_host);
    let addrs: Vec<SocketAddr> = match (connect_host, port).to_socket_addrs() {
        Ok(addrs) => addrs.collect(),
        Err(_) => return false,
    };
    addrs
        .iter()
        .take(4)
        .any(|addr| TcpStream::connect_timeout(addr, Duration::from_millis(250)).is_ok())
}

fn valid_local_gateway_token(token: &str) -> bool {
    token.len() == 64 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sanitize_url(raw: &str) -> Value {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return json!({
            "rawPresent": false,
            "parseOk": false
        });
    }

    match reqwest::Url::parse(trimmed) {
        Ok(url) => json!({
            "rawPresent": true,
            "parseOk": true,
            "scheme": url.scheme(),
            "host": url.host_str(),
            "port": url.port_or_known_default(),
            "path": url.path()
        }),
        Err(error) => json!({
            "rawPresent": true,
            "parseOk": false,
            "error": error.to_string()
        }),
    }
}

fn developer_settings_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA").map(PathBuf::from) {
        candidates.push(appdata.join("Claude").join("developer_settings.json"));
        candidates.push(appdata.join("Claude-3p").join("developer_settings.json"));
    }
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
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

fn diagnostic_log_path() -> PathBuf {
    crate::paths::app_state_dir().join(DIAGNOSTIC_LOG_FILE)
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

    #[test]
    fn sanitize_url_does_not_expose_query_or_secret() {
        let value = sanitize_url("http://127.0.0.1:15722/claude-desktop?key=secret-token");

        assert_eq!(value.get("parseOk").and_then(Value::as_bool), Some(true));
        assert_eq!(value.get("host").and_then(Value::as_str), Some("127.0.0.1"));
        assert_eq!(value.get("port").and_then(Value::as_u64), Some(15722));
        assert_eq!(
            value.get("path").and_then(Value::as_str),
            Some("/claude-desktop")
        );
        let rendered = serde_json::to_string(&value).unwrap();
        assert!(!rendered.contains("secret-token"));
        assert!(!rendered.contains("key="));
    }

    #[test]
    fn local_gateway_token_requires_64_hex_chars() {
        assert!(valid_local_gateway_token(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        ));
        assert!(!valid_local_gateway_token("0123456789abcdef"));
        assert!(!valid_local_gateway_token(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeg"
        ));
    }

    #[test]
    fn diagnostic_summary_counts_error_and_warning_findings() {
        let findings = vec![
            json!({"severity": "warning", "code": "first"}),
            json!({"severity": "error", "code": "second"}),
            json!({"severity": "info", "code": "third"}),
        ];

        let summary = diagnostic_summary(&findings);

        assert_eq!(summary.get("health").and_then(Value::as_str), Some("error"));
        assert_eq!(summary.get("errorCount").and_then(Value::as_u64), Some(1));
        assert_eq!(summary.get("warningCount").and_then(Value::as_u64), Some(1));
        assert_eq!(
            summary.pointer("/topFinding/code").and_then(Value::as_str),
            Some("first")
        );
    }

    #[test]
    fn read_json_file_status_reports_invalid_json_without_content() {
        let path = std::env::temp_dir().join(format!(
            "claude-plus-plus-invalid-json-{}.json",
            std::process::id()
        ));
        fs::write(&path, r#"{"apiKey":"secret""#).unwrap();

        let status = read_json_file_status(&path);

        assert_eq!(status.get("exists").and_then(Value::as_bool), Some(true));
        assert_eq!(status.get("readable").and_then(Value::as_bool), Some(true));
        assert_eq!(status.get("parseOk").and_then(Value::as_bool), Some(false));
        assert!(status.get("json").is_none());
        assert!(!serde_json::to_string(&status).unwrap().contains("secret"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn redact_sensitive_value_keeps_shape_without_secret_text() {
        let value = redact_sensitive_value(json!({
            "detail": {
                "apiKey": "sk-test-secret",
                "nested": {
                    "authorization": "Bearer token-value",
                    "baseUrl": "http://127.0.0.1:15722/claude-desktop",
                    "message": "upstream failed with Bearer inline-secret at http://127.0.0.1:15722/v1?api_key=query-secret"
                }
            }
        }));

        let rendered = serde_json::to_string(&value).unwrap();
        assert!(!rendered.contains("sk-test-secret"));
        assert!(!rendered.contains("token-value"));
        assert!(!rendered.contains("inline-secret"));
        assert!(!rendered.contains("query-secret"));
        assert!(!rendered.contains("api_key="));
        assert_eq!(
            value
                .pointer("/detail/apiKey/redacted")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            value
                .pointer("/detail/apiKey/length")
                .and_then(Value::as_u64),
            Some(14)
        );
        assert_eq!(
            value
                .pointer("/detail/nested/baseUrl")
                .and_then(Value::as_str),
            Some("http://127.0.0.1:15722/claude-desktop")
        );
        assert_eq!(
            value
                .pointer("/detail/nested/message")
                .and_then(Value::as_str),
            Some("upstream failed with Bearer [redacted] at http://127.0.0.1:15722/v1")
        );
    }
}
