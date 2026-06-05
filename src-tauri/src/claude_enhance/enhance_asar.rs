use crate::claude_patch_common as patch;
use serde_json::Value;
use std::{fs, path::Path};

pub(crate) struct BridgePatch<'a> {
    pub(crate) file_path: &'a str,
    pub(crate) script: &'a str,
    pub(crate) remover: fn(&str) -> String,
}

pub(crate) fn patch_bridge_files(
    resources_path: &Path,
    patches: &[BridgePatch<'_>],
    backup: &mut patch::BackupContext,
    enabled: bool,
) -> Result<(), String> {
    let asar_path = resources_path.join("app.asar");
    let mut data = fs::read(&asar_path).map_err(|e| format!("读取 app.asar 失败: {e}"))?;
    let parsed = patch::read_asar_header(&data, &asar_path)?;
    let mut header: Value = serde_json::from_str(&parsed.header_string)
        .map_err(|e| format!("解析 app.asar 头失败: {e}"))?;
    let mut pending = Vec::new();

    for patch_spec in patches {
        let entry = patch::get_required_asar_entry(&header, patch_spec.file_path)?;
        let offset = patch::entry_value_to_usize(entry.get("offset"), "offset")?;
        let old_size = patch::entry_value_to_usize(entry.get("size"), "size")?;
        let content_offset = 8 + parsed.header_size + offset;
        let content_end = content_offset + old_size;
        if content_end > data.len() {
            return Err("app.asar 目标文件边界无效".to_string());
        }
        let content = &data[content_offset..content_end];
        if let Some(patched_content) =
            patch_bridge_content(content, patch_spec.script, enabled, patch_spec.remover)?
        {
            pending.push(PendingAsarPatch {
                file_path: patch_spec.file_path,
                offset,
                old_size,
                content_offset,
                content_end,
                patched_content,
            });
        }
    }

    if pending.is_empty() {
        return Ok(());
    }

    pending.sort_by(|a, b| b.offset.cmp(&a.offset));
    backup.backup_resource(&asar_path)?;
    for pending_patch in pending {
        let entry = patch::get_asar_entry_mut(&mut header, pending_patch.file_path)?;
        entry["size"] = Value::Number((pending_patch.patched_content.len() as u64).into());
        entry["integrity"] = patch::asar_file_integrity(&pending_patch.patched_content);
        patch::shift_asar_offsets_after(
            &mut header,
            pending_patch.offset,
            pending_patch.patched_content.len() as i64 - pending_patch.old_size as i64,
        )?;
        data.splice(
            pending_patch.content_offset..pending_patch.content_end,
            pending_patch.patched_content.iter().copied(),
        );
    }

    let header_string =
        serde_json::to_string(&header).map_err(|e| format!("生成 app.asar 头失败: {e}"))?;
    let encoded_header = patch::encode_asar_header(&header_string, Some(parsed.header_size))?;
    let content_start = 8 + parsed.header_size;
    let mut next_data = Vec::with_capacity(encoded_header.len() + data.len() - content_start);
    next_data.extend_from_slice(&encoded_header);
    next_data.extend_from_slice(&data[content_start..]);
    data = next_data;
    patch::atomic_write(&asar_path, &data).map_err(|e| format!("写入 app.asar 失败: {e}"))?;
    patch::sync_claude_exe_asar_integrity(resources_path, Some(&header_string), Some(backup))?;
    Ok(())
}

struct PendingAsarPatch<'a> {
    file_path: &'a str,
    offset: usize,
    old_size: usize,
    content_offset: usize,
    content_end: usize,
    patched_content: Vec<u8>,
}

#[cfg(test)]
pub(crate) fn patch_bridge_files_for_test(
    contents: Vec<(&str, &str)>,
    enabled: bool,
    remover: fn(&str) -> String,
    script: &str,
) -> Result<Vec<String>, String> {
    contents
        .into_iter()
        .map(|(_, content)| {
            patch_bridge_content(content.as_bytes(), script, enabled, remover).and_then(|patched| {
                String::from_utf8(patched.unwrap_or_else(|| content.as_bytes().to_vec()))
                    .map_err(|e| format!("patched bridge is not UTF-8: {e}"))
            })
        })
        .collect()
}

fn patch_bridge_content(
    content: &[u8],
    script: &str,
    enabled: bool,
    remover: fn(&str) -> String,
) -> Result<Option<Vec<u8>>, String> {
    let text = std::str::from_utf8(content).map_err(|e| format!("preload 入口不是 UTF-8: {e}"))?;
    let mut next = remover(text);
    if enabled {
        next.insert_str(0, script);
    }
    if next == text {
        Ok(None)
    } else {
        Ok(Some(next.into_bytes()))
    }
}

pub(crate) fn read_asar_file(resources_path: &Path, file_path: &str) -> Result<Vec<u8>, String> {
    let asar_path = resources_path.join("app.asar");
    let data = fs::read(&asar_path).map_err(|e| format!("读取 app.asar 失败: {e}"))?;
    let parsed = patch::read_asar_header(&data, &asar_path)?;
    let header: Value = serde_json::from_str(&parsed.header_string)
        .map_err(|e| format!("解析 app.asar 头失败: {e}"))?;
    let entry = patch::get_required_asar_entry(&header, file_path)?;
    let offset = patch::entry_value_to_usize(entry.get("offset"), "offset")?;
    let size = patch::entry_value_to_usize(entry.get("size"), "size")?;
    let content_offset = 8 + parsed.header_size + offset;
    let content_end = content_offset + size;
    if content_end > data.len() {
        return Err("app.asar 目标文件边界无效".to_string());
    }
    Ok(data[content_offset..content_end].to_vec())
}
