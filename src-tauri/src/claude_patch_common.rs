#[cfg(target_os = "windows")]
use serde_json::{Map, Value};
#[cfg(target_os = "windows")]
use sha2::{Digest, Sha256};
#[cfg(target_os = "windows")]
use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "windows")]
const ASAR_INTEGRITY_BLOCK_SIZE: usize = 4 * 1024 * 1024;
#[cfg(target_os = "windows")]
const MAX_BACKUP_SETS: usize = 10;

#[cfg(target_os = "windows")]
pub struct ClaudePaths {
    pub app: PathBuf,
    pub resources: PathBuf,
}

#[cfg(target_os = "windows")]
pub struct BackupContext {
    resources_path: PathBuf,
    backup_dir_name: &'static str,
    backup_set: Option<PathBuf>,
    backed_up: HashSet<PathBuf>,
}

#[cfg(target_os = "windows")]
impl BackupContext {
    pub fn new(resources_path: &Path, backup_dir_name: &'static str) -> Self {
        Self {
            resources_path: resources_path.to_path_buf(),
            backup_dir_name,
            backup_set: None,
            backed_up: HashSet::new(),
        }
    }

    pub fn has_backup(&self) -> bool {
        self.backup_set.is_some()
    }

    fn ensure_set(&mut self) -> Result<PathBuf, String> {
        if let Some(path) = &self.backup_set {
            return Ok(path.clone());
        }

        let root = backup_root(&self.resources_path, self.backup_dir_name);
        fs::create_dir_all(&root).map_err(|e| format!("创建备份目录失败: {e}"))?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("读取系统时间失败: {e}"))?
            .as_secs();
        let mut path = root.join(stamp.to_string());
        let mut suffix = 0;
        while path.exists() {
            suffix += 1;
            path = root.join(format!("{stamp}-{suffix}"));
        }
        fs::create_dir_all(&path).map_err(|e| format!("创建备份集失败: {e}"))?;
        self.backup_set = Some(path.clone());
        prune_old_backups(&root, MAX_BACKUP_SETS)?;
        Ok(path)
    }

    pub fn backup_resource(&mut self, path: &Path) -> Result<(), String> {
        if !path.exists() || self.backed_up.contains(path) {
            return Ok(());
        }

        let relative = relative_to(path, &self.resources_path)?;
        let target = self.ensure_set()?.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建备份父目录失败: {e}"))?;
        }
        fs::copy(path, &target).map_err(|e| format!("备份文件失败: {e}"))?;
        self.backed_up.insert(path.to_path_buf());
        Ok(())
    }

    pub fn backup_app_file(&mut self, path: &Path) -> Result<(), String> {
        if !path.exists() || self.backed_up.contains(path) {
            return Ok(());
        }

        let app_path = app_path_from_resources(&self.resources_path);
        let relative = relative_to(path, &app_path)?;
        let target = self.ensure_set()?.join("_app").join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建备份父目录失败: {e}"))?;
        }
        fs::copy(path, &target).map_err(|e| format!("备份 Claude 程序文件失败: {e}"))?;
        self.backed_up.insert(path.to_path_buf());
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub struct AsarHeader {
    pub header_size: usize,
    pub header_string: String,
}

#[cfg(target_os = "windows")]
pub fn resolve_claude_paths() -> Result<ClaudePaths, String> {
    let app = find_claude_path().ok_or_else(|| "未找到 Claude Desktop 安装目录".to_string())?;
    let resources = resources_path_for_app(&app)
        .ok_or_else(|| format!("未找到 Claude resources 目录: {}", app.display()))?;
    Ok(ClaudePaths { app, resources })
}

#[cfg(target_os = "windows")]
pub fn find_claude_path() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    candidates.extend(crate::settings::claude_desktop_path_overrides());
    collect_running_claude_candidates(&mut candidates);
    collect_registry_claude_candidates(&mut candidates);
    collect_shortcut_claude_candidates(&mut candidates);
    for var in ["ProgramW6432", "ProgramFiles"] {
        if let Some(root) = env::var_os(var).map(PathBuf::from) {
            collect_windows_app_candidates(&root.join("WindowsApps"), &mut candidates);
            candidates.push(root.join("Claude"));
        }
    }
    if let Some(local) = env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        candidates.push(local.join("Programs").join("Claude"));
    }

    candidates
        .into_iter()
        .flat_map(expand_claude_app_candidate)
        .filter(|path| resources_path_for_app(path).is_some())
        .max_by_key(|path| modified_secs(path).unwrap_or(0))
}

#[cfg(target_os = "windows")]
fn collect_running_claude_candidates(candidates: &mut Vec<PathBuf>) {
    let script = "(Get-CimInstance Win32_Process -Filter \"Name='Claude.exe'\" -ErrorAction SilentlyContinue).ExecutablePath";
    for line in powershell_lines(script) {
        if let Some(path) = candidate_app_path_from_exe(&PathBuf::from(line)) {
            candidates.push(path);
        }
    }
}

#[cfg(target_os = "windows")]
fn collect_registry_claude_candidates(candidates: &mut Vec<PathBuf>) {
    let script = r#"
$roots = @(
  'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
  'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
  'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*'
)
foreach ($root in $roots) {
  Get-ItemProperty $root -ErrorAction SilentlyContinue |
    Where-Object { $_.DisplayName -match 'Claude' -or $_.Publisher -match 'Anthropic' } |
    ForEach-Object {
      if ($_.InstallLocation) { $_.InstallLocation }
      if ($_.DisplayIcon) { $_.DisplayIcon }
      if ($_.UninstallString) { $_.UninstallString }
    }
}
"#;
    for line in powershell_lines(script) {
        collect_candidate_from_text(&line, candidates);
    }
}

#[cfg(target_os = "windows")]
fn collect_shortcut_claude_candidates(candidates: &mut Vec<PathBuf>) {
    let mut roots = Vec::new();
    if let Some(program_data) = env::var_os("ProgramData").map(PathBuf::from) {
        roots.push(
            program_data
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs"),
        );
    }
    if let Some(app_data) = env::var_os("APPDATA").map(PathBuf::from) {
        roots.push(
            app_data
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs"),
        );
    }

    for root in roots {
        let root = root.display().to_string().replace('\'', "''");
        let script = format!(
            "$shell=New-Object -ComObject WScript.Shell; Get-ChildItem '{root}' -Filter '*Claude*.lnk' -Recurse -ErrorAction SilentlyContinue | ForEach-Object {{ $shortcut=$shell.CreateShortcut($_.FullName); $shortcut.TargetPath }}"
        );
        for line in powershell_lines(&script) {
            collect_candidate_from_text(&line, candidates);
        }
    }
}

#[cfg(target_os = "windows")]
fn collect_candidate_from_text(text: &str, candidates: &mut Vec<PathBuf>) {
    if let Some(path) = candidate_app_path_from_exe(&PathBuf::from(trim_windows_command_path(text)))
    {
        candidates.push(path);
        return;
    }
    let path = PathBuf::from(trim_windows_command_path(text));
    if !path.as_os_str().is_empty() {
        candidates.push(path);
    }
}

#[cfg(target_os = "windows")]
fn expand_claude_app_candidate(path: PathBuf) -> Vec<PathBuf> {
    let mut candidates = vec![path.clone()];
    if let Some(app) = candidate_app_path_from_exe(&path) {
        candidates.push(app);
    }
    if path.file_name().and_then(|name| name.to_str()) == Some("resources") {
        if let Some(app) = path.parent().map(Path::to_path_buf) {
            candidates.push(app);
        }
    }
    candidates
}

#[cfg(target_os = "windows")]
fn candidate_app_path_from_exe(path: &Path) -> Option<PathBuf> {
    let file_name = path.file_name()?.to_str()?;
    if !file_name.eq_ignore_ascii_case("Claude.exe") {
        return None;
    }
    let parent = path.parent()?;
    if parent.file_name().and_then(|name| name.to_str()) == Some("app") {
        return parent.parent().map(Path::to_path_buf);
    }
    Some(parent.to_path_buf())
}

#[cfg(target_os = "windows")]
fn trim_windows_command_path(text: &str) -> String {
    let text = text.trim().trim_matches('"').trim_matches('\'');
    let lower = text.to_ascii_lowercase();
    if let Some(index) = lower.find(".exe") {
        return text[..index + 4].trim_matches('"').to_string();
    }
    text.to_string()
}

#[cfg(target_os = "windows")]
fn powershell_lines(script: &str) -> Vec<String> {
    for shell in ["pwsh.exe", "powershell.exe"] {
        let Ok(output) = hidden_command(shell)
            .args(["-NoProfile", "-Command", script])
            .output()
        else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        let lines = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if !lines.is_empty() {
            return lines;
        }
    }
    Vec::new()
}

#[cfg(target_os = "windows")]
fn collect_windows_app_candidates(root: &Path, candidates: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        collect_windows_app_candidates_with_shell(root, candidates);
        return;
    };
    let before = candidates.len();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with("Claude_") && path.is_dir() {
            candidates.push(path);
        }
    }
    if candidates.len() == before {
        collect_windows_app_candidates_with_shell(root, candidates);
    }
}

#[cfg(target_os = "windows")]
fn collect_windows_app_candidates_with_shell(root: &Path, candidates: &mut Vec<PathBuf>) {
    let root = root.display().to_string().replace('\'', "''");
    let script = format!(
        "Get-ChildItem '{root}\\Claude_*' -Directory -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | ForEach-Object {{ $_.FullName }}"
    );
    for shell in ["pwsh.exe", "powershell.exe"] {
        let Ok(output) = hidden_command(shell)
            .args(["-NoProfile", "-Command", &script])
            .output()
        else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let path = PathBuf::from(line.trim());
            if !path.as_os_str().is_empty() && path.is_dir() {
                candidates.push(path);
            }
        }
        if !candidates.is_empty() {
            break;
        }
    }
}

#[cfg(target_os = "windows")]
pub fn resources_path_for_app(app: &Path) -> Option<PathBuf> {
    for candidate in [app.join("app").join("resources"), app.join("resources")] {
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(target_os = "windows")]
pub fn enable_write_access(resources_path: &Path, include_i18n: bool) {
    let Some(identity) = current_windows_identity() else {
        return;
    };

    let mut paths = vec![
        app_path_from_resources(resources_path),
        resources_path.to_path_buf(),
        resources_path.join("ion-dist"),
        resources_path.join("ion-dist").join("assets"),
        resources_path.join("ion-dist").join("assets").join("v1"),
    ];
    if include_i18n {
        paths.push(resources_path.join("ion-dist").join("i18n"));
        paths.push(resources_path.join("ion-dist").join("i18n").join("statsig"));
    }

    for path in paths {
        if path.exists() {
            let _ = hidden_command("icacls")
                .arg(&path)
                .args(["/grant", &format!("{identity}:(OI)(CI)F")])
                .output();
        }
    }
}

#[cfg(target_os = "windows")]
pub fn latest_backup(resources_path: &Path, backup_dir_name: &str) -> Option<PathBuf> {
    let root = backup_root(resources_path, backup_dir_name);
    let entries = fs::read_dir(root).ok()?;
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .max_by_key(|path| path.file_name().map(|n| n.to_os_string()))
}

#[cfg(target_os = "windows")]
pub fn js_files(assets_dir: &Path, index_only: bool) -> Result<Vec<PathBuf>, String> {
    let entries =
        fs::read_dir(assets_dir).map_err(|e| format!("读取 Claude 前端资源目录失败: {e}"))?;
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if path.extension().and_then(|e| e.to_str()) == Some("js")
            && (!index_only || name.starts_with("index-"))
        {
            files.push(path);
        }
    }
    if files.is_empty() {
        return Err("未找到 Claude 前端 JS bundle".to_string());
    }
    Ok(files)
}

#[cfg(target_os = "windows")]
pub fn read_asar_header(data: &[u8], path: &Path) -> Result<AsarHeader, String> {
    if data.len() < 16 {
        return Err(format!("不支持的 app.asar 头: {}", path.display()));
    }
    let size_pickle_payload = read_u32_le(data, 0)? as usize;
    let header_size = read_u32_le(data, 4)? as usize;
    if size_pickle_payload != 4 || header_size == 0 || data.len() < 8 + header_size {
        return Err(format!("不支持的 app.asar size pickle: {}", path.display()));
    }

    let header_pickle = &data[8..8 + header_size];
    let header_payload_size = read_u32_le(header_pickle, 0)? as usize;
    let header_string_size = read_i32_le(header_pickle, 4)? as usize;
    let expected_payload_size = align4(4 + header_string_size);
    if header_payload_size != expected_payload_size || header_size != 4 + header_payload_size {
        return Err(format!(
            "不支持的 app.asar header pickle: {}",
            path.display()
        ));
    }
    let header_bytes = &header_pickle[8..8 + header_string_size];
    let header_string = String::from_utf8(header_bytes.to_vec())
        .map_err(|e| format!("app.asar 头不是 UTF-8: {e}"))?;
    Ok(AsarHeader {
        header_size,
        header_string,
    })
}

#[cfg(target_os = "windows")]
pub fn encode_asar_header(
    header_string: &str,
    expected_header_size: Option<usize>,
) -> Result<Vec<u8>, String> {
    let header_bytes = header_string.as_bytes();
    let header_payload_size = align4(4 + header_bytes.len());
    let header_size = 4 + header_payload_size;
    let target_header_size = expected_header_size.unwrap_or(header_size);
    if header_size != target_header_size {
        return Err("app.asar 头长度变化，拒绝写入不安全补丁".to_string());
    }

    let mut header_pickle = vec![0u8; target_header_size];
    header_pickle[0..4].copy_from_slice(&(header_payload_size as u32).to_le_bytes());
    header_pickle[4..8].copy_from_slice(&(header_bytes.len() as i32).to_le_bytes());
    header_pickle[8..8 + header_bytes.len()].copy_from_slice(header_bytes);

    let mut encoded = vec![0u8; 8 + target_header_size];
    encoded[0..4].copy_from_slice(&4u32.to_le_bytes());
    encoded[4..8].copy_from_slice(&(target_header_size as u32).to_le_bytes());
    encoded[8..].copy_from_slice(&header_pickle);
    Ok(encoded)
}

#[cfg(target_os = "windows")]
pub fn get_asar_entry<'a>(header: &'a Value, file_path: &str) -> Result<&'a Value, String> {
    let mut node = header;
    for part in file_path.split('/') {
        let files = node
            .get("files")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("app.asar 中未找到 {file_path}"))?;
        node = files
            .get(part)
            .ok_or_else(|| format!("app.asar 中未找到 {file_path}"))?;
    }
    Ok(node)
}

#[cfg(target_os = "windows")]
pub fn get_required_asar_entry<'a>(
    header: &'a Value,
    file_path: &str,
) -> Result<&'a Value, String> {
    let node = get_asar_entry(header, file_path)?;
    ensure_asar_entry_fields(node)?;
    Ok(node)
}

#[cfg(target_os = "windows")]
pub fn get_asar_entry_mut<'a>(
    header: &'a mut Value,
    file_path: &str,
) -> Result<&'a mut Value, String> {
    let mut node = header;
    for part in file_path.split('/') {
        let files = node
            .get_mut("files")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| format!("app.asar 中未找到 {file_path}"))?;
        node = files
            .get_mut(part)
            .ok_or_else(|| format!("app.asar 中未找到 {file_path}"))?;
    }
    ensure_asar_entry_fields(node)?;
    Ok(node)
}

#[cfg(target_os = "windows")]
fn ensure_asar_entry_fields(node: &Value) -> Result<(), String> {
    for key in ["size", "offset", "integrity"] {
        if node.get(key).is_none() {
            return Err(format!("app.asar 目标缺少字段: {key}"));
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn shift_asar_offsets_after(
    header: &mut Value,
    changed_offset: usize,
    delta: i64,
) -> Result<(), String> {
    if delta == 0 {
        return Ok(());
    }
    fn visit(node: &mut Value, changed_offset: usize, delta: i64) -> Result<(), String> {
        if let Some(files) = node.get_mut("files").and_then(Value::as_object_mut) {
            for child in files.values_mut() {
                visit(child, changed_offset, delta)?;
            }
            return Ok(());
        }
        let Some(offset_value) = node.get_mut("offset") else {
            return Ok(());
        };
        let Some(current) = offset_value
            .as_u64()
            .or_else(|| offset_value.as_str().and_then(|s| s.parse::<u64>().ok()))
        else {
            return Err("app.asar offset 无效".to_string());
        };
        if current as usize > changed_offset {
            let next = current as i64 + delta;
            if next < 0 {
                return Err("app.asar offset 计算越界".to_string());
            }
            *offset_value = match offset_value {
                Value::String(_) => Value::String(next.to_string()),
                _ => Value::Number((next as u64).into()),
            };
        }
        Ok(())
    }
    visit(header, changed_offset, delta)
}

#[cfg(target_os = "windows")]
pub fn entry_value_to_usize(value: Option<&Value>, name: &str) -> Result<usize, String> {
    match value {
        Some(Value::Number(n)) => n
            .as_u64()
            .and_then(|v| usize::try_from(v).ok())
            .ok_or_else(|| format!("app.asar {name} 无效")),
        Some(Value::String(s)) => s
            .parse::<usize>()
            .map_err(|_| format!("app.asar {name} 无效")),
        _ => Err(format!("app.asar {name} 无效")),
    }
}

#[cfg(target_os = "windows")]
pub fn asar_file_integrity(data: &[u8]) -> Value {
    let mut blocks = Vec::new();
    if data.is_empty() {
        blocks.push(Value::String(sha256_hex(data)));
    } else {
        for chunk in data.chunks(ASAR_INTEGRITY_BLOCK_SIZE) {
            blocks.push(Value::String(sha256_hex(chunk)));
        }
    }
    let mut integrity = Map::new();
    integrity.insert("algorithm".to_string(), Value::String("SHA256".to_string()));
    integrity.insert("hash".to_string(), Value::String(sha256_hex(data)));
    integrity.insert(
        "blockSize".to_string(),
        Value::Number((ASAR_INTEGRITY_BLOCK_SIZE as u64).into()),
    );
    integrity.insert("blocks".to_string(), Value::Array(blocks));
    Value::Object(integrity)
}

#[cfg(target_os = "windows")]
pub fn sync_claude_exe_asar_integrity(
    resources_path: &Path,
    header_string: Option<&str>,
    backup: Option<&mut BackupContext>,
) -> Result<(), String> {
    let asar_path = resources_path.join("app.asar");
    let header_hash = match header_string {
        Some(s) => sha256_hex(s.as_bytes()),
        None => {
            let data = fs::read(&asar_path).map_err(|e| format!("读取 app.asar 失败: {e}"))?;
            let parsed = read_asar_header(&data, &asar_path)?;
            sha256_hex(parsed.header_string.as_bytes())
        }
    };

    let app_path = app_path_from_resources(resources_path);
    let exe_path = [app_path.join("Claude.exe"), app_path.join("claude.exe")]
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| "未找到 Claude.exe".to_string())?;
    let mut exe = fs::read(&exe_path).map_err(|e| format!("读取 Claude.exe 失败: {e}"))?;
    let marker = b"resources\\\\app.asar\",\"alg\":\"SHA256\",\"value\":\"";
    let matches = find_pattern(&exe, marker);
    if matches.len() != 1 {
        return Err("未找到 Claude.exe 内嵌 app.asar 完整性标记".to_string());
    }
    let hash_offset = matches[0] + marker.len();
    if hash_offset + 64 > exe.len() {
        return Err("Claude.exe 完整性标记边界无效".to_string());
    }
    let current = std::str::from_utf8(&exe[hash_offset..hash_offset + 64])
        .map_err(|e| format!("Claude.exe 完整性哈希不是 UTF-8: {e}"))?;
    if current == header_hash {
        return Ok(());
    }
    if !current.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Claude.exe 完整性哈希不是 SHA256 十六进制".to_string());
    }

    if let Some(backup) = backup {
        backup.backup_app_file(&exe_path)?;
    }
    exe[hash_offset..hash_offset + 64].copy_from_slice(header_hash.as_bytes());
    fs::write(&exe_path, exe).map_err(|e| format!("写入 Claude.exe 失败: {e}"))?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn app_path_from_resources(resources_path: &Path) -> PathBuf {
    resources_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| resources_path.to_path_buf())
}

#[cfg(target_os = "windows")]
fn backup_root(resources_path: &Path, backup_dir_name: &str) -> PathBuf {
    resources_path.join(backup_dir_name)
}

#[cfg(target_os = "windows")]
pub fn relative_to(path: &Path, root: &Path) -> Result<PathBuf, String> {
    let full = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    full.strip_prefix(&root)
        .map(Path::to_path_buf)
        .map_err(|_| format!("路径不在预期目录内: {}", path.display()))
}

#[cfg(target_os = "windows")]
fn prune_old_backups(root: &Path, keep: usize) -> Result<(), String> {
    let mut entries = fs::read_dir(root)
        .map_err(|e| format!("读取备份目录失败: {e}"))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    entries.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));
    let remove_count = entries.len().saturating_sub(keep);
    for path in entries.into_iter().take(remove_count) {
        fs::remove_dir_all(&path).map_err(|e| format!("清理旧备份失败: {e}"))?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn modified_secs(path: &Path) -> Option<u64> {
    path.metadata()
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

#[cfg(target_os = "windows")]
fn align4(value: usize) -> usize {
    value + ((4 - (value % 4)) % 4)
}

#[cfg(target_os = "windows")]
fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "读取 u32 越界".to_string())?;
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}

#[cfg(target_os = "windows")]
fn read_i32_le(bytes: &[u8], offset: usize) -> Result<i32, String> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "读取 i32 越界".to_string())?;
    Ok(i32::from_le_bytes(slice.try_into().unwrap()))
}

#[cfg(target_os = "windows")]
fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(target_os = "windows")]
fn find_pattern(data: &[u8], pattern: &[u8]) -> Vec<usize> {
    if pattern.is_empty() || data.len() < pattern.len() {
        return Vec::new();
    }
    data.windows(pattern.len())
        .enumerate()
        .filter_map(|(index, window)| (window == pattern).then_some(index))
        .collect()
}

#[cfg(target_os = "windows")]
fn current_windows_identity() -> Option<String> {
    let output = hidden_command("whoami").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

#[cfg(target_os = "windows")]
pub fn hidden_command(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .creation_flags(0x08000000)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn encode_asar_header_rejects_unexpected_size_change() {
        let encoded = encode_asar_header("{}", None).expect("encoded header");
        assert!(encode_asar_header("{\"files\":{}}", Some(encoded.len() - 8)).is_err());
    }

    #[test]
    fn candidate_app_path_accepts_exe_inside_app_directory() {
        let path = PathBuf::from(r"C:\Users\Ada\AppData\Local\Programs\Claude\Claude.exe");
        assert_eq!(
            candidate_app_path_from_exe(&path),
            Some(PathBuf::from(r"C:\Users\Ada\AppData\Local\Programs\Claude"))
        );
    }

    #[test]
    fn candidate_app_path_accepts_exe_inside_windows_app_subdir() {
        let path = PathBuf::from(
            r"C:\Program Files\WindowsApps\Anthropic.Claude_1.2.3_x64__abcd\app\Claude.exe",
        );
        assert_eq!(
            candidate_app_path_from_exe(&path),
            Some(PathBuf::from(
                r"C:\Program Files\WindowsApps\Anthropic.Claude_1.2.3_x64__abcd"
            ))
        );
    }
}
