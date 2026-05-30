// :15722 anthropic 兼容网关。
// GET  /claude-desktop/v1/models   -> 用映射的显示名生成列表(id=display_name=显示名,无 supports1m)
// POST /claude-desktop/v1/messages -> body.model 显示名->角色ID,转发 :15721,SSE 逐块透传
// 其它 /claude-desktop/*           -> 含 model 则改写后透传,否则原样透传
use crate::ccswitch_db::{self, Mapping};
use axum::{
    body::Body,
    extract::State,
    http::{
        header::{CACHE_CONTROL, EXPIRES, PRAGMA},
        HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri,
    },
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 上游地址兜底(读不到 CC Switch 配置时用)。正常应从 CC Switch 读取。
const UPSTREAM_FALLBACK: &str = "http://127.0.0.1:15721/claude-desktop";
const MODEL_ID_PREFIX: &str = "claude-plus-plus";

#[derive(Clone)]
pub struct AppState {
    pub db_path: PathBuf,
    pub client: reqwest::Client,
    /// 上次成功读取的映射缓存(读 DB 失败时回退)
    pub cache: Arc<RwLock<Vec<Mapping>>>,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 上游网关地址,实时跟随 CC Switch(读 157210.json 的 inferenceGatewayBaseUrl)。
    fn upstream(&self) -> String {
        crate::server::read_ccswitch_base_url().unwrap_or_else(|| UPSTREAM_FALLBACK.to_string())
    }

    /// 读 DB 取映射,失败时回退缓存;成功则刷新缓存。
    fn mappings(&self) -> Vec<Mapping> {
        match ccswitch_db::load_mappings(&self.db_path) {
            Ok(pm) => {
                if let Ok(mut c) = self.cache.write() {
                    *c = pm.mappings.clone();
                }
                pm.mappings
            }
            Err(e) => {
                tracing::warn!("load_mappings failed, fallback to cache: {e}");
                self.cache.read().map(|c| c.clone()).unwrap_or_default()
            }
        }
    }

    fn display_to_role(&self, display: &str) -> Option<String> {
        display_to_role_from_mappings(&self.mappings(), display)
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/claude-desktop/v1/models", get(handle_models))
        .route("/claude-desktop/v1/messages", any(handle_proxy))
        .route("/claude-desktop/*rest", any(handle_proxy))
        .with_state(state)
}

async fn handle_models(State(state): State<AppState>) -> Response {
    let mappings = state.mappings();
    let data: Vec<serde_json::Value> = mappings
        .iter()
        .map(|m| {
            serde_json::json!({
                "type": "model",
                "id": menu_model_id(m),
                "display_name": menu_display_name(m),
                "created_at": "2024-01-01T00:00:00Z"
            })
        })
        .collect();
    let first = data.first().and_then(|v| v.get("id")).cloned();
    let last = data.last().and_then(|v| v.get("id")).cloned();
    let body = serde_json::json!({
        "data": data,
        "has_more": false,
        "first_id": first,
        "last_id": last
    });
    let mut response = (StatusCode::OK, axum::Json(body)).into_response();
    let headers = response.headers_mut();
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(EXPIRES, HeaderValue::from_static("0"));
    response
}

fn menu_model_id(mapping: &Mapping) -> String {
    format!("{MODEL_ID_PREFIX}/{}-{}", mapping.role_kind, mapping.display)
}

fn menu_display_name(mapping: &Mapping) -> String {
    format!("{} - {}", role_label(&mapping.role_kind), mapping.display)
}

fn role_label(role_kind: &str) -> String {
    let mut chars = role_kind.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => "Model".to_string(),
    }
}

fn display_to_role_from_mappings(mappings: &[Mapping], model: &str) -> Option<String> {
    mappings
        .iter()
        .find(|m| menu_model_id(m) == model || menu_display_name(m) == model)
        .or_else(|| mappings.iter().find(|m| m.display == model))
        .or_else(|| mappings.iter().find(|m| model_matches_role_kind(model, &m.role_kind)))
        .map(|m| m.role.clone())
}

fn model_matches_role_kind(model: &str, role_kind: &str) -> bool {
    let model = model.to_ascii_lowercase();
    let role_kind = role_kind.to_ascii_lowercase();
    !role_kind.is_empty() && model.contains(&role_kind)
}

async fn handle_proxy(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    // 拼上游 URL:保留原 path 中 /claude-desktop 之后的部分 + query
    let upstream = state.upstream();
    let path = uri.path();
    let suffix = path.strip_prefix("/claude-desktop").unwrap_or(path);
    let query = uri.query().map(|q| format!("?{q}")).unwrap_or_default();
    let upstream_url = format!("{upstream}{suffix}{query}");

    // 若 body 是 JSON 且含 model,做 显示名->角色ID 改写
    let out_body: Vec<u8> = if !body.is_empty() {
        match serde_json::from_slice::<serde_json::Value>(&body) {
            Ok(mut v) => {
                if let Some(model) = v.get("model").and_then(|m| m.as_str()) {
                    if let Some(role) = state.display_to_role(model) {
                        v["model"] = serde_json::Value::String(role);
                    } else {
                        tracing::warn!("unknown model from picker: {model}");
                    }
                    serde_json::to_vec(&v).unwrap_or_else(|_| body.to_vec())
                } else {
                    body.to_vec()
                }
            }
            Err(_) => body.to_vec(),
        }
    } else {
        body.to_vec()
    };

    // 透传请求头(去掉 host/content-length,reqwest 自管)
    let mut req = state
        .client
        .request(method, &upstream_url)
        .body(out_body.clone());
    for (k, val) in headers.iter() {
        let kn = k.as_str().to_ascii_lowercase();
        if kn == "host" || kn == "content-length" {
            continue;
        }
        req = req.header(k, val);
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("upstream request failed: {e}");
            return (StatusCode::BAD_GATEWAY, format!("upstream error: {e}")).into_response();
        }
    };

    let status = resp.status();
    let resp_headers = resp.headers().clone();

    // 构造响应:流式透传上游字节流
    let mut builder = Response::builder().status(status.as_u16());
    for (k, v) in resp_headers.iter() {
        let kn = k.as_str().to_ascii_lowercase();
        // content-length 不透传(流式),transfer-encoding/connection 让 axum 自处理
        if kn == "content-length" || kn == "transfer-encoding" || kn == "connection" {
            continue;
        }
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(k.as_str().as_bytes()),
            HeaderValue::from_bytes(v.as_bytes()),
        ) {
            builder = builder.header(name, val);
        }
    }

    let stream = resp.bytes_stream();
    let body = Body::from_stream(stream);
    builder.body(body).unwrap_or_else(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "build response failed").into_response()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mapping(display: &str, role: &str, role_kind: &str, model: &str) -> Mapping {
        Mapping {
            display: display.to_string(),
            role: role.to_string(),
            role_kind: role_kind.to_string(),
            model: model.to_string(),
        }
    }

    #[test]
    fn model_ids_are_unique_by_role_even_when_display_names_match() {
        let sonnet = mapping("mimo-v2.5", "claude-sonnet-4-6", "sonnet", "mimo-v2.5");
        let haiku = mapping("mimo-v2.5", "claude-haiku-4-5", "haiku", "mimo-v2.5");

        assert_eq!(menu_model_id(&sonnet), "claude-plus-plus/sonnet-mimo-v2.5");
        assert_eq!(menu_model_id(&haiku), "claude-plus-plus/haiku-mimo-v2.5");
        assert_ne!(menu_model_id(&sonnet), menu_model_id(&haiku));
    }

    #[test]
    fn display_names_keep_role_and_model_visible() {
        let opus = mapping(
            "mimo-v2.5-pro",
            "claude-opus-4-7-r2",
            "opus",
            "mimo-v2.5-pro",
        );

        assert_eq!(menu_display_name(&opus), "Opus - mimo-v2.5-pro");
    }

    #[test]
    fn unknown_model_names_fallback_by_role_kind() {
        let mappings = vec![
            mapping("mimo-v2.5-pro", "claude-opus-4-7-r2", "opus", "mimo-v2.5-pro"),
            mapping("mimo-v2.5", "claude-sonnet-4-6", "sonnet", "mimo-v2.5"),
            mapping("mimo-v2.5", "claude-haiku-4-5", "haiku", "mimo-v2.5"),
        ];

        assert_eq!(
            display_to_role_from_mappings(&mappings, "now-opus-4-6"),
            Some("claude-opus-4-7-r2".to_string())
        );
        assert_eq!(
            display_to_role_from_mappings(&mappings, "claude-haiku-4-5-20251001"),
            Some("claude-haiku-4-5".to_string())
        );
    }
}
