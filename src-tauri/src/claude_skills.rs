use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::UNIX_EPOCH,
};

#[derive(Clone, Serialize)]
pub struct ClaudeSkill {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub source_label: String,
    pub project_path: Option<String>,
    pub path: String,
    pub skill_file: String,
    pub description: String,
    pub summary_zh: String,
    pub modified_secs: u64,
}

#[derive(Serialize)]
pub struct ClaudeSkillsResponse {
    pub skills: Vec<ClaudeSkill>,
    pub roots: Vec<String>,
    pub project_count: usize,
}

pub fn list_skills() -> ClaudeSkillsResponse {
    let mut roots = Vec::new();
    let mut skills = Vec::new();

    if let Some(global_root) = claude_home().map(|home| home.join("skills")) {
        if global_root.is_dir() {
            roots.push(global_root.display().to_string());
            collect_skill_root(&global_root, "global", "全局", None, &mut skills);
        }
    }

    let projects = project_paths();
    for project in &projects {
        let root = project.join(".claude").join("skills");
        if root.is_dir() {
            roots.push(root.display().to_string());
            collect_skill_root(
                &root,
                "project",
                "项目",
                Some(project.display().to_string()),
                &mut skills,
            );
        }
    }

    skills.sort_by(|a, b| {
        a.scope
            .cmp(&b.scope)
            .then_with(|| a.project_path.cmp(&b.project_path))
            .then_with(|| a.name.cmp(&b.name))
    });

    ClaudeSkillsResponse {
        skills,
        roots,
        project_count: projects.len(),
    }
}

pub fn trash_skill(id: &str) -> Result<(), String> {
    let snapshot = list_skills();
    let skill = snapshot
        .skills
        .into_iter()
        .find(|skill| skill.id == id)
        .ok_or_else(|| "未找到该 skill，可能已经被删除或路径已变化".to_string())?;
    let path = PathBuf::from(&skill.path);
    ensure_known_skill_path(&path)?;
    move_dir_to_recycle_bin(&path)
}

fn collect_skill_root(
    root: &Path,
    scope: &str,
    source_label: &str,
    project_path: Option<String>,
    skills: &mut Vec<ClaudeSkill>,
) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_file = path.join("SKILL.md");
        if !skill_file.is_file() {
            continue;
        }
        if let Some(skill) = read_skill(
            scope,
            source_label,
            project_path.clone(),
            &path,
            &skill_file,
        ) {
            skills.push(skill);
        }
    }
}

fn read_skill(
    scope: &str,
    source_label: &str,
    project_path: Option<String>,
    path: &Path,
    skill_file: &Path,
) -> Option<ClaudeSkill> {
    let text = fs::read_to_string(skill_file).unwrap_or_default();
    let frontmatter = parse_frontmatter(&text);
    let fallback_name = path.file_name()?.to_string_lossy().to_string();
    let name = frontmatter
        .get("name")
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or(fallback_name);
    let description = frontmatter
        .get("description")
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| first_body_sentence(&text));
    let summary_zh = summarize_zh(&name, &description);
    let modified_secs = skill_file
        .metadata()
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    Some(ClaudeSkill {
        id: skill_id(&canonical),
        name,
        scope: scope.to_string(),
        source_label: source_label.to_string(),
        project_path,
        path: canonical.display().to_string(),
        skill_file: skill_file
            .canonicalize()
            .unwrap_or_else(|_| skill_file.to_path_buf())
            .display()
            .to_string(),
        description,
        summary_zh,
        modified_secs,
    })
}

fn parse_frontmatter(text: &str) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return values;
    }
    for line in lines {
        let line = line.trim_end();
        if line.trim() == "---" {
            break;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some((_, last)) = values.iter_mut().next_back() {
                if !last.is_empty() {
                    last.push(' ');
                }
                last.push_str(line.trim());
            }
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            values.insert(
                key.trim().to_string(),
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            );
        }
    }
    values
}

fn first_body_sentence(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && *line != "---"
                && !line.starts_with('#')
                && !line.contains(':')
                && !line.starts_with("```")
        })
        .next()
        .unwrap_or("未提供描述")
        .chars()
        .take(120)
        .collect()
}

fn summarize_zh(name: &str, description: &str) -> String {
    let desc = description.trim();
    if desc.is_empty() || desc == "未提供描述" {
        return format!("该技能用于处理「{name}」相关工作流。");
    }
    let normalized = desc
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(140)
        .collect::<String>();
    format!("该技能用于：{normalized}")
}

fn project_paths() -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    for path in projects_from_claude_json() {
        if path.is_dir() {
            paths.insert(canonical_or_self(path));
        }
    }
    for path in projects_from_session_cwds() {
        if path.is_dir() {
            paths.insert(canonical_or_self(path));
        }
    }
    for path in projects_from_cache_dirs() {
        if path.is_dir() {
            paths.insert(canonical_or_self(path));
        }
    }
    paths.into_iter().collect()
}

fn projects_from_claude_json() -> Vec<PathBuf> {
    let Some(path) = crate::paths::home_dir().map(|home| claude_config_path_from_home(&home))
    else {
        return Vec::new();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    value
        .get("projects")
        .and_then(|projects| projects.as_object())
        .map(|projects| projects.keys().map(PathBuf::from).collect())
        .unwrap_or_default()
}

fn projects_from_session_cwds() -> Vec<PathBuf> {
    let Some(root) = claude_home().map(|home| home.join("projects")) else {
        return Vec::new();
    };
    let mut paths = BTreeSet::new();
    collect_session_cwds(&root, &mut paths);
    paths.into_iter().collect()
}

fn collect_session_cwds(root: &Path, paths: &mut BTreeSet<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_session_cwds(&path, paths);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        collect_cwds_from_jsonl(&path, paths);
    }
}

fn collect_cwds_from_jsonl(path: &Path, paths: &mut BTreeSet<PathBuf>) {
    let Ok(text) = fs::read_to_string(path) else {
        return;
    };
    for line in text.lines().filter(|line| line.contains("\"cwd\"")) {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if let Some(cwd) = value.get("cwd").and_then(|cwd| cwd.as_str()) {
            paths.insert(PathBuf::from(cwd));
        }
    }
}

fn projects_from_cache_dirs() -> Vec<PathBuf> {
    let Some(root) = claude_home().map(|home| home.join("projects")) else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| decode_project_cache_name(&entry.file_name().to_string_lossy()))
        .collect()
}

fn decode_project_cache_name(name: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = name.split("--").filter(|part| !part.is_empty()).collect();
    let drive = parts.first()?.strip_suffix(':').unwrap_or(parts.first()?);
    if drive.len() != 1 {
        return None;
    }
    let mut path = PathBuf::from(format!("{drive}:\\"));
    for part in parts.iter().skip(1) {
        if let Some(name) = part.strip_prefix("claude-worktrees-") {
            path.push(".claude");
            path.push("worktrees");
            path.push(name);
        } else {
            path.push(if *part == "claude" { ".claude" } else { part });
        }
    }
    Some(path)
}

fn claude_home() -> Option<PathBuf> {
    crate::paths::home_dir().map(|home| claude_home_from_home(&home))
}

fn claude_config_path_from_home(home: &Path) -> PathBuf {
    home.join(".claude.json")
}

fn claude_home_from_home(home: &Path) -> PathBuf {
    home.join(".claude")
}

fn canonical_or_self(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn skill_id(path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.display().to_string().to_ascii_lowercase().as_bytes());
    let hash = hasher.finalize();
    hash.iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn ensure_known_skill_path(path: &Path) -> Result<(), String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("读取 skill 路径失败: {e}"))?;
    if !canonical.is_dir() || !canonical.join("SKILL.md").is_file() {
        return Err("目标不是有效 skill 目录".to_string());
    }
    let allowed = list_skills()
        .skills
        .into_iter()
        .any(|skill| PathBuf::from(skill.path) == canonical);
    if allowed {
        Ok(())
    } else {
        Err("拒绝删除未登记的 skill 路径".to_string())
    }
}

#[cfg(target_os = "windows")]
fn move_dir_to_recycle_bin(path: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    let script = r#"$ErrorActionPreference='Stop'; Add-Type -AssemblyName Microsoft.VisualBasic; $path=$env:CLAUDE_PLUS_TRASH_PATH; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory($path, [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs, [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin)"#;
    let output = Command::new("powershell.exe")
        .creation_flags(0x08000000)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("CLAUDE_PLUS_TRASH_PATH", recycle_bin_path(path))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("移动到回收站失败: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if detail.is_empty() {
            "移动到回收站失败".to_string()
        } else {
            format!("移动到回收站失败: {detail}")
        })
    }
}

#[cfg(target_os = "windows")]
fn recycle_bin_path(path: &Path) -> String {
    let raw = path.display().to_string();
    if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{rest}");
    }
    raw.strip_prefix(r"\\?\").unwrap_or(&raw).to_string()
}

#[cfg(not(target_os = "windows"))]
fn move_dir_to_recycle_bin(_path: &Path) -> Result<(), String> {
    Err("当前只支持在 Windows 上移动 skill 到回收站".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_claude_project_cache_names() {
        assert_eq!(
            decode_project_cache_name("D--Alma--claude-worktrees-demo")
                .unwrap()
                .display()
                .to_string(),
            r"D:\Alma\.claude\worktrees\demo"
        );
    }

    #[test]
    fn reads_frontmatter_description() {
        let text = "---\nname: demo\ndescription: Run focused diagnostics.\n---\nBody";
        let values = parse_frontmatter(text);
        assert_eq!(values.get("name").map(String::as_str), Some("demo"));
        assert_eq!(
            values.get("description").map(String::as_str),
            Some("Run focused diagnostics.")
        );
    }

    #[test]
    fn builds_chinese_template_summary() {
        assert_eq!(
            summarize_zh("demo", "Run focused diagnostics."),
            "该技能用于：Run focused diagnostics."
        );
        assert_eq!(
            summarize_zh("demo", ""),
            "该技能用于处理「demo」相关工作流。"
        );
    }

    #[test]
    fn claude_config_path_uses_home_path() {
        let home = PathBuf::from(r"C:\Users\Ada");

        assert_eq!(
            claude_config_path_from_home(&home),
            PathBuf::from(r"C:\Users\Ada\.claude.json")
        );
    }

    #[test]
    fn claude_home_path_uses_home_path() {
        let home = PathBuf::from(r"C:\Users\Ada");

        assert_eq!(
            claude_home_from_home(&home),
            PathBuf::from(r"C:\Users\Ada\.claude")
        );
    }

    #[test]
    fn reads_cwds_from_session_jsonl() {
        let mut paths = BTreeSet::new();
        let temp_root =
            std::env::temp_dir().join(format!("cpp-skills-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_root);
        fs::create_dir_all(&temp_root).unwrap();
        let jsonl = temp_root.join("session.jsonl");
        fs::write(
            &jsonl,
            "{\"cwd\":\"D:\\\\Alma\",\"type\":\"user\"}\nnot-json\n{\"cwd\":\"C:\\\\Users\\\\Administrator\\\\.claude\"}\n",
        )
        .unwrap();

        collect_cwds_from_jsonl(&jsonl, &mut paths);

        assert!(paths.contains(&PathBuf::from(r"D:\Alma")));
        assert!(paths.contains(&PathBuf::from(r"C:\Users\Administrator\.claude")));
        let _ = fs::remove_dir_all(&temp_root);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalizes_windows_extended_paths_for_recycle_bin() {
        assert_eq!(
            recycle_bin_path(Path::new(r"\\?\C:\Users\Administrator\.claude\skills\demo")),
            r"C:\Users\Administrator\.claude\skills\demo"
        );
        assert_eq!(
            recycle_bin_path(Path::new(r"\\?\UNC\server\share\demo")),
            r"\\server\share\demo"
        );
    }
}
