// 在 Claude Desktop 3P 配置库新建独立 Claude++ 条目并切为生效。
// 不修改 CC Switch 写的条目(切服务商会被覆盖),两条目共存。
use crate::constants::{
    CLAUDE_PLUS_PLUS_ENTRY_ID, CLAUDE_PLUS_PLUS_ENTRY_NAME, CLAUDE_STORE_PACKAGE_NAME,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// 运行实例(Windows Store 版)的配置库目录。
pub fn store_config_library_dir() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from)?;
    Some(
        local
            .join("Packages")
            .join(CLAUDE_STORE_PACKAGE_NAME)
            .join("LocalCache")
            .join("Roaming")
            .join("Claude-3p")
            .join("configLibrary"),
    )
}

/// 备选(非 Store 版)路径,按存在性挑选。
pub fn candidate_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(d) = store_config_library_dir() {
        v.push(d);
    }
    if let Some(roaming) = std::env::var_os("APPDATA").map(PathBuf::from) {
        v.push(roaming.join("Claude-3p").join("configLibrary"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        v.push(local.join("Claude-3p").join("configLibrary"));
    }
    v
}

/// 选第一个已存在的配置库目录;都不存在则返回 Store 版路径(由调用方决定是否创建)。
pub fn resolve_config_library_dir() -> Result<PathBuf, String> {
    for d in candidate_dirs() {
        if d.is_dir() {
            return Ok(d);
        }
    }
    store_config_library_dir().ok_or_else(|| "cannot resolve config library dir".to_string())
}

fn meta_path(dir: &Path) -> PathBuf {
    dir.join("_meta.json")
}

fn entry_path(dir: &Path) -> PathBuf {
    dir.join(format!("{CLAUDE_PLUS_PLUS_ENTRY_ID}.json"))
}

/// 接入:写 Claude++ 条目文件 + 更新 _meta.json(appliedId 指向它,保留其它条目)。
/// `port`:中间件监听端口;`api_key`:沿用 CC Switch 当前 bearer key。
pub fn apply(port: u16, api_key: &str) -> Result<(), String> {
    let dir = resolve_config_library_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create config dir failed: {e}"))?;

    let profile = json!({
        "inferenceProvider": "gateway",
        "inferenceGatewayBaseUrl": format!("http://127.0.0.1:{port}/claude-desktop"),
        "inferenceGatewayApiKey": api_key,
        "inferenceGatewayAuthScheme": "bearer",
        "disableDeploymentModeChooser": true
        // 故意不写 inferenceModels -> 强制走 /v1/models 发现模式
    });
    let profile_text = serde_json::to_string_pretty(&profile)
        .map_err(|e| format!("serialize entry failed: {e}"))?;
    std::fs::write(entry_path(&dir), profile_text)
        .map_err(|e| format!("write entry failed: {e}"))?;

    // 更新 _meta.json
    let mp = meta_path(&dir);
    let mut meta: Value = if mp.is_file() {
        let s = std::fs::read_to_string(&mp).map_err(|e| format!("read meta failed: {e}"))?;
        serde_json::from_str(&s).map_err(|e| format!("parse meta failed: {e}"))?
    } else {
        json!({ "appliedId": "", "entries": [] })
    };

    let entries = meta
        .get_mut("entries")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "meta.entries not an array".to_string())?;

    let mut exists = false;
    for entry in entries.iter_mut() {
        if entry.get("id").and_then(|i| i.as_str()) == Some(CLAUDE_PLUS_PLUS_ENTRY_ID) {
            exists = true;
            entry["name"] = json!(CLAUDE_PLUS_PLUS_ENTRY_NAME);
            break;
        }
    }
    if !exists {
        entries
            .push(json!({ "id": CLAUDE_PLUS_PLUS_ENTRY_ID, "name": CLAUDE_PLUS_PLUS_ENTRY_NAME }));
    }
    meta["appliedId"] = json!(CLAUDE_PLUS_PLUS_ENTRY_ID);

    let meta_text =
        serde_json::to_string_pretty(&meta).map_err(|e| format!("serialize meta failed: {e}"))?;
    std::fs::write(&mp, meta_text).map_err(|e| format!("write meta failed: {e}"))?;
    Ok(())
}

/// 回滚:把 appliedId 切回指定条目(默认 CC Switch 的 157210),不删除 Claude++ 条目文件。
pub fn revert(target_entry_id: Option<&str>) -> Result<(), String> {
    let dir = resolve_config_library_dir()?;
    let mp = meta_path(&dir);
    if !mp.is_file() {
        return Err("_meta.json not found".to_string());
    }
    let s = std::fs::read_to_string(&mp).map_err(|e| format!("read meta failed: {e}"))?;
    let mut meta: Value =
        serde_json::from_str(&s).map_err(|e| format!("parse meta failed: {e}"))?;

    let fallback = target_entry_id
        .map(|s| s.to_string())
        .or_else(|| {
            // 优先切回非 Claude++ 的第一个条目
            meta.get("entries")
                .and_then(|v| v.as_array())
                .and_then(|arr| {
                    arr.iter()
                        .find(|e| {
                            e.get("id").and_then(|i| i.as_str()) != Some(CLAUDE_PLUS_PLUS_ENTRY_ID)
                        })
                        .and_then(|e| e.get("id").and_then(|i| i.as_str()).map(|s| s.to_string()))
                })
        })
        .ok_or_else(|| "no fallback entry to revert to".to_string())?;

    meta["appliedId"] = json!(fallback);
    let meta_text =
        serde_json::to_string_pretty(&meta).map_err(|e| format!("serialize meta failed: {e}"))?;
    std::fs::write(&mp, meta_text).map_err(|e| format!("write meta failed: {e}"))?;
    Ok(())
}

/// 当前 appliedId 是否为 Claude++。
pub fn is_applied() -> bool {
    let Ok(dir) = resolve_config_library_dir() else {
        return false;
    };
    let mp = meta_path(&dir);
    let Ok(s) = std::fs::read_to_string(&mp) else {
        return false;
    };
    let Ok(meta) = serde_json::from_str::<Value>(&s) else {
        return false;
    };
    meta.get("appliedId").and_then(|v| v.as_str()) == Some(CLAUDE_PLUS_PLUS_ENTRY_ID)
}
