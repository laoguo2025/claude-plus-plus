mod ccswitch_db;
mod cd_config;
mod claude_desktop;
mod claude_enhance;
mod claude_patch_common;
mod claude_skills;
mod claude_zh;
mod constants;
mod diagnostics;
mod proxy;
mod server;
mod settings;
mod time_utils;
mod welcome;

use constants::CC_SWITCH_CLAUDE_DESKTOP_ENTRY_ID;
use server::ServerHandle;
use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

#[derive(serde::Serialize)]
struct StatusInfo {
    running: bool,
    port: Option<u16>,
    cd_applied: bool,
    ccswitch_route: CcSwitchRouteStatus,
}

#[derive(serde::Serialize)]
struct CcSwitchRouteStatus {
    enabled: bool,
    configured: Option<bool>,
    has_mappings: bool,
    reachable: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticsRequest {
    restart_needed: Option<bool>,
}

#[tauri::command]
fn proxy_status(state: tauri::State<ServerHandle>) -> StatusInfo {
    ensure_proxy_if_applied(state.inner());
    let port = settings::proxy_port();
    StatusInfo {
        running: state.is_healthy_on(port),
        port: Some(port),
        cd_applied: cd_config::is_applied(),
        ccswitch_route: ccswitch_route_status(),
    }
}

fn ccswitch_route_status() -> CcSwitchRouteStatus {
    let proxy_config = ccswitch_db::load_proxy_config(&server::default_db_path()).ok();
    let configured = proxy_config.as_ref().map(|config| config.proxy_enabled);
    let reachable = proxy_config
        .as_ref()
        .map(|config| config.proxy_enabled && tcp_endpoint_accepts(&config.listen_address, config.listen_port))
        .unwrap_or(false);
    let has_mappings = ccswitch_db::load_mappings(&server::default_db_path()).is_ok();

    CcSwitchRouteStatus {
        enabled: configured.unwrap_or(false),
        configured,
        has_mappings,
        reachable,
    }
}

fn tcp_endpoint_accepts(host: &str, port: u16) -> bool {
    let Ok(addrs) = (host, port).to_socket_addrs() else {
        return false;
    };
    addrs
        .into_iter()
        .any(|addr| std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok())
}

#[tauri::command]
fn use_claude_plus_route(state: tauri::State<ServerHandle>) -> Result<(), String> {
    let _ = diagnostics::append_event("manager.use_claude_plus_route.start", serde_json::json!({}));
    let was_running = state.is_healthy();
    state.ensure_running(settings::proxy_port(), server::default_db_path())?;
    let result = apply_cd_config();
    if result.is_err() && !was_running {
        if let Err(error) = state.stop() {
            let _ = diagnostics::append_event(
                "manager.use_claude_plus_route.stop_proxy_failed",
                serde_json::json!({
                    "error": error
                }),
            );
        }
    }
    let _ = diagnostics::append_event(
        if result.is_ok() {
            "manager.use_claude_plus_route.ok"
        } else {
            "manager.use_claude_plus_route.failed"
        },
        serde_json::json!({
            "error": result.as_ref().err()
        }),
    );
    result
}

#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
fn use_ccs_route(state: tauri::State<ServerHandle>) -> Result<(), String> {
    let result = revert_cd_config();
    if result.is_ok() {
        if let Err(error) = state.stop() {
            let _ = diagnostics::append_event(
                "manager.use_ccs_route.stop_proxy_failed",
                serde_json::json!({
                    "error": error
                }),
            );
        }
    }
    let _ = diagnostics::append_event(
        if result.is_ok() {
            "manager.use_ccs_route.ok"
        } else {
            "manager.use_ccs_route.failed"
        },
        serde_json::json!({
            "error": result.as_ref().err()
        }),
    );
    result
}

#[tauri::command]
fn get_mappings() -> Result<ccswitch_db::ProviderMappings, String> {
    ccswitch_db::load_mappings(&server::default_db_path())
}

#[tauri::command]
fn apply_cd_config() -> Result<(), String> {
    let key = server::read_ccswitch_api_key()
        .ok_or_else(|| "无法从 CC Switch 配置读取 API key".to_string())?;
    cd_config::apply(settings::proxy_port(), &key)
}

#[tauri::command]
fn revert_cd_config() -> Result<(), String> {
    cd_config::revert(Some(CC_SWITCH_CLAUDE_DESKTOP_ENTRY_ID))
}

fn ensure_proxy_if_applied(state: &ServerHandle) {
    let port = settings::proxy_port();
    if !cd_config::is_applied() || state.is_healthy_on(port) {
        return;
    }
    if let Err(error) = state.ensure_running(port, server::default_db_path()) {
        let _ = diagnostics::append_event(
            "manager.proxy_health_restart_failed",
            serde_json::json!({
                "error": error
            }),
        );
    } else {
        let _ = diagnostics::append_event(
            "manager.proxy_health_restart_ok",
            serde_json::json!({
                "port": port
            }),
        );
    }
}

#[tauri::command]
fn restart_claude_desktop() -> Result<(), String> {
    let result = claude_desktop::restart();
    let _ = diagnostics::append_event(
        if result.is_ok() {
            "manager.restart_claude_desktop.ok"
        } else {
            "manager.restart_claude_desktop.failed"
        },
        serde_json::json!({
            "error": result.as_ref().err()
        }),
    );
    result
}

#[tauri::command]
fn claude_zh_status() -> claude_zh::ClaudeZhStatus {
    claude_zh::status()
}

#[tauri::command]
fn welcome_status() -> welcome::WelcomeStatus {
    welcome::status()
}

#[tauri::command]
fn install_claude_zh(language: String, skip_asar_patch: bool) -> Result<(), String> {
    claude_zh::install(&language, skip_asar_patch)
}

#[tauri::command]
fn backup_claude_zh() -> Result<(), String> {
    claude_zh::backup()
}

#[tauri::command]
fn uninstall_claude_zh() -> Result<(), String> {
    claude_zh::uninstall()
}

#[tauri::command]
fn claude_enhance_status() -> claude_enhance::ClaudeEnhanceStatus {
    claude_enhance::status()
}

#[tauri::command]
fn install_claude_enhance(feature: String) -> Result<(), String> {
    claude_enhance::install(&feature)
}

#[tauri::command]
fn uninstall_claude_enhance(feature: String) -> Result<(), String> {
    claude_enhance::uninstall(&feature)
}

#[tauri::command]
fn list_claude_skills() -> claude_skills::ClaudeSkillsResponse {
    claude_skills::list_skills()
}

#[tauri::command]
fn trash_claude_skill(id: String) -> Result<(), String> {
    claude_skills::trash_skill(&id)
}

#[tauri::command]
fn read_latest_logs(lines: Option<usize>) -> diagnostics::LogsPayload {
    diagnostics::read_latest_logs(lines.unwrap_or(200))
}

#[tauri::command]
fn generate_diagnostics(
    state: tauri::State<ServerHandle>,
    request: Option<DiagnosticsRequest>,
) -> diagnostics::DiagnosticsPayload {
    let status = proxy_status(state);
    let mappings = get_mappings()
        .and_then(|mappings| serde_json::to_value(mappings).map_err(|e| e.to_string()));
    let payload = diagnostics::report(
        serde_json::json!({
            "running": status.running,
            "port": status.port,
            "cd_applied": status.cd_applied,
            "ccswitch_route": status.ccswitch_route,
            "restart_needed": request.and_then(|r| r.restart_needed).unwrap_or(false)
        }),
        mappings,
        serde_json::to_value(claude_zh_status()).unwrap_or_else(|e| {
            serde_json::json!({
                "serialization_error": e.to_string()
            })
        }),
        serde_json::to_value(claude_enhance_status()).unwrap_or_else(|e| {
            serde_json::json!({
                "serialization_error": e.to_string()
            })
        }),
    );
    let _ = diagnostics::append_event("manager.generate_diagnostics", serde_json::json!({}));
    payload
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ServerHandle::default())
        .setup(|app| {
            // 启动即自动开代理
            let handle = app.state::<ServerHandle>().inner().clone();
            let port = settings::proxy_port();
            if let Err(e) = handle.ensure_running(port, server::default_db_path()) {
                tracing::error!("auto start proxy failed: {e}");
            }
            spawn_mapping_monitor(handle);
            let show = MenuItem::with_id(app, "show", "Show Claude++", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let tray = TrayIconBuilder::new()
                .tooltip("Claude++")
                .menu(&menu)
                .show_menu_on_left_click(false);
            let tray = if let Some(icon) = app.default_window_icon().cloned() {
                tray.icon(icon)
            } else {
                tray
            };
            tray.on_menu_event(|app, event| match event.id().as_ref() {
                "show" => show_main_window(app),
                "quit" => app.exit(0),
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if matches!(
                    event,
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    }
                ) {
                    show_main_window(tray.app_handle());
                }
            })
            .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(e) = window.hide() {
                    tracing::error!("hide window failed: {e}");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            proxy_status,
            use_claude_plus_route,
            app_version,
            use_ccs_route,
            get_mappings,
            apply_cd_config,
            revert_cd_config,
            restart_claude_desktop,
            claude_zh_status,
            welcome_status,
            install_claude_zh,
            backup_claude_zh,
            uninstall_claude_zh,
            claude_enhance_status,
            install_claude_enhance,
            uninstall_claude_enhance,
            list_claude_skills,
            trash_claude_skill,
            read_latest_logs,
            generate_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_main_window<R: tauri::Runtime>(manager: &impl Manager<R>) {
    if let Some(window) = manager.get_webview_window("main") {
        if let Err(e) = window.show() {
            tracing::error!("show window failed: {e}");
        }
        if let Err(e) = window.unminimize() {
            tracing::error!("unminimize window failed: {e}");
        }
        if let Err(e) = window.set_focus() {
            tracing::error!("focus window failed: {e}");
        }
    }
}

fn spawn_mapping_monitor(handle: ServerHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last_fingerprint: Option<String> = None;
        let mut last_refreshed_fingerprint: Option<String> = None;
        let mut pending_change_since: Option<Instant> = None;
        let mut pending_fingerprint: Option<String> = None;
        let mut refreshed_on_startup = false;

        loop {
            ensure_proxy_if_applied(&handle);
            match ccswitch_db::load_mappings(&server::default_db_path()) {
                Ok(pm) => {
                    let fingerprint = mapping_monitor_fingerprint(&pm);
                    let changed = last_fingerprint
                        .as_ref()
                        .map(|last| last != &fingerprint)
                        .unwrap_or(false);

                    if changed {
                        if pending_fingerprint.as_ref() != Some(&fingerprint) {
                            pending_change_since = Some(Instant::now());
                        }
                        pending_fingerprint = Some(fingerprint.clone());
                    }

                    let change_ready = pending_change_since
                        .map(|since| since.elapsed() >= Duration::from_secs(2))
                        .unwrap_or(false);
                    let pending_ready =
                        change_ready && pending_fingerprint.as_ref() == Some(&fingerprint);
                    let should_refresh = !refreshed_on_startup || pending_ready;
                    let already_refreshed =
                        last_refreshed_fingerprint.as_ref() == Some(&fingerprint);

                    if should_refresh && !already_refreshed && cd_config::is_applied() {
                        let port = settings::proxy_port();
                        let reason = if pending_ready {
                            "routing config changed"
                        } else {
                            "startup"
                        };
                        if refresh_cd_config(port, reason) {
                            last_refreshed_fingerprint = Some(fingerprint.clone());
                            pending_change_since = None;
                            pending_fingerprint = None;
                            refreshed_on_startup = true;
                        }
                    }

                    last_fingerprint = Some(fingerprint);
                }
                Err(e) => {
                    tracing::warn!("mapping monitor failed to load mappings: {e}");
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

fn mapping_monitor_fingerprint(pm: &ccswitch_db::ProviderMappings) -> String {
    serde_json::json!({
        "mappings": pm,
        "api_key": server::read_ccswitch_api_key(),
        "upstream": server::read_ccswitch_base_url(),
    })
    .to_string()
}

fn refresh_cd_config(port: u16, reason: &str) -> bool {
    let Some(key) = server::read_ccswitch_api_key() else {
        tracing::warn!("skip Claude Desktop config entry refresh ({reason}): API key not found");
        let _ = diagnostics::append_event(
            "manager.mapping_refresh_skipped",
            serde_json::json!({
                "reason": reason,
                "error": "api key not found"
            }),
        );
        return false;
    };

    if let Err(e) = cd_config::apply(port, &key) {
        tracing::warn!("Claude Desktop config entry refresh failed ({reason}): {e}");
        let _ = diagnostics::append_event(
            "manager.mapping_refresh_failed",
            serde_json::json!({
                "reason": reason,
                "port": port,
                "error": e
            }),
        );
        false
    } else {
        tracing::info!(
            "Claude Desktop config entry refreshed ({reason}); restart Claude Desktop to refresh model picker"
        );
        let _ = diagnostics::append_event(
            "manager.mapping_refresh_ok",
            serde_json::json!({
                "reason": reason,
                "port": port
            }),
        );
        true
    }
}
