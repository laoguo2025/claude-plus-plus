#[cfg(target_os = "windows")]
mod imp {
    use crate::claude_desktop;
    use serde::Serialize;
    use std::{
        collections::HashSet,
        env, fs,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        time::{SystemTime, UNIX_EPOCH},
    };

    const SCRIPT_MARKER: &str = "__claudePlusEnhanceNavV2";
    const NAV_API_MARKER: &str = "__claudePlusEnhanceThirdPartyApiV1";
    const NAV_PLUGINS_MARKER: &str = "__claudePlusEnhancePluginsV1";
    const NAV_MCP_MARKER: &str = "__claudePlusEnhanceMcpV1";
    const INJECT_SCRIPT: &str = r#";(()=>{const m="__claudePlusEnhanceNavV2";if(window[m])return;Object.defineProperty(window,m,{value:!0});const n=[{marker:"__claudePlusEnhanceThirdPartyApiV1",label:"第三方API",keys:["Custom inference headers","自定义推理请求头","第三方API"],icon:'<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7h16"/><path d="M4 17h16"/><path d="M7 7v10"/><path d="M17 7v10"/></svg>'},{marker:"__claudePlusEnhancePluginsV1",label:"插件与技能",keys:["Plugins & skills","插件与技能"],icon:'<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M9 3h6v5h5v6h-5v7H9v-7H4V8h5z"/></svg>'},{marker:"__claudePlusEnhanceMcpV1",label:"MCP与扩展",keys:["Connectors & extensions","连接器与扩展","MCP与扩展"],icon:'<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="12" r="3"/><circle cx="18" cy="6" r="3"/><circle cx="18" cy="18" r="3"/><path d="M8.7 10.7 15.3 7.3"/><path d="M8.7 13.3 15.3 16.7"/></svg>'}],t="claude-plus-enhance-style";function r(){if(document.getElementById(t))return;const e=document.createElement("style");e.id=t,e.textContent=".claude-plus-enhance-nav{width:100%;display:flex;align-items:center;gap:10px;border:0;background:transparent;color:inherit;text-align:left;font:inherit;border-radius:8px;padding:7px 10px;margin:1px 0;cursor:pointer;opacity:.9}.claude-plus-enhance-nav:hover{background:rgba(128,128,128,.12);opacity:1}.claude-plus-enhance-nav .cpe-icon{width:16px;height:16px;display:inline-flex;align-items:center;justify-content:center;flex:0 0 16px;opacity:.72}.claude-plus-enhance-nav .cpe-icon svg{width:16px;height:16px;display:block}.claude-plus-enhance-nav .cpe-label{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}",document.head.appendChild(e)}function o(e){return(e.textContent||"").replace(/\s+/g," ").trim()}function i(e){return Array.from(e.querySelectorAll("button,a,[role=button]"))}function p(){return n.filter(e=>window[e.marker])}function d(e,n){e.type="button",e.className="claude-plus-enhance-nav",e.dataset.claudePlusEnhance="1",e.dataset.target=n.label,e.innerHTML='<span class="cpe-icon" aria-hidden="true">'+n.icon+'</span><span class="cpe-label"></span>',e.querySelector(".cpe-label").textContent=n.label,e.onclick=()=>a(n)}function c(e){const t=i(e).find(e=>/计划任务|Scheduled/.test(o(e)));if(!t)return;const c=t.parentElement||t.parentNode;if(!c)return;const l=p(),g=Array.from(c.querySelectorAll('[data-claude-plus-enhance="1"]'));g.forEach(e=>{l.some(n=>n.label===e.dataset.target)?d(e,l.find(n=>n.label===e.dataset.target)):e.remove()}),l.slice().reverse().forEach(n=>{let e=Array.from(c.querySelectorAll('[data-claude-plus-enhance="1"]')).find(e=>e.dataset.target===n.label);e||(e=document.createElement("button"),d(e,n)),c.insertBefore(e,t.nextSibling)})}function l(){r(),document.querySelectorAll("nav,aside,[role=navigation]").forEach(c)}function a(e){u(),setTimeout(()=>s(e),180),setTimeout(()=>s(e),650),setTimeout(()=>s(e),1300)}function u(){const e=[...document.querySelectorAll("a,button,[role=button]")].find(e=>/自定义|Customize|开发者|Developer/.test(o(e)));if(e){e.click();return}try{history.pushState(null,"","/customize"),window.dispatchEvent(new PopStateEvent("popstate"))}catch(e){}}function s(e){const n=[...document.querySelectorAll("button,a,[role=button],h1,h2,h3,h4,[data-testid],label,summary,div,span")].find(n=>e.keys.some(e=>o(n).includes(e)));if(n){n.scrollIntoView({block:"center",behavior:"smooth"});const e=n.closest("button,a,[role=button],summary");e&&e.click&&e.click();const t=n.closest("section,div")||n;t.animate&&t.animate([{outline:"2px solid rgba(220,125,87,.0)"},{outline:"2px solid rgba(220,125,87,.75)"},{outline:"2px solid rgba(220,125,87,.0)"}],{duration:1100,iterations:1})}}new MutationObserver(()=>l()).observe(document.documentElement,{childList:!0,subtree:!0}),"loading"===document.readyState?document.addEventListener("DOMContentLoaded",l,{once:!0}):l()})();"#;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum EnhanceFeatureId {
        ThirdPartyApi,
        Plugins,
        Mcp,
        Markdown,
        Timeline,
    }

    impl EnhanceFeatureId {
        fn parse(value: &str) -> Option<Self> {
            match value {
                "third_party_api" => Some(Self::ThirdPartyApi),
                "plugins" => Some(Self::Plugins),
                "mcp" => Some(Self::Mcp),
                "markdown" => Some(Self::Markdown),
                "timeline" => Some(Self::Timeline),
                _ => None,
            }
        }

        fn marker(self) -> &'static str {
            match self {
                Self::ThirdPartyApi => NAV_API_MARKER,
                Self::Plugins => NAV_PLUGINS_MARKER,
                Self::Mcp => NAV_MCP_MARKER,
                Self::Markdown => "__claudePlusEnhanceMarkdownExportV1",
                Self::Timeline => "__claudePlusEnhanceTimelineV1",
            }
        }
    }

    #[derive(Serialize)]
    pub struct ClaudeEnhanceStatus {
        pub supported: bool,
        pub claude_found: bool,
        pub installed: bool,
        pub backup_available: bool,
        pub install_path: Option<String>,
        pub resources_path: Option<String>,
        pub features: Vec<EnhanceFeature>,
    }

    #[derive(Serialize)]
    pub struct EnhanceFeature {
        pub id: &'static str,
        pub label: &'static str,
        pub enabled: bool,
        pub available: bool,
        pub note: &'static str,
    }

    struct ClaudePaths {
        app: PathBuf,
        resources: PathBuf,
    }

    struct BackupContext {
        resources_path: PathBuf,
        backup_set: Option<PathBuf>,
        backed_up: HashSet<PathBuf>,
    }

    impl BackupContext {
        fn new(resources_path: &Path) -> Self {
            Self {
                resources_path: resources_path.to_path_buf(),
                backup_set: None,
                backed_up: HashSet::new(),
            }
        }

        fn ensure_set(&mut self) -> Result<PathBuf, String> {
            if let Some(path) = &self.backup_set {
                return Ok(path.clone());
            }

            let root = backup_root(&self.resources_path);
            fs::create_dir_all(&root).map_err(|e| format!("创建增强备份目录失败: {e}"))?;
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
            fs::create_dir_all(&path).map_err(|e| format!("创建增强备份集失败: {e}"))?;
            self.backup_set = Some(path.clone());
            Ok(path)
        }

        fn backup_resource(&mut self, path: &Path) -> Result<(), String> {
            if !path.exists() || self.backed_up.contains(path) {
                return Ok(());
            }

            let relative = relative_to(path, &self.resources_path)?;
            let target = self.ensure_set()?.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("创建增强备份父目录失败: {e}"))?;
            }
            fs::copy(path, &target).map_err(|e| format!("备份增强文件失败: {e}"))?;
            self.backed_up.insert(path.to_path_buf());
            Ok(())
        }
    }

    pub fn status() -> ClaudeEnhanceStatus {
        let paths = resolve_claude_paths().ok();
        let resources_path = paths.as_ref().map(|p| p.resources.clone());
        let enabled = resources_path
            .as_ref()
            .map(|path| feature_states(path))
            .unwrap_or_default();
        let installed = enabled.iter().any(|(_, is_enabled)| *is_enabled);

        ClaudeEnhanceStatus {
            supported: true,
            claude_found: paths.is_some(),
            installed,
            backup_available: resources_path
                .as_ref()
                .map(|path| latest_backup(path).is_some())
                .unwrap_or(false),
            install_path: paths.as_ref().map(|p| p.app.display().to_string()),
            resources_path: resources_path.as_ref().map(|p| p.display().to_string()),
            features: feature_list(&enabled),
        }
    }

    pub fn install(feature: &str) -> Result<(), String> {
        let feature = EnhanceFeatureId::parse(feature)
            .ok_or_else(|| format!("未知页面增强项: {feature}"))?;
        if matches!(feature, EnhanceFeatureId::Markdown | EnhanceFeatureId::Timeline) {
            return Err("该增强项将在下一阶段接入".to_string());
        }
        let paths = resolve_claude_paths()?;
        claude_desktop::stop_claude_processes()?;
        enable_write_access(&paths.resources);

        let mut backup = BackupContext::new(&paths.resources);
        update_feature_marker(&paths.resources, &mut backup, feature, true)?;
        claude_desktop::launch_claude()?;
        Ok(())
    }

    pub fn uninstall(feature: &str) -> Result<(), String> {
        let feature = EnhanceFeatureId::parse(feature)
            .ok_or_else(|| format!("未知页面增强项: {feature}"))?;
        if matches!(feature, EnhanceFeatureId::Markdown | EnhanceFeatureId::Timeline) {
            return Err("该增强项尚未接入，无需取消".to_string());
        }
        let paths = resolve_claude_paths()?;
        claude_desktop::stop_claude_processes()?;
        enable_write_access(&paths.resources);
        let mut backup = BackupContext::new(&paths.resources);
        update_feature_marker(&paths.resources, &mut backup, feature, false)?;
        claude_desktop::launch_claude()?;
        Ok(())
    }

    fn feature_list(enabled: &[(EnhanceFeatureId, bool)]) -> Vec<EnhanceFeature> {
        vec![
            EnhanceFeature {
                id: "third_party_api",
                label: "第三方API",
                enabled: is_enabled(enabled, EnhanceFeatureId::ThirdPartyApi),
                available: true,
                note: "侧边栏软入口",
            },
            EnhanceFeature {
                id: "plugins",
                label: "插件与技能",
                enabled: is_enabled(enabled, EnhanceFeatureId::Plugins),
                available: true,
                note: "侧边栏软入口",
            },
            EnhanceFeature {
                id: "mcp",
                label: "MCP与扩展",
                enabled: is_enabled(enabled, EnhanceFeatureId::Mcp),
                available: true,
                note: "侧边栏软入口",
            },
            EnhanceFeature {
                id: "markdown",
                label: "Markdown 导出",
                enabled: is_enabled(enabled, EnhanceFeatureId::Markdown),
                available: true,
                note: "待接入导出逻辑",
            },
            EnhanceFeature {
                id: "timeline",
                label: "Conversation Timeline",
                enabled: is_enabled(enabled, EnhanceFeatureId::Timeline),
                available: true,
                note: "待接入时间线逻辑",
            },
        ]
    }

    fn is_enabled(enabled: &[(EnhanceFeatureId, bool)], feature: EnhanceFeatureId) -> bool {
        enabled
            .iter()
            .find_map(|(candidate, value)| (*candidate == feature).then_some(*value))
            .unwrap_or(false)
    }

    fn feature_states(resources_path: &Path) -> Vec<(EnhanceFeatureId, bool)> {
        let text = read_index_bundle(resources_path).unwrap_or_default();
        [
            EnhanceFeatureId::ThirdPartyApi,
            EnhanceFeatureId::Plugins,
            EnhanceFeatureId::Mcp,
            EnhanceFeatureId::Markdown,
            EnhanceFeatureId::Timeline,
        ]
        .into_iter()
        .map(|feature| (feature, text.contains(feature.marker())))
        .collect()
    }

    fn update_feature_marker(
        resources_path: &Path,
        backup: &mut BackupContext,
        feature: EnhanceFeatureId,
        enabled: bool,
    ) -> Result<(), String> {
        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        let mut patched = false;
        for path in js_files(&assets_dir, true)? {
            let text = fs::read_to_string(&path).map_err(|e| format!("读取 Claude 前端入口失败: {e}"))?;
            let mut next = remove_old_script(&text);
            next = set_marker(next, feature.marker(), enabled);
            next = ensure_script(next);
            if next == text {
                patched = true;
                continue;
            }
            backup.backup_resource(&path)?;
            fs::write(&path, next)
                .map_err(|e| format!("写入 Claude 页面增强入口失败: {e}"))?;
            patched = true;
        }

        if patched {
            Ok(())
        } else {
            Err("未找到可注入的 Claude 前端入口".to_string())
        }
    }

    fn ensure_script(mut text: String) -> String {
        if !text.contains(SCRIPT_MARKER) {
            text.push_str(INJECT_SCRIPT);
        }
        text
    }

    fn remove_old_script(text: &str) -> String {
        let Some(start) = text.find(";(()=>{const m=\"__claudePlusEnhanceNavV2\"") else {
            return text.to_string();
        };
        let Some(relative_end) = text[start..].find("})();") else {
            return text.to_string();
        };
        let end = start + relative_end + "})();".len();
        format!("{}{}", &text[..start], &text[end..])
    }

    fn set_marker(mut text: String, marker: &str, enabled: bool) -> String {
        let payload = format!(";window.{marker}=true;");
        if enabled {
            if !text.contains(marker) {
                text.push_str(&payload);
            }
            return text;
        }
        text.replace(&payload, "")
    }

    fn read_index_bundle(resources_path: &Path) -> Result<String, String> {
        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        let mut output = String::new();
        for path in js_files(&assets_dir, true)? {
            let text = fs::read_to_string(&path).map_err(|e| format!("读取 Claude 前端入口失败: {e}"))?;
            output.push_str(&text);
        }
        Ok(output)
    }

    fn resolve_claude_paths() -> Result<ClaudePaths, String> {
        let app = find_claude_path().ok_or_else(|| "未找到 Claude Desktop 安装目录".to_string())?;
        let resources = resources_path_for_app(&app)
            .ok_or_else(|| format!("未找到 Claude resources 目录: {}", app.display()))?;
        Ok(ClaudePaths { app, resources })
    }

    fn find_claude_path() -> Option<PathBuf> {
        let mut candidates = Vec::new();
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
            .filter(|path| resources_path_for_app(path).is_some())
            .max_by_key(|path| modified_secs(path).unwrap_or(0))
    }

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

    fn resources_path_for_app(app: &Path) -> Option<PathBuf> {
        for candidate in [app.join("app").join("resources"), app.join("resources")] {
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
        None
    }

    fn enable_write_access(resources_path: &Path) {
        let Some(identity) = current_windows_identity() else {
            return;
        };
        for path in [
            app_path_from_resources(resources_path),
            resources_path.to_path_buf(),
            resources_path.join("ion-dist"),
            resources_path.join("ion-dist").join("assets"),
            resources_path.join("ion-dist").join("assets").join("v1"),
        ] {
            if path.exists() {
                let _ = hidden_command("icacls")
                    .arg(&path)
                    .args(["/grant", &format!("{identity}:(OI)(CI)F")])
                    .output();
            }
        }
    }

    fn latest_backup(resources_path: &Path) -> Option<PathBuf> {
        let root = backup_root(resources_path);
        let entries = fs::read_dir(root).ok()?;
        entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .max_by_key(|path| path.file_name().map(|n| n.to_os_string()))
    }

    fn js_files(assets_dir: &Path, index_only: bool) -> Result<Vec<PathBuf>, String> {
        let entries = fs::read_dir(assets_dir).map_err(|e| format!("读取 Claude 前端资源目录失败: {e}"))?;
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

    fn app_path_from_resources(resources_path: &Path) -> PathBuf {
        resources_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| resources_path.to_path_buf())
    }

    fn backup_root(resources_path: &Path) -> PathBuf {
        resources_path.join(".claude-plus-enhance-backups")
    }

    fn relative_to(path: &Path, root: &Path) -> Result<PathBuf, String> {
        let full = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        full.strip_prefix(&root)
            .map(Path::to_path_buf)
            .map_err(|_| format!("路径不在预期目录内: {}", path.display()))
    }

    fn modified_secs(path: &Path) -> Option<u64> {
        path.metadata()
            .ok()?
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs())
    }

    fn current_windows_identity() -> Option<String> {
        let output = hidden_command("whoami").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        (!text.is_empty()).then_some(text)
    }

    fn hidden_command(program: &str) -> Command {
        let mut command = Command::new(program);
        command
            .creation_flags(0x08000000)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    use std::os::windows::process::CommandExt;
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct ClaudeEnhanceStatus {
        pub supported: bool,
        pub claude_found: bool,
        pub installed: bool,
        pub backup_available: bool,
        pub install_path: Option<String>,
        pub resources_path: Option<String>,
        pub features: Vec<EnhanceFeature>,
    }

    #[derive(Serialize)]
    pub struct EnhanceFeature {
        pub label: &'static str,
        pub enabled: bool,
        pub available: bool,
        pub note: &'static str,
    }

    pub fn status() -> ClaudeEnhanceStatus {
        ClaudeEnhanceStatus {
            supported: false,
            claude_found: false,
            installed: false,
            backup_available: false,
            install_path: None,
            resources_path: None,
            features: Vec::new(),
        }
    }

    pub fn install() -> Result<(), String> {
        Err("当前只支持在 Windows 上安装 Claude Desktop 页面增强".to_string())
    }

    pub fn uninstall() -> Result<(), String> {
        Err("当前只支持在 Windows 上恢复 Claude Desktop 页面增强".to_string())
    }
}

pub use imp::*;
