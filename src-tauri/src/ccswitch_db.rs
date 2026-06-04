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
/// 角色只有这三种,版本号会变(4-6/4-7/4-8…),所以按分隔 token 匹配。
fn role_kind_of(role_key: &str) -> String {
    let lower = role_key.to_ascii_lowercase();
    if role_key_has_token(&lower, "opus") {
        "opus".to_string()
    } else if role_key_has_token(&lower, "sonnet") {
        "sonnet".to_string()
    } else if role_key_has_token(&lower, "haiku") {
        "haiku".to_string()
    } else {
        role_key.to_string()
    }
}

fn role_key_has_token(role_key: &str, target: &str) -> bool {
    role_key
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|token| token == target)
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

#[derive(Debug, Clone)]
pub struct CcSwitchUsageSnapshot {
    pub id: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub elapsed_ms: u64,
    pub updated_at_ms: u64,
}

/// 默认数据库路径。
pub fn default_db_path() -> PathBuf {
    crate::paths::ccswitch_db_path()
        .unwrap_or_else(|| PathBuf::from(".cc-switch").join("cc-switch.db"))
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

    let columns = proxy_config_columns(&conn)?;
    let has_enabled = columns.iter().any(|column| column == "enabled");
    let has_proxy_enabled = columns.iter().any(|column| column == "proxy_enabled");
    if !has_enabled && !has_proxy_enabled {
        return Err("proxy_config enabled column missing".to_string());
    }
    let enabled_expr = if has_enabled { "enabled" } else { "NULL" };
    let proxy_enabled_expr = if has_proxy_enabled {
        "proxy_enabled"
    } else {
        "NULL"
    };
    let query = format!(
        "SELECT {enabled_expr}, {proxy_enabled_expr}, listen_address, listen_port \
             FROM proxy_config \
             WHERE app_type = 'claude' LIMIT 1"
    );

    let (enabled, proxy_enabled, listen_address, listen_port): (
        Option<i64>,
        Option<i64>,
        String,
        i64,
    ) = conn
        .query_row(&query, [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| format!("query proxy config failed: {e}"))?;

    let listen_port =
        u16::try_from(listen_port).map_err(|_| "proxy listen_port out of range".to_string())?;
    let route_enabled = enabled.unwrap_or(0) != 0 || proxy_enabled.unwrap_or(0) != 0;

    Ok(ProxyConfig {
        proxy_enabled: route_enabled,
        listen_address,
        listen_port,
    })
}

fn proxy_config_columns(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(proxy_config)")
        .map_err(|e| format!("query proxy_config schema failed: {e}"))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("query proxy_config schema failed: {e}"))?;

    let mut names = Vec::new();
    for column in columns {
        names.push(column.map_err(|e| format!("query proxy_config schema failed: {e}"))?);
    }

    Ok(names)
}

/// 只读读取 CC Switch 已落库的 Claude Desktop 用量。
///
/// 传入 `since_ms` 时只取本轮开始后的最新一条日志快照；不传时取最新一条。
/// CC Switch 会把历史加载、标题、状态等后台请求也写入同表,这里不能按时间段求和。
/// 只读打开数据库,不修改 CC Switch 的任何文件或配置。
pub fn load_claude_desktop_usage(
    db_path: &std::path::Path,
    since_ms: Option<u64>,
) -> Result<Option<CcSwitchUsageSnapshot>, String> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("open db failed: {e}"))?;

    let Some(since_ms) = since_ms else {
        let row = conn
            .query_row(
                "SELECT created_at, \
                        COALESCE(input_tokens, 0), \
                        COALESCE(output_tokens, 0), \
                        COALESCE(cache_read_tokens, 0), \
                        COALESCE(cache_creation_tokens, 0), \
                        COALESCE(latency_ms, duration_ms, 0) \
                 FROM proxy_request_logs \
                 WHERE app_type = 'claude-desktop' \
                   AND data_source = 'proxy' \
                   AND status_code BETWEEN 200 AND 299 \
                 ORDER BY created_at DESC, request_id DESC \
                 LIMIT 1",
                [],
                |row| {
                    Ok(CcSwitchUsageSnapshot {
                        id: created_at_to_ms(row.get::<_, i64>(0)?),
                        input_tokens: i64_to_u64(row.get(1)?),
                        output_tokens: i64_to_u64(row.get(2)?),
                        cache_read_tokens: i64_to_u64(row.get(3)?),
                        cache_creation_tokens: i64_to_u64(row.get(4)?),
                        elapsed_ms: i64_to_u64(row.get(5)?),
                        updated_at_ms: created_at_to_ms(row.get::<_, i64>(0)?),
                    })
                },
            )
            .map(Some)
            .or_else(|err| {
                if matches!(err, rusqlite::Error::QueryReturnedNoRows) {
                    Ok(None)
                } else {
                    Err(format!("query claude-desktop usage failed: {err}"))
                }
            })?;
        return Ok(row);
    };

    let since_seconds = (since_ms / 1000) as i64;
    let row = conn
        .query_row(
            "SELECT created_at, \
                    COALESCE(input_tokens, 0), \
                    COALESCE(output_tokens, 0), \
                    COALESCE(cache_read_tokens, 0), \
                    COALESCE(cache_creation_tokens, 0), \
                    COALESCE(latency_ms, duration_ms, 0) \
             FROM proxy_request_logs \
             WHERE app_type = 'claude-desktop' \
               AND data_source = 'proxy' \
               AND status_code BETWEEN 200 AND 299 \
               AND created_at >= ?1 \
             ORDER BY created_at DESC, request_id DESC \
             LIMIT 1",
            [since_seconds],
            |row| {
                let updated_at = row.get::<_, i64>(0)?;
                Ok(Some(CcSwitchUsageSnapshot {
                    id: created_at_to_ms(updated_at),
                    input_tokens: i64_to_u64(row.get(1)?),
                    output_tokens: i64_to_u64(row.get(2)?),
                    cache_read_tokens: i64_to_u64(row.get(3)?),
                    cache_creation_tokens: i64_to_u64(row.get(4)?),
                    elapsed_ms: i64_to_u64(row.get(5)?),
                    updated_at_ms: created_at_to_ms(updated_at),
                }))
            },
        )
        .or_else(|err| {
            if matches!(err, rusqlite::Error::QueryReturnedNoRows) {
                Ok(None)
            } else {
                Err(format!("query claude-desktop usage failed: {err}"))
            }
        })?;

    Ok(row)
}

fn created_at_to_ms(created_at_seconds: i64) -> u64 {
    i64_to_u64(created_at_seconds).saturating_mul(1000)
}

fn i64_to_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_db(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        path.push(format!(
            "claude-plus-{name}-{}-{stamp}.db",
            std::process::id()
        ));
        path
    }

    fn usage_test_db() -> PathBuf {
        let path = unique_test_db("ccswitch-usage");
        let conn = Connection::open(&path).expect("create db");
        conn.execute_batch(
            "CREATE TABLE proxy_request_logs (
                request_id TEXT,
                app_type TEXT,
                data_source TEXT,
                status_code INTEGER,
                input_tokens INTEGER,
                output_tokens INTEGER,
                cache_read_tokens INTEGER,
                cache_creation_tokens INTEGER,
                latency_ms INTEGER,
                duration_ms INTEGER,
                created_at INTEGER
            );
            INSERT INTO proxy_request_logs VALUES
                ('old', 'claude-desktop', 'proxy', 200, 10, 4, 200, 5, 100, 110, 100),
                ('ignored-app', 'claude', 'proxy', 200, 999, 999, 999, 999, 999, 999, 101),
                ('ignored-status', 'claude-desktop', 'proxy', 500, 999, 999, 999, 999, 999, 999, 102),
                ('new', 'claude-desktop', 'proxy', 200, 20, 6, 300, 7, NULL, 400, 103);",
        )
        .expect("seed db");
        drop(conn);
        path
    }

    fn proxy_config_test_db(schema: &str, values: &str) -> PathBuf {
        let path = unique_test_db("ccswitch-proxy-config");
        let conn = Connection::open(&path).expect("create db");
        conn.execute_batch(&format!(
            "CREATE TABLE proxy_config ({schema});
             INSERT INTO proxy_config VALUES ({values});"
        ))
        .expect("seed db");
        drop(conn);
        path
    }

    #[test]
    fn proxy_config_accepts_proxy_enabled_when_enabled_column_is_stale() {
        let path = proxy_config_test_db(
            "app_type TEXT, proxy_enabled INTEGER, enabled INTEGER, listen_address TEXT, listen_port INTEGER",
            "'claude', 1, 0, '127.0.0.1', 15721",
        );
        let config = load_proxy_config(&path).expect("query proxy config");
        std::fs::remove_file(&path).ok();

        assert!(config.proxy_enabled);
        assert_eq!(config.listen_address, "127.0.0.1");
        assert_eq!(config.listen_port, 15721);
    }

    #[test]
    fn proxy_config_accepts_enabled_when_proxy_enabled_is_off() {
        let path = proxy_config_test_db(
            "app_type TEXT, proxy_enabled INTEGER, enabled INTEGER, listen_address TEXT, listen_port INTEGER",
            "'claude', 0, 1, '127.0.0.1', 15721",
        );
        let config = load_proxy_config(&path).expect("query proxy config");
        std::fs::remove_file(&path).ok();

        assert!(config.proxy_enabled);
        assert_eq!(config.listen_address, "127.0.0.1");
        assert_eq!(config.listen_port, 15721);
    }

    #[test]
    fn proxy_config_falls_back_to_proxy_enabled_for_legacy_schema() {
        let path = proxy_config_test_db(
            "app_type TEXT, proxy_enabled INTEGER, listen_address TEXT, listen_port INTEGER",
            "'claude', 1, '127.0.0.1', 15721",
        );
        let config = load_proxy_config(&path).expect("query proxy config");
        std::fs::remove_file(&path).ok();

        assert!(config.proxy_enabled);
        assert_eq!(config.listen_address, "127.0.0.1");
        assert_eq!(config.listen_port, 15721);
    }

    #[test]
    fn claude_desktop_usage_reads_latest_successful_proxy_row() {
        let path = usage_test_db();
        let usage = load_claude_desktop_usage(&path, None)
            .expect("query usage")
            .expect("usage");
        std::fs::remove_file(&path).ok();

        assert_eq!(usage.id, 103000);
        assert_eq!(usage.input_tokens, 20);
        assert_eq!(usage.output_tokens, 6);
        assert_eq!(usage.cache_read_tokens, 300);
        assert_eq!(usage.cache_creation_tokens, 7);
        assert_eq!(usage.elapsed_ms, 400);
        assert_eq!(usage.updated_at_ms, 103000);
    }

    #[test]
    fn claude_desktop_usage_since_turn_start_uses_latest_row_not_background_sum() {
        let path = usage_test_db();
        let usage = load_claude_desktop_usage(&path, Some(100000))
            .expect("query usage")
            .expect("usage");
        std::fs::remove_file(&path).ok();

        assert_eq!(usage.id, 103000);
        assert_eq!(usage.input_tokens, 20);
        assert_eq!(usage.output_tokens, 6);
        assert_eq!(usage.cache_read_tokens, 300);
        assert_eq!(usage.cache_creation_tokens, 7);
        assert_eq!(usage.elapsed_ms, 400);
        assert_eq!(usage.updated_at_ms, 103000);
    }

    #[test]
    fn role_kind_requires_token_boundary() {
        assert_eq!(role_kind_of("claude-opus-4-7-r2"), "opus");
        assert_eq!(role_kind_of("claude-sonnet-4-6"), "sonnet");
        assert_eq!(role_kind_of("claude-haiku-4-5"), "haiku");
        assert_eq!(
            role_kind_of("claude-notopus-4-7-r2"),
            "claude-notopus-4-7-r2"
        );
        assert_eq!(role_kind_of("sonnetlite"), "sonnetlite");
    }
}
