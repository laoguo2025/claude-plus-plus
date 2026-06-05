mod ccswitch_db;
mod cd_config;
mod claude_desktop;
mod claude_enhance;
mod claude_patch_common;
mod claude_skills;
mod claude_zh;
mod constants;
mod developer_settings;
mod diagnostics;
mod net_utils;
mod paths;
mod proxy;
mod server;
mod settings;
mod time_utils;
mod welcome;

use constants::CC_SWITCH_CLAUDE_DESKTOP_ENTRY_ID;
use server::ServerHandle;
use std::time::{Duration, Instant};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

const MAPPING_REFRESH_DEBOUNCE: Duration = Duration::from_secs(2);
const MAPPING_MONITOR_POLL_INTERVAL: Duration = Duration::from_secs(2);

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| format!("后台任务执行失败: {error}"))
}

#[cfg(windows)]
fn set_windows_taskbar_icon(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    use std::ptr::null;
    use windows_sys::{
        core::PCWSTR,
        Win32::{
            Foundation::{HINSTANCE, HWND, LPARAM, WPARAM},
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{
                LoadImageW, SendMessageW, SetClassLongPtrW, GCLP_HICON, GCLP_HICONSM, ICON_BIG,
                ICON_SMALL, IMAGE_ICON, LR_DEFAULTSIZE, LR_SHARED, WM_SETICON,
            },
        },
    };

    const APP_ICON_RESOURCE_ID: u16 = 32512;

    fn resource_id(id: u16) -> PCWSTR {
        id as usize as PCWSTR
    }

    let hwnd = window.hwnd()?.0 as HWND;
    let hinstance = unsafe { GetModuleHandleW(null()) } as HINSTANCE;
    let icon = unsafe {
        LoadImageW(
            hinstance,
            resource_id(APP_ICON_RESOURCE_ID),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        )
    };

    if icon.is_null() {
        tracing::error!("load embedded window icon failed");
        return Ok(());
    }

    unsafe {
        SendMessageW(hwnd, WM_SETICON, ICON_SMALL as WPARAM, icon as LPARAM);
        SendMessageW(hwnd, WM_SETICON, ICON_BIG as WPARAM, icon as LPARAM);
        SetClassLongPtrW(hwnd, GCLP_HICONSM, icon as isize);
        SetClassLongPtrW(hwnd, GCLP_HICON, icon as isize);
    }

    Ok(())
}

#[cfg(not(windows))]
fn set_windows_taskbar_icon(_window: &tauri::WebviewWindow) -> tauri::Result<()> {
    Ok(())
}

#[derive(serde::Serialize)]
struct StatusInfo {
    running: bool,
    port: Option<u16>,
    cd_applied: bool,
    ccswitch_route: CcSwitchRouteStatus,
}

#[derive(serde::Serialize)]
struct CcSwitchRouteStatus {
    claude_route_enabled: bool,
    proxy_enabled: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatusMode {
    Manager,
    Diagnostics,
}

fn proxy_status_blocking(state: &ServerHandle) -> StatusInfo {
    proxy_status_for_mode(state, StatusMode::Manager)
}

fn proxy_status_for_mode(state: &ServerHandle, mode: StatusMode) -> StatusInfo {
    if status_mode_restores_proxy(mode) {
        ensure_proxy_if_applied(state);
    }
    let port = settings::proxy_port();
    StatusInfo {
        running: state.is_healthy_on(port),
        port: Some(port),
        cd_applied: cd_config::is_applied(),
        ccswitch_route: ccswitch_route_status(),
    }
}

fn status_mode_restores_proxy(mode: StatusMode) -> bool {
    matches!(mode, StatusMode::Manager)
}

#[tauri::command]
async fn proxy_status(state: tauri::State<'_, ServerHandle>) -> Result<StatusInfo, String> {
    let state = state.inner().clone();
    run_blocking(move || proxy_status_blocking(&state)).await
}

fn ccswitch_route_status() -> CcSwitchRouteStatus {
    let proxy_config = ccswitch_db::load_proxy_config(&server::default_db_path()).ok();
    let configured = proxy_config.as_ref().map(|config| config.proxy_enabled);
    let reachable = proxy_config
        .as_ref()
        .map(|config| {
            config.proxy_enabled
                && net_utils::tcp_endpoint_accepts(&config.listen_address, config.listen_port)
        })
        .unwrap_or(false);
    let has_mappings = ccswitch_db::load_mappings(&server::default_db_path()).is_ok();
    let proxy_enabled = proxy_config
        .as_ref()
        .map(|config| config.proxy_enabled)
        .unwrap_or(false);
    let claude_route_enabled = proxy_config
        .as_ref()
        .map(|config| config.claude_route_enabled)
        .unwrap_or(false);

    CcSwitchRouteStatus {
        claude_route_enabled,
        proxy_enabled,
        enabled: proxy_enabled,
        configured,
        has_mappings,
        reachable,
    }
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

fn get_mappings_blocking() -> Result<ccswitch_db::ProviderMappings, String> {
    ccswitch_db::load_mappings(&server::default_db_path())
}

#[tauri::command]
async fn get_mappings() -> Result<ccswitch_db::ProviderMappings, String> {
    run_blocking(get_mappings_blocking).await?
}

#[tauri::command]
fn apply_cd_config() -> Result<(), String> {
    let profile = server::read_ccswitch_gateway_profile()
        .ok_or_else(|| "无法从 CC Switch 配置读取 API key 和上游地址".to_string())?;
    cd_config::apply(settings::proxy_port(), &profile.api_key)
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

fn claude_zh_status_blocking() -> claude_zh::ClaudeZhStatus {
    claude_zh::status()
}

#[tauri::command]
async fn claude_zh_status() -> Result<claude_zh::ClaudeZhStatus, String> {
    run_blocking(claude_zh_status_blocking).await
}

fn welcome_status_blocking() -> welcome::WelcomeStatus {
    welcome::status()
}

#[tauri::command]
async fn welcome_status() -> Result<welcome::WelcomeStatus, String> {
    run_blocking(welcome_status_blocking).await
}

#[tauri::command]
fn enable_claude_developer_mode() -> Result<(), String> {
    let was_running = claude_desktop::is_running();
    let result = welcome::enable_developer_mode().and_then(|_| {
        if was_running {
            claude_desktop::restart()
        } else {
            Ok(())
        }
    });
    let _ = diagnostics::append_event(
        if result.is_ok() {
            "manager.enable_claude_developer_mode.ok"
        } else {
            "manager.enable_claude_developer_mode.failed"
        },
        serde_json::json!({
            "claude_was_running": was_running,
            "error": result.as_ref().err()
        }),
    );
    result
}

#[tauri::command]
fn install_claude_code() -> Result<(), String> {
    let result = welcome::install_claude_code();
    let _ = diagnostics::append_event(
        if result.is_ok() {
            "manager.install_claude_code.ok"
        } else {
            "manager.install_claude_code.failed"
        },
        serde_json::json!({
            "error": result.as_ref().err()
        }),
    );
    result
}

#[tauri::command]
fn enable_virtual_machine_platform() -> Result<(), String> {
    let result = welcome::enable_virtual_machine_platform();
    let _ = diagnostics::append_event(
        if result.is_ok() {
            "manager.enable_virtual_machine_platform.ok"
        } else {
            "manager.enable_virtual_machine_platform.failed"
        },
        serde_json::json!({
            "error": result.as_ref().err()
        }),
    );
    result
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

fn claude_enhance_status_blocking() -> claude_enhance::ClaudeEnhanceStatus {
    claude_enhance::status()
}

#[tauri::command]
async fn claude_enhance_status() -> Result<claude_enhance::ClaudeEnhanceStatus, String> {
    run_blocking(claude_enhance_status_blocking).await
}

#[tauri::command]
fn install_claude_enhance(
    feature: String,
    state: tauri::State<ServerHandle>,
) -> Result<(), String> {
    if install_enhance_starts_proxy_before_write(&feature) {
        state.ensure_running(settings::proxy_port(), server::default_db_path())?;
    }
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

fn read_latest_logs_blocking(lines: Option<usize>) -> diagnostics::LogsPayload {
    diagnostics::read_latest_logs(lines.unwrap_or(200))
}

#[tauri::command]
async fn read_latest_logs(lines: Option<usize>) -> Result<diagnostics::LogsPayload, String> {
    run_blocking(move || read_latest_logs_blocking(lines)).await
}

#[tauri::command]
async fn generate_diagnostics(
    state: tauri::State<'_, ServerHandle>,
    request: Option<DiagnosticsRequest>,
) -> Result<diagnostics::DiagnosticsPayload, String> {
    let state = state.inner().clone();
    run_blocking(move || generate_diagnostics_blocking(&state, request)).await
}

fn generate_diagnostics_blocking(
    state: &ServerHandle,
    request: Option<DiagnosticsRequest>,
) -> diagnostics::DiagnosticsPayload {
    let status = proxy_status_for_mode(state, StatusMode::Diagnostics);
    let mappings = get_mappings_blocking()
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
        serde_json::to_value(claude_zh_status_blocking()).unwrap_or_else(|e| {
            serde_json::json!({
                "serialization_error": e.to_string()
            })
        }),
        serde_json::to_value(claude_enhance_status_blocking()).unwrap_or_else(|e| {
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
            let handle = app.state::<ServerHandle>().inner().clone();
            if should_restore_proxy_on_startup() {
                let port = settings::proxy_port();
                if let Err(e) = handle.ensure_running(port, server::default_db_path()) {
                    tracing::error!("auto start proxy failed: {e}");
                }
            }
            spawn_mapping_monitor(handle);
            if let (Some(window), Some(icon)) = (
                app.get_webview_window("main"),
                app.default_window_icon().cloned(),
            ) {
                if let Err(e) = window.set_icon(icon) {
                    tracing::error!("set main window icon failed: {e}");
                }
                if let Err(e) = set_windows_taskbar_icon(&window) {
                    tracing::error!("set main taskbar icon failed: {e}");
                }
            }
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
            enable_claude_developer_mode,
            install_claude_code,
            enable_virtual_machine_platform,
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

fn should_restore_proxy_on_startup() -> bool {
    should_restore_proxy_for_state(cd_config::is_applied())
}

fn should_restore_proxy_for_state(cd_applied: bool) -> bool {
    cd_applied
}

fn enhance_feature_needs_local_gateway(feature: &str) -> bool {
    matches!(
        feature,
        "plugins" | "conversation_title_i18n" | "token_usage"
    )
}

fn install_enhance_starts_proxy_before_write(feature: &str) -> bool {
    enhance_feature_needs_local_gateway(feature)
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
                        .map(|since| since.elapsed() >= MAPPING_REFRESH_DEBOUNCE)
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

            tokio::time::sleep(MAPPING_MONITOR_POLL_INTERVAL).await;
        }
    });
}

fn mapping_monitor_fingerprint(pm: &ccswitch_db::ProviderMappings) -> String {
    serde_json::json!({
        "mappings": pm,
        "gateway": server::read_ccswitch_gateway_profile(),
    })
    .to_string()
}

fn refresh_cd_config(port: u16, reason: &str) -> bool {
    let Some(profile) = server::read_ccswitch_gateway_profile() else {
        tracing::warn!(
            "skip Claude Desktop config entry refresh ({reason}): gateway profile not found"
        );
        let _ = diagnostics::append_event(
            "manager.mapping_refresh_skipped",
            serde_json::json!({
                "reason": reason,
                "error": "gateway profile not found"
            }),
        );
        return false;
    };

    if let Err(e) = cd_config::apply(port, &profile.api_key) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_proxy_restores_only_when_route_is_applied() {
        assert!(!should_restore_proxy_for_state(false));
        assert!(should_restore_proxy_for_state(true));
    }

    #[test]
    fn gateway_enhance_features_start_proxy_after_install() {
        assert!(enhance_feature_needs_local_gateway(
            "conversation_title_i18n"
        ));
        assert!(enhance_feature_needs_local_gateway("token_usage"));
        assert!(enhance_feature_needs_local_gateway("plugins"));
        assert!(!enhance_feature_needs_local_gateway("timeline"));
    }

    #[test]
    fn diagnostics_status_does_not_request_proxy_restore() {
        assert!(!status_mode_restores_proxy(StatusMode::Diagnostics));
        assert!(status_mode_restores_proxy(StatusMode::Manager));
    }

    #[test]
    fn gateway_enhance_features_start_proxy_before_install() {
        assert!(install_enhance_starts_proxy_before_write("plugins"));
        assert!(install_enhance_starts_proxy_before_write(
            "conversation_title_i18n"
        ));
        assert!(install_enhance_starts_proxy_before_write("token_usage"));
        assert!(!install_enhance_starts_proxy_before_write("timeline"));
    }
}
