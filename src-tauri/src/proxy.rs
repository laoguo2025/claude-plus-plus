// :15722 anthropic 兼容网关。
// GET  /claude-desktop/v1/models   -> 用映射的显示名生成列表(id=display_name=显示名,无 supports1m)
// POST /claude-desktop/v1/messages -> body.model 显示名->角色ID,转发 :15721,SSE 逐块透传
// 其它 /claude-desktop/*           -> 含 model 则改写后透传,否则原样透传
use crate::ccswitch_db::{self, Mapping};
use crate::server::CcSwitchGatewayProfile;
use crate::settings::ProxyRuntimeTuning;
use crate::time_utils::now_ms;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{
        header::{CACHE_CONTROL, EXPIRES, PRAGMA},
        HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri,
    },
    response::{IntoResponse, Response},
    routing::{any, get, post},
    Router,
};
use bytes::Bytes;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

const MODEL_ID_PREFIX: &str = "claude-plus-plus";
const LOCAL_GATEWAY_TOKEN_HEADER: &str = "x-claude-plus-gateway-token";
const TITLE_I18N_MAX_INPUT_CHARS: usize = 120;
const TITLE_I18N_MAX_TOKENS: u16 = 160;
const TITLE_I18N_MAX_OUTPUT_CHARS: usize = 40;
const TOKEN_USAGE_SCORE_TOTAL_CAP: u64 = 1_000_000;
const TOKEN_USAGE_MAX_JSON_DEPTH: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OriginTrust {
    Trusted,
    Missing,
    Untrusted,
}

#[derive(Debug)]
struct RateLimiter {
    window_started: Instant,
    count: u32,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self {
            window_started: Instant::now(),
            count: 0,
        }
    }
}

impl RateLimiter {
    fn allow(&mut self, now: Instant, window: Duration, max: u32) -> bool {
        if now.duration_since(self.window_started) >= window {
            self.window_started = now;
            self.count = 0;
        }
        if self.count >= max {
            return false;
        }
        self.count += 1;
        true
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db_path: PathBuf,
    pub client: reqwest::Client,
    pub local_gateway_token: String,
    /// 上次成功读取的映射缓存(读 DB 失败时回退)
    pub cache: Arc<RwLock<Vec<Mapping>>>,
    pub token_usage: Arc<RwLock<TokenUsageState>>,
    title_i18n_rate_limit: Arc<RwLock<RateLimiter>>,
    tuning: ProxyRuntimeTuning,
}

impl AppState {
    pub fn new(db_path: PathBuf, local_gateway_token: String) -> Self {
        Self {
            db_path,
            client: reqwest::Client::new(),
            local_gateway_token,
            cache: Arc::new(RwLock::new(Vec::new())),
            token_usage: Arc::new(RwLock::new(TokenUsageState::default())),
            title_i18n_rate_limit: Arc::new(RwLock::new(RateLimiter::default())),
            tuning: crate::settings::proxy_runtime_tuning(),
        }
    }

    /// 上游网关配置,实时跟随 CC Switch(读同一个配置条目 157210 的 URL 和 key)。
    fn gateway_profile(&self) -> Option<CcSwitchGatewayProfile> {
        proxy_gateway_profile(crate::server::read_ccswitch_gateway_profile())
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

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageSnapshot {
    pub id: u64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub cache_read_known: bool,
    pub cache_creation_tokens: u64,
    pub cache_creation_known: bool,
    pub context_used: u64,
    pub context_limit: u64,
    pub elapsed_ms: u64,
    pub updated_at_ms: u64,
    pub call_count: u64,
    pub source: String,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageState {
    pub usage: Option<TokenUsageSnapshot>,
    pub pending: bool,
    pub last_empty_at_ms: u64,
    pub last_error: Option<String>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/claude-plus/skills", get(handle_skills))
        .route("/claude-plus/skills/:id/trash", post(handle_trash_skill))
        .route("/claude-plus/token-usage", get(handle_token_usage))
        .route(
            "/claude-plus/conversation-title-i18n",
            post(handle_conversation_title_i18n),
        )
        .route("/claude-desktop/v1/models", get(handle_models))
        .route("/claude-desktop/v1/messages", any(handle_proxy))
        .route("/claude-desktop/*rest", any(handle_proxy))
        .with_state(state)
}

#[derive(Deserialize)]
struct TitleI18nRequest {
    title: String,
}

async fn handle_conversation_title_i18n(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    if let Some(response) = reject_untrusted_auxiliary_request(&state, &headers) {
        return response;
    }

    let title = match parse_title_i18n_request(&body) {
        Some(title) => title,
        None => {
            let mut response = (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "ok": false, "error": "title is required" })),
            )
                .into_response();
            apply_json_headers(response.headers_mut());
            return response;
        }
    };

    if !allow_title_i18n_request(&state) {
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(serde_json::json!({ "ok": false, "error": "title translation rate limit exceeded" })),
        )
            .into_response();
        apply_json_headers(response.headers_mut());
        return response;
    }

    let Some(model) = select_title_translation_model(&state.mappings()) else {
        let mut response = (
            StatusCode::BAD_GATEWAY,
            axum::Json(serde_json::json!({ "ok": false, "error": "no model mapping available" })),
        )
            .into_response();
        apply_json_headers(response.headers_mut());
        return response;
    };

    let Some(profile) = state.gateway_profile() else {
        let mut response = (
            StatusCode::BAD_GATEWAY,
            axum::Json(serde_json::json!({ "ok": false, "error": "gateway profile not found" })),
        )
            .into_response();
        apply_json_headers(response.headers_mut());
        return response;
    };
    if profile.api_key.is_empty() {
        let mut response = (
            StatusCode::BAD_GATEWAY,
            axum::Json(serde_json::json!({ "ok": false, "error": "api key not found" })),
        )
            .into_response();
        apply_json_headers(response.headers_mut());
        return response;
    }

    let upstream_url = format!("{}/v1/messages", profile.base_url.trim_end_matches('/'));
    let request_body = build_title_translation_request(&title, &model);
    let response = match state
        .client
        .post(upstream_url)
        .bearer_auth(profile.api_key)
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            let mut response = (
                StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({ "ok": false, "error": error.to_string() })),
            )
                .into_response();
            apply_json_headers(response.headers_mut());
            return response;
        }
    };

    let status = response.status();
    let payload = response
        .json::<serde_json::Value>()
        .await
        .unwrap_or_default();
    let translated = extract_title_translation(&payload);
    let mut response = if status.is_success() {
        (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "ok": translated.is_some(), "title": translated.unwrap_or_default() })),
        )
            .into_response()
    } else {
        (
            StatusCode::BAD_GATEWAY,
            axum::Json(serde_json::json!({ "ok": false, "error": "upstream translation failed" })),
        )
            .into_response()
    };
    apply_json_headers(response.headers_mut());
    response
}

fn allow_title_i18n_request(state: &AppState) -> bool {
    state
        .title_i18n_rate_limit
        .write()
        .map(|mut limiter| {
            limiter.allow(
                Instant::now(),
                state.tuning.title_i18n_rate_limit_window(),
                state.tuning.title_i18n_rate_limit_max,
            )
        })
        .unwrap_or(false)
}

async fn handle_skills(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(response) = reject_untrusted_auxiliary_request(&state, &headers) {
        return response;
    }

    let mut response = (
        StatusCode::OK,
        axum::Json(crate::claude_skills::list_skills()),
    )
        .into_response();
    apply_json_headers(response.headers_mut());
    response
}

async fn handle_trash_skill(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Some(response) = reject_untrusted_auxiliary_request(&state, &headers) {
        return response;
    }

    match crate::claude_skills::trash_skill(&id) {
        Ok(()) => {
            let mut response = (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "ok": true })),
            )
                .into_response();
            apply_json_headers(response.headers_mut());
            response
        }
        Err(error) => {
            let mut response = (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "ok": false, "error": error })),
            )
                .into_response();
            apply_json_headers(response.headers_mut());
            response
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenUsageQuery {
    since_ms: Option<u64>,
}

async fn handle_token_usage(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TokenUsageQuery>,
) -> Response {
    if let Some(response) = reject_untrusted_auxiliary_request(&state, &headers) {
        return response;
    }

    let mut token_usage = state
        .token_usage
        .read()
        .ok()
        .map(|state| state.clone())
        .unwrap_or_default();
    let db_usage = query
        .since_ms
        .and_then(|since_ms| {
            ccswitch_db::load_claude_desktop_usage(&state.db_path, Some(since_ms)).ok()
        })
        .flatten();
    if let Some(db_usage) = db_usage {
        if token_usage_fresh_for_query(
            db_usage.updated_at_ms,
            query.since_ms,
            state.tuning.token_usage_db_fresh_window_ms,
        ) {
            let usage = TokenUsageSnapshot::from(db_usage);
            token_usage.usage = Some(usage.clone());
            token_usage.pending = false;
            token_usage.last_error = None;
        }
    }
    if let Some(usage) = token_usage.usage.as_ref() {
        if !token_usage_fresh_for_query(
            usage.updated_at_ms,
            query.since_ms,
            state.tuning.token_usage_db_fresh_window_ms,
        ) {
            token_usage.usage = None;
            token_usage.pending = false;
            token_usage.last_error = None;
        }
    }
    let mut response = (
        StatusCode::OK,
        axum::Json(serde_json::json!({
            "ok": token_usage.usage.is_some(),
            "usage": token_usage.usage,
            "pending": token_usage.pending,
            "lastEmptyAtMs": token_usage.last_empty_at_ms,
            "lastError": token_usage.last_error
        })),
    )
        .into_response();
    apply_json_headers(response.headers_mut());
    response
}

fn reject_untrusted_auxiliary_request(state: &AppState, headers: &HeaderMap) -> Option<Response> {
    if matches!(local_origin_trust(headers), OriginTrust::Untrusted) {
        let mut response = (
            StatusCode::FORBIDDEN,
            axum::Json(serde_json::json!({ "ok": false, "error": "untrusted origin" })),
        )
            .into_response();
        apply_json_headers(response.headers_mut());
        return Some(response);
    }
    if local_gateway_token_matches(headers, &state.local_gateway_token) {
        return None;
    }

    let mut response = (
        StatusCode::UNAUTHORIZED,
        axum::Json(serde_json::json!({ "ok": false, "error": "local gateway token required" })),
    )
        .into_response();
    apply_json_headers(response.headers_mut());
    Some(response)
}

fn proxy_gateway_profile(
    profile: Option<CcSwitchGatewayProfile>,
) -> Option<CcSwitchGatewayProfile> {
    profile
        .filter(|profile| !profile.api_key.trim().is_empty() && !profile.base_url.trim().is_empty())
}

fn forward_proxy_header(name: &str) -> bool {
    !matches!(
        name.to_ascii_lowercase().as_str(),
        "host" | "content-length" | "authorization"
    )
}

fn local_gateway_token_matches(headers: &HeaderMap, expected: &str) -> bool {
    if expected.is_empty() {
        return false;
    }
    headers
        .get(LOCAL_GATEWAY_TOKEN_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .is_some_and(|value| value == expected)
}

fn trusted_local_origin(headers: &HeaderMap) -> bool {
    matches!(local_origin_trust(headers), OriginTrust::Trusted)
}

fn local_origin_trust(headers: &HeaderMap) -> OriginTrust {
    let Some(origin) = headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return OriginTrust::Missing;
    };

    if trusted_origin_value(origin) {
        OriginTrust::Trusted
    } else {
        OriginTrust::Untrusted
    }
}

fn bearer_token_matches(headers: &HeaderMap, expected: &str) -> bool {
    if expected.trim().is_empty() {
        return false;
    }
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .and_then(|value| {
            value
                .strip_prefix("Bearer ")
                .or_else(|| value.strip_prefix("bearer "))
        })
        .map(str::trim)
        .is_some_and(|value| value == expected)
}

fn reject_untrusted_claude_desktop_request(
    state: &AppState,
    profile: Option<&CcSwitchGatewayProfile>,
    headers: &HeaderMap,
) -> Option<Response> {
    match local_origin_trust(headers) {
        OriginTrust::Trusted => None,
        OriginTrust::Untrusted => {
            let mut response = (
                StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({ "ok": false, "error": "untrusted origin" })),
            )
                .into_response();
            apply_json_headers(response.headers_mut());
            Some(response)
        }
        OriginTrust::Missing => {
            if local_gateway_token_matches(headers, &state.local_gateway_token)
                || profile.is_some_and(|profile| bearer_token_matches(headers, &profile.api_key))
            {
                return None;
            }
            let mut response = (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({ "ok": false, "error": "local gateway credentials required" })),
            )
                .into_response();
            apply_json_headers(response.headers_mut());
            Some(response)
        }
    }
}

fn trusted_origin_value(origin: &str) -> bool {
    let Ok(uri) = origin.parse::<Uri>() else {
        return false;
    };
    let scheme = uri.scheme_str().unwrap_or_default().to_ascii_lowercase();
    let host = uri.host().unwrap_or_default().to_ascii_lowercase();
    if host == "127.0.0.1" || host == "localhost" || host == "::1" {
        return true;
    }
    if host == "claude.ai" || host.ends_with(".claude.ai") {
        return scheme == "https";
    }
    matches!(scheme.as_str(), "tauri" | "asset") && (host.is_empty() || host == "localhost")
}

fn token_usage_fresh_for_query(
    updated_at_ms: u64,
    since_ms: Option<u64>,
    fresh_window_ms: u64,
) -> bool {
    since_ms
        .map(|since_ms| updated_at_ms.saturating_add(fresh_window_ms) >= since_ms)
        .unwrap_or(true)
}

impl From<ccswitch_db::CcSwitchUsageSnapshot> for TokenUsageSnapshot {
    fn from(usage: ccswitch_db::CcSwitchUsageSnapshot) -> Self {
        let total_tokens = usage.input_tokens.saturating_add(usage.output_tokens);
        Self {
            id: usage.id,
            total_tokens,
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cached_tokens: usage.cache_read_tokens,
            cache_read_known: true,
            cache_creation_tokens: usage.cache_creation_tokens,
            cache_creation_known: true,
            context_used: total_tokens,
            context_limit: 0,
            elapsed_ms: usage.elapsed_ms,
            updated_at_ms: usage.updated_at_ms,
            call_count: 1,
            source: "cc-switch".to_string(),
        }
    }
}

fn apply_json_headers(headers: &mut HeaderMap) {
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(EXPIRES, HeaderValue::from_static("0"));
}

async fn handle_models(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let mappings = state.mappings();
    let profile = state.gateway_profile();
    if let Some(response) =
        reject_untrusted_claude_desktop_request(&state, profile.as_ref(), &headers)
    {
        return response;
    }

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
    apply_json_headers(response.headers_mut());
    response
}

fn menu_model_id(mapping: &Mapping) -> String {
    format!(
        "{MODEL_ID_PREFIX}/{}-{}",
        mapping.role_kind, mapping.display
    )
}

fn menu_display_name(mapping: &Mapping) -> String {
    mapping.display.clone()
}

fn display_to_role_from_mappings(mappings: &[Mapping], model: &str) -> Option<String> {
    mappings
        .iter()
        .find(|m| menu_model_id(m) == model)
        .or_else(|| mappings.iter().find(|m| m.display == model))
        .or_else(|| {
            mappings
                .iter()
                .find(|m| model_matches_role_kind(model, &m.role_kind))
        })
        .map(|m| m.role.clone())
}

fn model_matches_role_kind(model: &str, role_kind: &str) -> bool {
    let model = model.to_ascii_lowercase();
    let role_kind = role_kind.to_ascii_lowercase();
    if role_kind.is_empty() {
        return false;
    }
    if model == role_kind {
        return true;
    }

    let has_role_token = model
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|part| part == role_kind);
    has_role_token
        && (model.starts_with("claude-")
            || model.starts_with("now-")
            || model.contains("/claude-")
            || model.starts_with(&format!("{role_kind}-")))
}

fn parse_title_i18n_request(body: &[u8]) -> Option<String> {
    let request = serde_json::from_slice::<TitleI18nRequest>(body).ok()?;
    let title = request
        .title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!title.is_empty() && title.len() <= TITLE_I18N_MAX_INPUT_CHARS).then_some(title)
}

fn select_title_translation_model(mappings: &[Mapping]) -> Option<String> {
    mappings
        .iter()
        .find(|mapping| mapping.role_kind.eq_ignore_ascii_case("sonnet"))
        .or_else(|| {
            mappings
                .iter()
                .find(|mapping| mapping.role_kind.eq_ignore_ascii_case("opus"))
        })
        .map(|mapping| mapping.role.clone())
}

fn build_title_translation_request(title: &str, model: &str) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "max_tokens": TITLE_I18N_MAX_TOKENS,
        "thinking": {
            "type": "disabled"
        },
        "system": "你是短标题翻译器。只输出翻译后的简体中文标题，不输出思考、解释、引号或 Markdown。",
        "messages": [
            {
                "role": "user",
                "content": format!(
                    "请把下面这个英文对话标题翻译成简体中文，保留专有名词，15 个汉字以内。只输出标题，不要解释，不要 Markdown。\n\n{}",
                    title
                )
            }
        ]
    })
}

fn extract_title_translation(payload: &serde_json::Value) -> Option<String> {
    let text = payload
        .get("content")
        .and_then(|content| content.as_array())
        .and_then(|items| {
            items.iter().find_map(|item| {
                item.get("text")
                    .and_then(|text| text.as_str())
                    .or_else(|| item.get("content").and_then(|text| text.as_str()))
            })
        })
        .or_else(|| {
            payload
                .get("choices")
                .and_then(|choices| choices.as_array())
                .and_then(|choices| {
                    choices.iter().find_map(|choice| {
                        choice
                            .get("message")
                            .and_then(|message| message.get("content"))
                            .and_then(|content| content.as_str())
                            .or_else(|| choice.get("text").and_then(|text| text.as_str()))
                    })
                })
        })
        .or_else(|| payload.get("text").and_then(|text| text.as_str()))
        .or_else(|| payload.get("title").and_then(|title| title.as_str()))?;
    let cleaned = text
        .trim()
        .trim_matches('"')
        .trim_matches('“')
        .trim_matches('”')
        .trim()
        .to_string();
    (!cleaned.is_empty()
        && cleaned.chars().count() <= TITLE_I18N_MAX_OUTPUT_CHARS
        && cleaned
            .chars()
            .any(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch)))
    .then_some(cleaned)
}

async fn handle_proxy(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    // 拼上游 URL:保留原 path 中 /claude-desktop 之后的部分 + query
    let Some(profile) = state.gateway_profile() else {
        let mut response = (
            StatusCode::BAD_GATEWAY,
            axum::Json(serde_json::json!({ "ok": false, "error": "gateway profile not found" })),
        )
            .into_response();
        apply_json_headers(response.headers_mut());
        return response;
    };
    if let Some(response) =
        reject_untrusted_claude_desktop_request(&state, Some(&profile), &headers)
    {
        return response;
    }

    let upstream = profile.base_url.trim_end_matches('/');
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
        .bearer_auth(profile.api_key)
        .body(out_body.clone());
    for (k, val) in headers.iter() {
        if !forward_proxy_header(k.as_str()) {
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
    let body = Body::from_stream(track_token_usage_stream(
        stream,
        state.token_usage.clone(),
        Instant::now(),
        state.tuning.token_usage_max_pending_line,
    ));
    builder.body(body).unwrap_or_else(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "build response failed").into_response()
    })
}

fn track_token_usage_stream<S>(
    stream: S,
    store: Arc<RwLock<TokenUsageState>>,
    started_at: Instant,
    max_pending_line: usize,
) -> impl Stream<Item = Result<Bytes, reqwest::Error>>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>>,
{
    if let Ok(mut stored) = store.write() {
        stored.pending = true;
        stored.last_error = None;
    }
    TokenUsageTrackedStream {
        stream: Box::pin(stream),
        tracker: TokenUsageStreamTracker::with_max_pending_line(max_pending_line),
        store,
        started_at,
        published: false,
    }
}

struct TokenUsageTrackedStream<S> {
    stream: Pin<Box<S>>,
    tracker: TokenUsageStreamTracker,
    store: Arc<RwLock<TokenUsageState>>,
    started_at: Instant,
    published: bool,
}

impl<S> TokenUsageTrackedStream<S> {
    fn publish_final_usage(&mut self) {
        if self.published {
            return;
        }
        self.published = true;
        let tracker = std::mem::take(&mut self.tracker);
        if let Some(mut usage) = tracker.finish() {
            usage.elapsed_ms = self.started_at.elapsed().as_millis() as u64;
            usage.updated_at_ms = now_ms();
            usage.id = usage.updated_at_ms;
            if let Ok(mut stored) = self.store.write() {
                stored.usage = Some(usage);
                stored.pending = false;
                stored.last_error = None;
            }
        } else if let Ok(mut stored) = self.store.write() {
            stored.pending = false;
            stored.last_empty_at_ms = now_ms();
            stored.last_error = Some("stream ended without token usage".to_string());
        }
    }
}

impl<S> Stream for TokenUsageTrackedStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>>,
{
    type Item = Result<Bytes, reqwest::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.stream.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                let text = String::from_utf8_lossy(&chunk);
                self.tracker.ingest_text(&text);
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(None) => {
                self.publish_final_usage();
                Poll::Ready(None)
            }
            other => other,
        }
    }
}

struct TokenUsageStreamTracker {
    pending_line: String,
    aggregate: TokenUsageSnapshot,
    found: bool,
    max_pending_line: usize,
}

impl Default for TokenUsageStreamTracker {
    fn default() -> Self {
        Self::with_max_pending_line(
            crate::settings::ProxyRuntimeTuning::default().token_usage_max_pending_line,
        )
    }
}

impl TokenUsageStreamTracker {
    fn with_max_pending_line(max_pending_line: usize) -> Self {
        Self {
            pending_line: String::new(),
            aggregate: TokenUsageSnapshot::default(),
            found: false,
            max_pending_line: max_pending_line.max(1),
        }
    }

    fn ingest_text(&mut self, text: &str) -> Option<TokenUsageSnapshot> {
        self.pending_line.push_str(text);
        while let Some(newline) = self.pending_line.find('\n') {
            let line = self.pending_line[..newline]
                .trim_end_matches('\r')
                .to_string();
            self.pending_line.drain(..=newline);
            self.ingest_line(&line);
        }
        let max_pending_line = self.max_pending_line.max(1);
        if self.pending_line.len() > max_pending_line {
            trim_string_to_last_bytes(&mut self.pending_line, max_pending_line);
        }
        self.snapshot()
    }

    fn finish(mut self) -> Option<TokenUsageSnapshot> {
        if !self.pending_line.is_empty() {
            let line = std::mem::take(&mut self.pending_line);
            self.ingest_line(line.trim_end_matches('\r'));
        }
        self.snapshot()
    }

    fn ingest_line(&mut self, line: &str) {
        let line = line.trim();
        let Some(data) = line.strip_prefix("data:").map(str::trim) else {
            return;
        };
        if data.is_empty() || data == "[DONE]" {
            return;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(data) else {
            return;
        };
        if let Some(usage) = best_token_usage(&value) {
            merge_token_usage(&mut self.aggregate, &usage);
            self.found = true;
        }
    }

    fn snapshot(&self) -> Option<TokenUsageSnapshot> {
        self.found.then(|| finalized_token_usage(&self.aggregate))
    }
}

fn trim_string_to_last_bytes(text: &mut String, max_bytes: usize) {
    if text.len() <= max_bytes {
        return;
    }
    let mut keep_from = text.len() - max_bytes;
    while keep_from < text.len() && !text.is_char_boundary(keep_from) {
        keep_from += 1;
    }
    text.drain(..keep_from);
}

#[cfg(test)]
fn extract_token_usage_from_text(text: &str) -> Option<TokenUsageSnapshot> {
    let mut tracker = TokenUsageStreamTracker::with_max_pending_line(
        crate::settings::ProxyRuntimeTuning::default().token_usage_max_pending_line,
    );
    tracker.ingest_text(text).or_else(|| tracker.finish())
}

fn finalized_token_usage(usage: &TokenUsageSnapshot) -> TokenUsageSnapshot {
    let mut usage = usage.clone();
    usage.cached_tokens = usage.cached_tokens.min(usage.input_tokens);
    usage.cache_creation_tokens = usage
        .cache_creation_tokens
        .min(usage.input_tokens.saturating_sub(usage.cached_tokens));
    usage.total_tokens = usage
        .total_tokens
        .max(usage.input_tokens + usage.output_tokens);
    usage.context_used = usage.context_used.max(usage.total_tokens);
    usage
}

fn best_token_usage(value: &serde_json::Value) -> Option<TokenUsageSnapshot> {
    let mut candidates = Vec::new();
    collect_token_usage_candidates(value, 0, &mut candidates);
    candidates
        .into_iter()
        .max_by_key(|usage| token_usage_score(usage))
        .map(|usage| finalized_token_usage(&usage))
}

fn token_usage_score(usage: &TokenUsageSnapshot) -> u64 {
    let mut score = 0;
    if usage.input_tokens > 0 {
        score += 200;
    }
    if usage.output_tokens > 0 {
        score += 200;
    }
    if usage.input_tokens > 0 && usage.output_tokens > 0 {
        score += 500;
    }
    if usage.total_tokens > 0 {
        score += 100;
    }
    if usage.cached_tokens > 0 {
        score += 50;
    }
    if usage.cache_creation_tokens > 0 {
        score += 30;
    }
    if usage.context_limit > 0 {
        score += 20;
    }
    if usage.context_used > usage.total_tokens {
        score += 10;
    }
    score + usage.total_tokens.min(TOKEN_USAGE_SCORE_TOTAL_CAP)
}

fn collect_token_usage_candidates(
    value: &serde_json::Value,
    depth: usize,
    candidates: &mut Vec<TokenUsageSnapshot>,
) {
    if depth > TOKEN_USAGE_MAX_JSON_DEPTH {
        return;
    }
    if let Some(usage) = normalize_token_usage(value) {
        candidates.push(usage);
        return;
    }
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                collect_token_usage_candidates(item, depth + 1, candidates);
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(token_status) = map
                .get("last")
                .or_else(|| map.get("lastUsage"))
                .or_else(|| map.get("last_token_usage"))
                .or_else(|| map.get("lastTokenUsage"))
            {
                if let Some(context_limit) = number_field_optional(
                    value,
                    &[
                        "context_limit",
                        "contextLimit",
                        "model_context_window",
                        "modelContextWindow",
                        "context_window",
                        "contextWindow",
                    ],
                ) {
                    let mut merged = token_status.clone();
                    if let serde_json::Value::Object(ref mut merged_map) = merged {
                        merged_map.insert(
                            "contextWindow".to_string(),
                            serde_json::Value::Number(context_limit.into()),
                        );
                    }
                    if let Some(usage) = normalize_token_usage(&merged) {
                        candidates.push(usage);
                        return;
                    }
                }
            }
            for key in [
                "usage",
                "token_usage",
                "tokenUsage",
                "context_usage",
                "contextUsage",
                "last",
                "lastUsage",
                "last_token_usage",
                "lastTokenUsage",
                "message",
                "response",
                "data",
                "body",
                "result",
                "event",
                "params",
                "info",
                "completion",
                "delta",
            ] {
                if let Some(child) = map.get(key) {
                    collect_token_usage_candidates(child, depth + 1, candidates);
                }
            }
        }
        _ => {}
    }
}

fn merge_token_usage(aggregate: &mut TokenUsageSnapshot, usage: &TokenUsageSnapshot) {
    aggregate.total_tokens = aggregate.total_tokens.max(usage.total_tokens);
    aggregate.input_tokens = aggregate.input_tokens.max(usage.input_tokens);
    aggregate.output_tokens = aggregate.output_tokens.max(usage.output_tokens);
    aggregate.cached_tokens = aggregate.cached_tokens.max(usage.cached_tokens);
    aggregate.cache_read_known |= usage.cache_read_known;
    aggregate.cache_creation_tokens = aggregate
        .cache_creation_tokens
        .max(usage.cache_creation_tokens);
    aggregate.cache_creation_known |= usage.cache_creation_known;
    aggregate.context_used = aggregate.context_used.max(usage.context_used);
    aggregate.context_limit = aggregate.context_limit.max(usage.context_limit);
}

fn normalize_token_usage(value: &serde_json::Value) -> Option<TokenUsageSnapshot> {
    let input_tokens = number_field(
        value,
        &[
            "input_tokens",
            "inputTokens",
            "prompt_tokens",
            "promptTokens",
        ],
    );
    let output_tokens = number_field(
        value,
        &[
            "output_tokens",
            "outputTokens",
            "completion_tokens",
            "completionTokens",
        ],
    );
    let total_tokens = number_field(
        value,
        &[
            "total_tokens",
            "totalTokens",
            "used_tokens",
            "usedTokens",
            "used",
        ],
    )
    .max(input_tokens + output_tokens);
    let direct_cache_read =
        number_field_optional(value, &["cache_read_input_tokens", "cacheReadInputTokens"]);
    let nested_cache_read = nested_number_field_optional(
        value,
        &[
            ("prompt_tokens_details", "cached_tokens"),
            ("promptTokensDetails", "cachedTokens"),
            ("input_tokens_details", "cached_tokens"),
            ("inputTokensDetails", "cachedTokens"),
        ],
    );
    let cache_read_known = direct_cache_read.is_some() || nested_cache_read.is_some();
    let cached_tokens = direct_cache_read
        .unwrap_or(0)
        .max(nested_cache_read.unwrap_or(0));
    let cache_creation = number_field_optional(
        value,
        &["cache_creation_input_tokens", "cacheCreationInputTokens"],
    );
    let cache_creation_known = cache_creation.is_some();
    let cache_creation_tokens = cache_creation.unwrap_or(0);
    let context_used = number_field(
        value,
        &[
            "context_used",
            "contextUsed",
            "used_tokens",
            "usedTokens",
            "used",
        ],
    )
    .max(total_tokens);
    let context_limit = number_field(
        value,
        &[
            "context_limit",
            "contextLimit",
            "model_context_window",
            "modelContextWindow",
            "context_window",
            "contextWindow",
            "limit",
        ],
    );
    if input_tokens == 0
        && output_tokens == 0
        && total_tokens == 0
        && cached_tokens == 0
        && cache_creation_tokens == 0
        && context_limit == 0
    {
        return None;
    }
    Some(TokenUsageSnapshot {
        id: 0,
        total_tokens,
        input_tokens,
        output_tokens,
        cached_tokens,
        cache_read_known,
        cache_creation_tokens,
        cache_creation_known,
        context_used,
        context_limit,
        elapsed_ms: 0,
        updated_at_ms: 0,
        call_count: 0,
        source: "stream".to_string(),
    })
}

fn number_field(value: &serde_json::Value, keys: &[&str]) -> u64 {
    number_field_optional(value, keys).unwrap_or(0)
}

fn number_field_optional(value: &serde_json::Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .filter_map(|key| value.get(*key).and_then(json_number))
        .max()
}

fn nested_number_field_optional(value: &serde_json::Value, keys: &[(&str, &str)]) -> Option<u64> {
    keys.iter()
        .filter_map(|(parent, child)| value.get(*parent)?.get(*child).and_then(json_number))
        .max()
}

fn json_number(value: &serde_json::Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_f64().map(|number| number.max(0.0).round() as u64))
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
    fn display_names_preserve_ccswitch_label() {
        let opus = mapping(
            "mimo-v2.5-pro",
            "claude-opus-4-7-r2",
            "opus",
            "mimo-v2.5-pro",
        );

        assert_eq!(menu_display_name(&opus), "mimo-v2.5-pro");
    }

    #[test]
    fn display_names_allow_duplicate_user_labels() {
        let sonnet = mapping("mimo-v2.5", "claude-sonnet-4-6", "sonnet", "mimo-v2.5");
        let haiku = mapping("mimo-v2.5", "claude-haiku-4-5", "haiku", "mimo-v2.5");

        assert_eq!(menu_display_name(&sonnet), "mimo-v2.5");
        assert_eq!(menu_display_name(&haiku), "mimo-v2.5");
        assert_ne!(menu_model_id(&sonnet), menu_model_id(&haiku));
    }

    #[test]
    fn unknown_model_names_fallback_by_role_kind() {
        let mappings = vec![
            mapping(
                "mimo-v2.5-pro",
                "claude-opus-4-7-r2",
                "opus",
                "mimo-v2.5-pro",
            ),
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
        assert_eq!(
            display_to_role_from_mappings(&mappings, "Opus - mimo-v2.5-pro"),
            None
        );
    }

    #[test]
    fn role_kind_fallback_requires_token_boundary() {
        assert!(model_matches_role_kind("now-opus-4-6", "opus"));
        assert!(model_matches_role_kind(
            "claude-haiku-4-5-20251001",
            "haiku"
        ));
        assert!(model_matches_role_kind(
            "anthropic/claude-3-5-sonnet-20241022",
            "sonnet"
        ));
        assert!(!model_matches_role_kind("preopus-4-6", "opus"));
        assert!(model_matches_role_kind("claude-sonnet-4-6", "sonnet"));
        assert!(!model_matches_role_kind("test-sonnet-model", "sonnet"));
    }

    #[test]
    fn rate_limiter_resets_after_window() {
        let start = Instant::now();
        let mut limiter = RateLimiter {
            window_started: start,
            count: 0,
        };

        assert!(limiter.allow(start, Duration::from_secs(10), 2));
        assert!(limiter.allow(start + Duration::from_secs(1), Duration::from_secs(10), 2));
        assert!(!limiter.allow(start + Duration::from_secs(2), Duration::from_secs(10), 2));
        assert!(limiter.allow(start + Duration::from_secs(10), Duration::from_secs(10), 2));
    }

    #[test]
    fn title_translation_request_forces_short_simplified_chinese_title() {
        let body =
            build_title_translation_request("Prepare quarterly roadmap", "claude-sonnet-4-6");

        assert_eq!(body["model"], "claude-sonnet-4-6");
        let text = body["messages"][0]["content"].as_str().unwrap();
        assert!(text.contains("简体中文"));
        assert!(text.contains("15 个汉字以内"));
        assert!(text.contains("Prepare quarterly roadmap"));
        assert!(text.contains("不要 Markdown"));
        assert_eq!(body["thinking"]["type"], "disabled");
    }

    #[test]
    fn missing_gateway_profile_is_not_replaced_with_fixed_upstream() {
        assert!(proxy_gateway_profile(None).is_none());
    }

    #[test]
    fn proxy_forwarding_replaces_inbound_authorization() {
        assert!(!forward_proxy_header("authorization"));
        assert!(!forward_proxy_header("Authorization"));
        assert!(!forward_proxy_header("host"));
        assert!(!forward_proxy_header("content-length"));
        assert!(forward_proxy_header("accept"));
        assert!(forward_proxy_header("x-request-id"));
    }

    #[test]
    fn local_auxiliary_routes_reject_untrusted_web_origins() {
        assert!(!trusted_origin_value("https://example.com"));
        assert!(!trusted_origin_value("http://claude.ai"));
        assert!(trusted_origin_value("https://claude.ai"));
        assert!(trusted_origin_value("https://console.claude.ai"));
        assert!(trusted_origin_value("http://127.0.0.1:1420"));
        assert!(trusted_origin_value("http://localhost:1420"));
        assert!(trusted_origin_value("tauri://localhost"));
        assert!(trusted_origin_value("asset://localhost"));
    }

    #[test]
    fn local_gateway_rejects_missing_origin_without_credentials() {
        let headers = HeaderMap::new();
        assert!(!trusted_local_origin(&headers));
    }

    #[test]
    fn claude_desktop_routes_allow_missing_origin_with_gateway_bearer() {
        let state = AppState::new(PathBuf::from("missing.db"), "secret-token".to_string());
        let profile = CcSwitchGatewayProfile {
            api_key: "cc-switch-key".to_string(),
            base_url: "http://127.0.0.1:15721/claude-desktop".to_string(),
        };
        let mut headers = HeaderMap::new();

        assert!(
            reject_untrusted_claude_desktop_request(&state, Some(&profile), &headers).is_some()
        );

        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer cc-switch-key"),
        );
        assert!(
            reject_untrusted_claude_desktop_request(&state, Some(&profile), &headers).is_none()
        );
    }

    #[test]
    fn claude_desktop_routes_reject_untrusted_origin_even_with_credentials() {
        let state = AppState::new(PathBuf::from("missing.db"), "secret-token".to_string());
        let profile = CcSwitchGatewayProfile {
            api_key: "cc-switch-key".to_string(),
            base_url: "http://127.0.0.1:15721/claude-desktop".to_string(),
        };
        let mut headers = HeaderMap::new();
        headers.insert("origin", HeaderValue::from_static("https://example.com"));
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer cc-switch-key"),
        );

        assert!(
            reject_untrusted_claude_desktop_request(&state, Some(&profile), &headers).is_some()
        );
    }

    #[test]
    fn auxiliary_routes_require_local_gateway_token() {
        let state = AppState::new(PathBuf::from("missing.db"), "secret-token".to_string());
        let mut headers = HeaderMap::new();

        assert!(reject_untrusted_auxiliary_request(&state, &headers).is_some());

        headers.insert(
            HeaderName::from_static(LOCAL_GATEWAY_TOKEN_HEADER),
            HeaderValue::from_static("wrong-token"),
        );
        assert!(reject_untrusted_auxiliary_request(&state, &headers).is_some());

        headers.insert(
            HeaderName::from_static(LOCAL_GATEWAY_TOKEN_HEADER),
            HeaderValue::from_static("secret-token"),
        );
        assert!(reject_untrusted_auxiliary_request(&state, &headers).is_none());
    }

    #[test]
    fn auxiliary_routes_still_reject_untrusted_web_origins_with_token() {
        let state = AppState::new(PathBuf::from("missing.db"), "secret-token".to_string());
        let mut headers = HeaderMap::new();
        headers.insert("origin", HeaderValue::from_static("https://example.com"));
        headers.insert(
            HeaderName::from_static(LOCAL_GATEWAY_TOKEN_HEADER),
            HeaderValue::from_static("secret-token"),
        );

        assert!(reject_untrusted_auxiliary_request(&state, &headers).is_some());
    }

    #[test]
    fn title_translation_model_prefers_sonnet_then_opus_and_skips_haiku_only() {
        let mappings = vec![
            mapping("mimo-haiku", "claude-haiku-4-5", "haiku", "mimo-haiku"),
            mapping("mimo-opus", "claude-opus-4-7", "opus", "mimo-opus"),
            mapping("mimo-sonnet", "claude-sonnet-4-6", "sonnet", "mimo-sonnet"),
        ];

        assert_eq!(
            select_title_translation_model(&mappings),
            Some("claude-sonnet-4-6".to_string())
        );
        assert_eq!(
            select_title_translation_model(&mappings[..2]),
            Some("claude-opus-4-7".to_string())
        );
        assert_eq!(select_title_translation_model(&mappings[..1]), None);
    }

    #[test]
    fn extracts_openai_style_title_translation_response() {
        let payload = serde_json::json!({
            "choices": [
                {
                    "message": {
                        "content": "扫描 Alma 项目"
                    }
                }
            ]
        });

        assert_eq!(
            extract_title_translation(&payload),
            Some("扫描 Alma 项目".to_string())
        );
    }

    #[test]
    fn extracts_and_merges_token_usage_from_sse() {
        let text = [
            "event: message_start",
            "data: {\"message\":{\"usage\":{\"input_tokens\":100,\"cache_read_input_tokens\":80}}}",
            "",
            "event: message_delta",
            "data: {\"usage\":{\"output_tokens\":25,\"total_tokens\":125,\"context_window\":200000}}",
            "",
        ]
        .join("\n");

        let usage = extract_token_usage_from_text(&text).expect("token usage");

        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 25);
        assert_eq!(usage.total_tokens, 125);
        assert_eq!(usage.cached_tokens, 80);
        assert!(usage.cache_read_known);
        assert_eq!(usage.context_limit, 200000);
    }

    #[test]
    fn token_usage_does_not_treat_cached_input_as_cache_hit() {
        let text = [
            "event: message_start",
            "data: {\"usage\":{\"input_tokens\":9573,\"cached_input_tokens\":41920,\"cache_creation_input_tokens\":200}}",
            "",
            "event: message_delta",
            "data: {\"usage\":{\"output_tokens\":25,\"total_tokens\":9598}}",
            "",
        ]
        .join("\n");

        let usage = extract_token_usage_from_text(&text).expect("token usage");

        assert_eq!(usage.input_tokens, 9573);
        assert_eq!(usage.cached_tokens, 0);
        assert!(!usage.cache_read_known);
        assert_eq!(usage.cache_creation_tokens, 200);
        assert!(usage.cache_creation_known);
    }

    #[test]
    fn token_usage_prefers_same_candidate_over_cross_node_cache_mix() {
        let text = [
            "event: response.completed",
            "data: {\"usage\":{\"input_tokens\":325,\"output_tokens\":56,\"total_tokens\":381},\"token_usage\":{\"input_tokens\":325,\"cached_input_tokens\":325}}",
            "",
        ]
        .join("\n");

        let usage = extract_token_usage_from_text(&text).expect("token usage");

        assert_eq!(usage.input_tokens, 325);
        assert_eq!(usage.output_tokens, 56);
        assert_eq!(usage.total_tokens, 381);
        assert_eq!(usage.cached_tokens, 0);
        assert!(!usage.cache_read_known);
    }

    #[test]
    fn token_usage_reads_responses_input_token_details_cache() {
        let text = [
            "event: response.completed",
            "data: {\"response\":{\"usage\":{\"input_tokens\":1396,\"output_tokens\":94,\"total_tokens\":1490,\"input_tokens_details\":{\"cached_tokens\":1302}}}}",
            "",
        ]
        .join("\n");

        let usage = extract_token_usage_from_text(&text).expect("token usage");

        assert_eq!(usage.input_tokens, 1396);
        assert_eq!(usage.output_tokens, 94);
        assert_eq!(usage.total_tokens, 1490);
        assert_eq!(usage.cached_tokens, 1302);
        assert!(usage.cache_read_known);
        assert!(!usage.cache_creation_known);
    }

    #[test]
    fn token_usage_does_not_treat_generic_cached_tokens_as_cache_read() {
        let text = [
            "event: response.completed",
            "data: {\"usage\":{\"input_tokens\":2511,\"output_tokens\":614,\"total_tokens\":3125,\"cached_tokens\":2511}}",
            "",
        ]
        .join("\n");

        let usage = extract_token_usage_from_text(&text).expect("token usage");

        assert_eq!(usage.input_tokens, 2511);
        assert_eq!(usage.output_tokens, 614);
        assert_eq!(usage.total_tokens, 3125);
        assert_eq!(usage.cached_tokens, 0);
        assert!(!usage.cache_read_known);
    }

    #[test]
    fn token_usage_marks_zero_cache_as_unknown_unless_field_exists() {
        let missing_cache = extract_token_usage_from_text(
            "data: {\"usage\":{\"input_tokens\":100,\"output_tokens\":25,\"total_tokens\":125}}\n",
        )
        .expect("token usage");
        assert_eq!(missing_cache.cached_tokens, 0);
        assert!(!missing_cache.cache_read_known);
        assert_eq!(missing_cache.cache_creation_tokens, 0);
        assert!(!missing_cache.cache_creation_known);

        let explicit_zero = extract_token_usage_from_text(
            "data: {\"usage\":{\"input_tokens\":100,\"output_tokens\":25,\"cache_read_input_tokens\":0,\"cache_creation_input_tokens\":0}}\n",
        )
        .expect("token usage");
        assert_eq!(explicit_zero.cached_tokens, 0);
        assert!(explicit_zero.cache_read_known);
        assert_eq!(explicit_zero.cache_creation_tokens, 0);
        assert!(explicit_zero.cache_creation_known);
    }

    #[test]
    fn token_usage_maps_ccswitch_usage_snapshot() {
        let usage = TokenUsageSnapshot::from(ccswitch_db::CcSwitchUsageSnapshot {
            id: 1700000000002,
            input_tokens: 120,
            output_tokens: 30,
            cache_read_tokens: 900,
            cache_creation_tokens: 45,
            elapsed_ms: 2345,
            updated_at_ms: 1700000000000,
        });

        assert_eq!(usage.id, 1700000000002);
        assert_eq!(usage.total_tokens, 150);
        assert_eq!(usage.input_tokens, 120);
        assert_eq!(usage.output_tokens, 30);
        assert_eq!(usage.cached_tokens, 900);
        assert!(usage.cache_read_known);
        assert_eq!(usage.cache_creation_tokens, 45);
        assert!(usage.cache_creation_known);
        assert_eq!(usage.context_used, 150);
        assert_eq!(usage.context_limit, 0);
        assert_eq!(usage.elapsed_ms, 2345);
        assert_eq!(usage.updated_at_ms, 1700000000000);
        assert_eq!(usage.call_count, 1);
        assert_eq!(usage.source, "cc-switch");
    }

    #[test]
    fn token_usage_requires_fresh_snapshot_for_since_query() {
        assert!(token_usage_fresh_for_query(100_000, None, 15_000));
        assert!(token_usage_fresh_for_query(100_000, Some(115_000), 15_000));
        assert!(!token_usage_fresh_for_query(100_000, Some(115_001), 15_000));
        assert!(token_usage_fresh_for_query(100_000, Some(130_000), 30_000));
    }

    #[test]
    fn token_usage_tracker_handles_split_sse_lines() {
        let mut tracker = TokenUsageStreamTracker::default();

        assert!(tracker.ingest_text("data: {\"usage\":{\"input_").is_none());
        let usage = tracker
            .ingest_text("tokens\":42,\"output_tokens\":8}}\n")
            .expect("token usage");

        assert_eq!(usage.input_tokens, 42);
        assert_eq!(usage.output_tokens, 8);
        assert_eq!(usage.total_tokens, 50);
    }

    #[test]
    fn token_usage_tracker_uses_configured_pending_line_limit() {
        let mut tracker = TokenUsageStreamTracker::with_max_pending_line(8);

        assert!(tracker.ingest_text("data: {\"usage\":{\"input_").is_none());

        assert_eq!(tracker.pending_line, "{\"input_");
    }

    #[test]
    fn token_usage_tracker_trims_multibyte_pending_line_without_panic() {
        let mut tracker = TokenUsageStreamTracker::with_max_pending_line(5);

        assert!(tracker.ingest_text("prefix😀").is_none());
        assert!(tracker.pending_line.is_char_boundary(0));

        let usage = tracker
            .ingest_text("\ndata: {\"usage\":{\"input_tokens\":9,\"output_tokens\":4}}\n")
            .expect("token usage");

        assert_eq!(usage.input_tokens, 9);
        assert_eq!(usage.output_tokens, 4);
        assert_eq!(usage.total_tokens, 13);
    }

    #[tokio::test]
    async fn token_usage_stream_publishes_only_after_stream_end() {
        use futures_util::{stream, StreamExt};

        let store = Arc::new(RwLock::new(TokenUsageState {
            usage: Some(TokenUsageSnapshot {
                id: 1,
                input_tokens: 1,
                ..TokenUsageSnapshot::default()
            }),
            ..TokenUsageState::default()
        }));
        let chunks: Vec<Result<Bytes, reqwest::Error>> = vec![Ok(Bytes::from_static(
            b"data: {\"usage\":{\"input_tokens\":42,\"output_tokens\":8}}\n",
        ))];
        let mut tracked = Box::pin(track_token_usage_stream(
            stream::iter(chunks),
            store.clone(),
            Instant::now(),
            crate::settings::ProxyRuntimeTuning::default().token_usage_max_pending_line,
        ));

        assert_eq!(
            store
                .read()
                .expect("store")
                .usage
                .as_ref()
                .map(|usage| usage.id),
            Some(1)
        );
        assert!(store.read().expect("store").pending);
        assert!(tracked.next().await.expect("chunk").is_ok());
        assert_eq!(
            store
                .read()
                .expect("store")
                .usage
                .as_ref()
                .map(|usage| usage.id),
            Some(1)
        );
        assert!(tracked.next().await.is_none());

        let usage = store
            .read()
            .expect("store")
            .usage
            .clone()
            .expect("final token usage");
        assert_eq!(usage.input_tokens, 42);
        assert_eq!(usage.output_tokens, 8);
        assert_eq!(usage.total_tokens, 50);
        assert!(!store.read().expect("store").pending);
        assert!(store.read().expect("store").last_error.is_none());
    }

    #[tokio::test]
    async fn token_usage_stream_without_usage_keeps_previous_snapshot() {
        use futures_util::{stream, StreamExt};

        let previous = TokenUsageSnapshot {
            id: 7,
            input_tokens: 123,
            output_tokens: 45,
            total_tokens: 168,
            ..TokenUsageSnapshot::default()
        };
        let store = Arc::new(RwLock::new(TokenUsageState {
            usage: Some(previous.clone()),
            ..TokenUsageState::default()
        }));
        let chunks: Vec<Result<Bytes, reqwest::Error>> =
            vec![Ok(Bytes::from_static(b"data: {\"type\":\"ping\"}\n"))];
        let mut tracked = Box::pin(track_token_usage_stream(
            stream::iter(chunks),
            store.clone(),
            Instant::now(),
            crate::settings::ProxyRuntimeTuning::default().token_usage_max_pending_line,
        ));

        assert_eq!(
            store
                .read()
                .expect("store")
                .usage
                .as_ref()
                .map(|usage| usage.id),
            Some(7)
        );
        assert!(store.read().expect("store").pending);
        assert!(tracked.next().await.expect("chunk").is_ok());
        assert!(tracked.next().await.is_none());

        let usage = store
            .read()
            .expect("store")
            .usage
            .clone()
            .expect("previous token usage");
        assert_eq!(usage.id, previous.id);
        assert_eq!(usage.input_tokens, previous.input_tokens);
        assert_eq!(usage.output_tokens, previous.output_tokens);
        assert_eq!(usage.total_tokens, previous.total_tokens);
        let state = store.read().expect("store");
        assert!(!state.pending);
        assert!(state.last_empty_at_ms > 0);
        assert_eq!(
            state.last_error.as_deref(),
            Some("stream ended without token usage")
        );
    }
}
