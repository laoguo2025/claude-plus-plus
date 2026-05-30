// 代理服务生命周期管理 + 从 CC Switch 读取 bearer key。
use crate::ccswitch_db;
use crate::proxy::{self, AppState};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::async_runtime::JoinHandle;
use tokio::sync::oneshot;

pub struct ProxyServer {
    pub port: u16,
    shutdown: Option<oneshot::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

#[derive(Default)]
pub struct ServerHandle(pub Arc<Mutex<Option<ProxyServer>>>);

impl ServerHandle {
    pub fn is_running(&self) -> bool {
        self.0.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    pub fn port(&self) -> Option<u16> {
        self.0.lock().ok().and_then(|g| g.as_ref().map(|s| s.port))
    }

    /// 启动代理。返回 Err 表示启动失败(如端口占用)。
    pub fn start(&self, port: u16, db_path: PathBuf) -> Result<(), String> {
        let mut guard = self.0.lock().map_err(|_| "lock poisoned".to_string())?;
        if guard.is_some() {
            return Ok(()); // 已在运行
        }

        let state = AppState::new(db_path);
        let app = proxy::router(state);
        let (tx, rx) = oneshot::channel::<()>();

        let addr = format!("127.0.0.1:{port}");
        let handle = tauri::async_runtime::spawn(async move {
            let listener = match tokio::net::TcpListener::bind(&addr).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("bind {addr} failed: {e}");
                    return;
                }
            };
            tracing::info!("ccs2claude proxy listening on {addr}");
            let server = axum::serve(listener, app).with_graceful_shutdown(async {
                let _ = rx.await;
            });
            if let Err(e) = server.await {
                tracing::error!("server error: {e}");
            }
        });

        *guard = Some(ProxyServer {
            port,
            shutdown: Some(tx),
            handle: Some(handle),
        });
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut guard = self.0.lock().map_err(|_| "lock poisoned".to_string())?;
        if let Some(mut srv) = guard.take() {
            if let Some(tx) = srv.shutdown.take() {
                let _ = tx.send(());
            }
            if let Some(h) = srv.handle.take() {
                h.abort();
            }
        }
        Ok(())
    }
}

/// 从 CC Switch 当前生效的 Claude Desktop profile 文件读 bearer key。
/// 优先读运行实例配置库里 CC Switch 写的 157210 条目。
pub fn read_ccswitch_api_key() -> Option<String> {
    let dir = crate::cd_config::resolve_config_library_dir().ok()?;
    // CC Switch 固定写这个条目
    let candidates = [
        dir.join("00000000-0000-4000-8000-000000157210.json"),
    ];
    for p in candidates {
        if let Ok(s) = std::fs::read_to_string(&p) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(k) = v.get("inferenceGatewayApiKey").and_then(|x| x.as_str()) {
                    if !k.is_empty() {
                        return Some(k.to_string());
                    }
                }
            }
        }
    }
    None
}

/// 默认 DB 路径透出,供命令层使用。
pub fn default_db_path() -> PathBuf {
    ccswitch_db::default_db_path()
}
