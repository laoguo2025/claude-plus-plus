mod ccswitch_db;
mod cd_config;
mod claude_desktop;
mod claude_enhance;
mod claude_skills;
mod claude_zh;
mod constants;
mod diagnostics;
mod proxy;
mod server;
mod time_utils;

use constants::{CC_SWITCH_CLAUDE_DESKTOP_ENTRY_ID, DEFAULT_PROXY_PORT};
use server::ServerHandle;
use std::time::Duration;
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
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticsRequest {
    restart_needed: Option<bool>,
}

#[tauri::command]
fn proxy_status(state: tauri::State<ServerHandle>) -> StatusInfo {
    StatusInfo {
        running: state.is_running(),
        port: state.port(),
        cd_applied: cd_config::is_applied(),
    }
}

#[tauri::command]
fn use_claude_plus_route(state: tauri::State<ServerHandle>) -> Result<(), String> {
    let _ = diagnostics::append_event("manager.use_claude_plus_route.start", serde_json::json!({}));
    state.start(DEFAULT_PROXY_PORT, server::default_db_path())?;
    let result = apply_cd_config();
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
    cd_config::apply(DEFAULT_PROXY_PORT, &key)
}

#[tauri::command]
fn revert_cd_config() -> Result<(), String> {
    cd_config::revert(Some(CC_SWITCH_CLAUDE_DESKTOP_ENTRY_ID))
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
            let handle: tauri::State<ServerHandle> = app.state();
            if let Err(e) = handle.start(DEFAULT_PROXY_PORT, server::default_db_path()) {
                tracing::error!("auto start proxy failed: {e}");
            }
            spawn_mapping_monitor(DEFAULT_PROXY_PORT);
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
            use_ccs_route,
            get_mappings,
            apply_cd_config,
            revert_cd_config,
            restart_claude_desktop,
            claude_zh_status,
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

fn spawn_mapping_monitor(port: u16) {
    tauri::async_runtime::spawn(async move {
        let mut last_fingerprint: Option<String> = None;
        let mut refreshed_on_startup = false;

        loop {
            match ccswitch_db::load_mappings(&server::default_db_path()) {
                Ok(pm) => {
                    let fingerprint = serde_json::to_string(&pm).unwrap_or_default();
                    let changed = last_fingerprint
                        .as_ref()
                        .map(|last| last != &fingerprint)
                        .unwrap_or(false);

                    if (!refreshed_on_startup || changed) && cd_config::is_applied() {
                        let reason = if changed {
                            "mapping changed"
                        } else {
                            "startup"
                        };
                        refresh_cd_config(port, reason);
                        refreshed_on_startup = true;
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

fn refresh_cd_config(port: u16, reason: &str) {
    let Some(key) = server::read_ccswitch_api_key() else {
        tracing::warn!("skip Claude Desktop config entry refresh ({reason}): API key not found");
        return;
    };

    if let Err(e) = cd_config::apply(port, &key) {
        tracing::warn!("Claude Desktop config entry refresh failed ({reason}): {e}");
    } else {
        tracing::info!(
            "Claude Desktop config entry refreshed ({reason}); restart Claude Desktop to refresh model picker"
        );
    }
}
