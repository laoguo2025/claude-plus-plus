// 只读 CC Switch 的 SQLite 数据库,解析当前生效 claude-desktop 服务商的模型映射。
// 数据库即唯一真相源,CC Switch 的增/改/删/切换服务商在这里重读即自动同步。
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use std::path::PathBuf;

/// 单条模型映射:显示名(picker 展示) <-> 角色 ID(转发给 CC Switch) -> 真实模型(仅展示/调试)。
#[derive(Debug, Clone, Serialize)]
pub struct Mapping {
    /// labelOverride,若空回退为 role
    pub display: String,
    /// claudeDesktopModelRoutes 的 key(转发给 CC Switch,如 claude-opus-4-7-r2),不可改
    pub role: String,
    /// 从 role key 提取的角色类别:opus / sonnet / haiku(仅展示)
    pub role_kind: String,
    /// 路由实际转发的上游模型(如 mimo-v2.5)
    pub model: String,
}

/// 从 route key(如 claude-opus-4-7-r2)提取角色类别 opus/sonnet/haiku。
/// 角色只有这三种,版本号会变(4-6/4-7/4-8…),所以按关键字匹配。
fn role_kind_of(role_key: &str) -> String {
    let lower = role_key.to_ascii_lowercase();
    if lower.contains("opus") {
        "opus".to_string()
    } else if lower.contains("sonnet") {
        "sonnet".to_string()
    } else if lower.contains("haiku") {
        "haiku".to_string()
    } else {
        role_key.to_string()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderMappings {
    /// 当前生效 claude-desktop 服务商名(只读展示)
    pub provider_name: String,
    pub provider_id: String,
    pub mappings: Vec<Mapping>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProxyConfig {
    pub proxy_enabled: bool,
    pub listen_address: String,
    pub listen_port: u16,
}

/// 默认数据库路径。
pub fn default_db_path() -> PathBuf {
    // <home>\.cc-switch\cc-switch.db
    if let Some(home) = dirs_home() {
        return home.join(".cc-switch").join("cc-switch.db");
    }
    // 兜底:相对路径(正常情况下 USERPROFILE/HOME 总能取到,不会走到这里)
    PathBuf::from(".cc-switch").join("cc-switch.db")
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
}

/// 只读读取当前生效 claude-desktop 服务商的映射。
/// readonly + 短连接,避免与 CC Switch 写冲突(其 journal_mode=delete)。
pub fn load_mappings(db_path: &std::path::Path) -> Result<ProviderMappings, String> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("open db failed: {e}"))?;

    let (provider_id, provider_name, meta_json): (String, String, String) = conn
        .query_row(
            "SELECT id, name, meta FROM providers \
             WHERE app_type = 'claude-desktop' AND is_current = 1 LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("query current claude-desktop provider failed: {e}"))?;

    let meta: serde_json::Value =
        serde_json::from_str(&meta_json).map_err(|e| format!("parse meta json failed: {e}"))?;

    let routes = meta
        .get("claudeDesktopModelRoutes")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "meta.claudeDesktopModelRoutes missing or not an object".to_string())?;

    let mut mappings = Vec::with_capacity(routes.len());
    for (role, v) in routes.iter() {
        let model = v
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string();
        let display = v
            .get("labelOverride")
            .and_then(|m| m.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(role)
            .to_string();
        mappings.push(Mapping {
            display,
            role_kind: role_kind_of(role),
            role: role.clone(),
            model,
        });
    }

    Ok(ProviderMappings {
        provider_name,
        provider_id,
        mappings,
    })
}

pub fn load_proxy_config(db_path: &std::path::Path) -> Result<ProxyConfig, String> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("open db failed: {e}"))?;

    let (proxy_enabled, listen_address, listen_port): (i64, String, i64) = conn
        .query_row(
            "SELECT proxy_enabled, listen_address, listen_port \
             FROM proxy_config \
             WHERE app_type = 'claude' LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("query proxy config failed: {e}"))?;

    let listen_port =
        u16::try_from(listen_port).map_err(|_| "proxy listen_port out of range".to_string())?;

    Ok(ProxyConfig {
        proxy_enabled: proxy_enabled != 0,
        listen_address,
        listen_port,
    })
}
