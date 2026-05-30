mod ccswitch_db;
mod cd_config;
mod proxy;
mod server;

use server::ServerHandle;
use tauri::Manager;

const DEFAULT_PORT: u16 = 15722;

#[derive(serde::Serialize)]
struct StatusInfo {
    running: bool,
    port: Option<u16>,
    cd_applied: bool,
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
fn start_proxy(state: tauri::State<ServerHandle>) -> Result<(), String> {
    state.start(DEFAULT_PORT, server::default_db_path())
}

#[tauri::command]
fn stop_proxy(state: tauri::State<ServerHandle>) -> Result<(), String> {
    state.stop()
}

#[tauri::command]
fn get_mappings() -> Result<ccswitch_db::ProviderMappings, String> {
    ccswitch_db::load_mappings(&server::default_db_path())
}

#[tauri::command]
fn apply_cd_config() -> Result<(), String> {
    let key = server::read_ccswitch_api_key()
        .ok_or_else(|| "无法从 CC Switch 配置读取 API key".to_string())?;
    cd_config::apply(DEFAULT_PORT, &key)
}

#[tauri::command]
fn revert_cd_config() -> Result<(), String> {
    cd_config::revert(Some("00000000-0000-4000-8000-000000157210"))
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
            if let Err(e) = handle.start(DEFAULT_PORT, server::default_db_path()) {
                tracing::error!("auto start proxy failed: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            proxy_status,
            start_proxy,
            stop_proxy,
            get_mappings,
            apply_cd_config,
            revert_cd_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
