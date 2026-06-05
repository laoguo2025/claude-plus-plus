// 代理服务生命周期管理 + 从 CC Switch 读取 bearer key。
use crate::ccswitch_db;
use crate::constants::CC_SWITCH_CLAUDE_DESKTOP_ENTRY_ID;
use crate::proxy::{self, AppState};
use serde::Serialize;
use std::fs;
#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::io::Read;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use tauri::async_runtime::JoinHandle;
use tokio::sync::oneshot;

pub struct ProxyServer {
    pub port: u16,
    shutdown: Option<oneshot::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

#[derive(Clone, Default)]
pub struct ServerHandle(pub Arc<Mutex<Option<ProxyServer>>>);

const LOCAL_GATEWAY_TOKEN_FILE: &str = "local-gateway-token";
const LOCAL_GATEWAY_TOKEN_BYTES: usize = 32;
const LOCAL_GATEWAY_TOKEN_HEX_LEN: usize = LOCAL_GATEWAY_TOKEN_BYTES * 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CcSwitchGatewayProfile {
    pub api_key: String,
    pub base_url: String,
}

impl ServerHandle {
    pub fn port(&self) -> Option<u16> {
        self.0.lock().ok().and_then(|g| g.as_ref().map(|s| s.port))
    }

    pub fn is_healthy(&self) -> bool {
        let Some(port) = self.port() else {
            return false;
        };
        crate::net_utils::tcp_local_port_accepts(port)
    }

    pub fn is_healthy_on(&self, port: u16) -> bool {
        self.port() == Some(port) && crate::net_utils::tcp_local_port_accepts(port)
    }

    pub fn ensure_running(&self, port: u16, db_path: PathBuf) -> Result<(), String> {
        if self.is_healthy_on(port) {
            return Ok(());
        }
        self.stop()?;
        self.start(port, db_path)
    }

    /// 启动代理。返回 Err 表示启动失败(如端口占用)。
    pub fn start(&self, port: u16, db_path: PathBuf) -> Result<(), String> {
        let mut guard = self.0.lock().map_err(|_| "lock poisoned".to_string())?;
        if guard.is_some() {
            return Ok(()); // 已在运行
        }

        let state = AppState::new(db_path, ensure_local_gateway_token()?);
        let app = proxy::router(state);
        let (tx, rx) = oneshot::channel::<()>();
        let (bind_tx, bind_rx) = mpsc::channel::<Result<(), String>>();

        let addr = format!("127.0.0.1:{port}");
        let handle = tauri::async_runtime::spawn(async move {
            let listener = match tokio::net::TcpListener::bind(&addr).await {
                Ok(l) => l,
                Err(e) => {
                    let msg = format!("bind {addr} failed: {e}");
                    tracing::error!("{msg}");
                    let _ = bind_tx.send(Err(msg));
                    return;
                }
            };
            let _ = bind_tx.send(Ok(()));
            tracing::info!("Claude++ proxy listening on {addr}");
            let server = axum::serve(listener, app).with_graceful_shutdown(async {
                let _ = rx.await;
            });
            if let Err(e) = server.await {
                tracing::error!("server error: {e}");
            }
        });

        match bind_rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(format!("server startup failed: {e}")),
        }

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

pub fn local_gateway_token_path() -> PathBuf {
    crate::paths::app_state_dir().join(LOCAL_GATEWAY_TOKEN_FILE)
}

pub fn ensure_local_gateway_token() -> Result<String, String> {
    let path = local_gateway_token_path();
    if let Ok(text) = fs::read_to_string(&path) {
        let token = text.trim();
        if valid_local_gateway_token(token) {
            return Ok(token.to_string());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create local gateway token dir failed: {e}"))?;
    }
    let token = generate_local_gateway_token()?;
    fs::write(&path, format!("{token}\n"))
        .map_err(|e| format!("write local gateway token failed: {e}"))?;
    Ok(token)
}

fn generate_local_gateway_token() -> Result<String, String> {
    let mut bytes = [0u8; LOCAL_GATEWAY_TOKEN_BYTES];
    fill_random_bytes(&mut bytes)?;
    Ok(hex_encode(&bytes))
}

#[cfg(target_os = "windows")]
fn fill_random_bytes(bytes: &mut [u8]) -> Result<(), String> {
    #[link(name = "advapi32")]
    extern "system" {
        #[link_name = "SystemFunction036"]
        fn rtl_gen_random(buffer: *mut std::ffi::c_void, length: u32) -> u8;
    }

    let length = u32::try_from(bytes.len())
        .map_err(|_| "local gateway token buffer too large".to_string())?;
    let ok = unsafe { rtl_gen_random(bytes.as_mut_ptr().cast(), length) };
    if ok == 0 {
        Err("generate local gateway token failed".to_string())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn fill_random_bytes(bytes: &mut [u8]) -> Result<(), String> {
    File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(bytes))
        .map_err(|e| format!("generate local gateway token failed: {e}"))
}

#[cfg(not(any(target_os = "windows", unix)))]
fn fill_random_bytes(_bytes: &mut [u8]) -> Result<(), String> {
    Err("generate local gateway token failed: unsupported platform".to_string())
}

pub(crate) fn valid_local_gateway_token(token: &str) -> bool {
    token.len() == LOCAL_GATEWAY_TOKEN_HEX_LEN && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// 原子读取同一个 157210.json 里的 CC Switch 网关地址和 bearer key。
pub fn read_ccswitch_gateway_profile() -> Option<CcSwitchGatewayProfile> {
    read_ccswitch_gateway_profile_from_dirs(crate::cd_config::candidate_dirs())
}

fn read_ccswitch_gateway_profile_from_dirs<I>(dirs: I) -> Option<CcSwitchGatewayProfile>
where
    I: IntoIterator<Item = PathBuf>,
{
    dirs.into_iter()
        .filter_map(|dir| {
            let p = dir.join(format!("{CC_SWITCH_CLAUDE_DESKTOP_ENTRY_ID}.json"));
            let modified = p.metadata().and_then(|m| m.modified()).ok()?;
            let s = fs::read_to_string(&p).ok()?;
            let v = serde_json::from_str::<serde_json::Value>(&s).ok()?;
            let profile = CcSwitchGatewayProfile {
                api_key: read_profile_string_field(&v, "inferenceGatewayApiKey")?,
                base_url: read_profile_string_field(&v, "inferenceGatewayBaseUrl")?,
            };
            Some((modified, profile))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, profile)| profile)
}

fn read_profile_string_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(|field| field.as_str())
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .map(str::to_string)
}

/// 默认 DB 路径透出,供命令层使用。
pub fn default_db_path() -> PathBuf {
    ccswitch_db::default_db_path()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("claude-plus-server-{name}-{unique}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_profile(dir: &std::path::Path, body: serde_json::Value) {
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join(format!("{CC_SWITCH_CLAUDE_DESKTOP_ENTRY_ID}.json")),
            serde_json::to_string_pretty(&body).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn ccswitch_gateway_profile_requires_key_and_base_url_from_same_entry() {
        let root = temp_dir("gateway-profile");
        let complete = root.join("complete");
        let key_only = root.join("key-only");
        let base_only = root.join("base-only");

        write_profile(
            &complete,
            serde_json::json!({
                "inferenceGatewayApiKey": "complete-key",
                "inferenceGatewayBaseUrl": "http://127.0.0.1:15721/claude-desktop"
            }),
        );
        write_profile(
            &key_only,
            serde_json::json!({
                "inferenceGatewayApiKey": "orphan-key"
            }),
        );
        write_profile(
            &base_only,
            serde_json::json!({
                "inferenceGatewayBaseUrl": "http://127.0.0.1:16666/claude-desktop"
            }),
        );

        let profile = read_ccswitch_gateway_profile_from_dirs([
            key_only.clone(),
            base_only.clone(),
            complete.clone(),
        ])
        .expect("complete gateway profile");

        assert_eq!(
            profile,
            CcSwitchGatewayProfile {
                api_key: "complete-key".to_string(),
                base_url: "http://127.0.0.1:15721/claude-desktop".to_string(),
            }
        );

        let _ = fs::remove_dir_all(root);
    }
}
