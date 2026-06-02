#[cfg(target_os = "windows")]
mod imp {
    use crate::claude_desktop;
    use regex::{bytes::Regex as BytesRegex, Regex};
    use serde::Serialize;
    use serde_json::{Map, Value};
    use sha2::{Digest, Sha256};
    use std::{
        collections::HashSet,
        env, fs, io,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        time::{SystemTime, UNIX_EPOCH},
    };

    const LANGS: &[&str] = &["zh-CN", "zh-TW", "zh-HK"];
    const BASE_LANGUAGE_LIST: &str = r#"["en-US","de-DE","fr-FR","ko-KR","ja-JP","es-419","es-ES","it-IT","hi-IN","pt-BR","id-ID""#;
    const ASAR_PATCH_TARGET: &str = ".vite/build/index.js";
    const ASAR_INTEGRITY_BLOCK_SIZE: usize = 4 * 1024 * 1024;

    #[derive(Serialize)]
    pub struct ClaudeZhStatus {
        pub supported: bool,
        pub claude_found: bool,
        pub installed: bool,
        pub backup_available: bool,
        pub claude_version: Option<String>,
        pub install_path: Option<String>,
        pub resources_path: Option<String>,
        pub locale: Option<String>,
        pub language_files: Vec<String>,
    }

    struct LanguagePack {
        frontend: &'static str,
        frontend_visible_overrides: &'static str,
        frontend_hardcoded: &'static str,
        desktop: &'static str,
        statsig: &'static str,
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
            Ok(path)
        }

        fn backup_resource(&mut self, path: &Path) -> Result<(), String> {
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

        fn backup_app_file(&mut self, path: &Path) -> Result<(), String> {
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

    pub fn status() -> ClaudeZhStatus {
        let paths = resolve_claude_paths().ok();
        let resources_path = paths.as_ref().map(|p| p.resources.clone());
        let language_files = resources_path
            .as_ref()
            .map(|path| {
                LANGS
                    .iter()
                    .filter(|lang| {
                        path.join("ion-dist")
                            .join("i18n")
                            .join(format!("{lang}.json"))
                            .is_file()
                    })
                    .map(|lang| (*lang).to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        ClaudeZhStatus {
            supported: true,
            claude_found: paths.is_some(),
            installed: !language_files.is_empty(),
            backup_available: resources_path
                .as_ref()
                .map(|path| latest_backup(path).is_some())
                .unwrap_or(false),
            claude_version: paths
                .as_ref()
                .and_then(|paths| claude_version(&paths.app, &paths.resources)),
            install_path: paths.as_ref().map(|p| p.app.display().to_string()),
            resources_path: resources_path.as_ref().map(|p| p.display().to_string()),
            locale: read_current_locale(),
            language_files,
        }
    }

    pub fn install(language: &str, skip_asar_patch: bool) -> Result<(), String> {
        validate_language(language)?;
        let pack = language_pack(language)?;
        let paths = resolve_claude_paths()?;
        claude_desktop::stop_claude_processes()?;
        enable_write_access(&paths.resources);

        let mut backup = BackupContext::new(&paths.resources);
        install_language_files(&paths.resources, &pack, language, &mut backup)?;
        register_language(&paths.resources, language, &mut backup)?;
        patch_hardcoded_frontend_strings(&paths.resources, &pack, &mut backup)?;
        patch_language_display_names(&paths.resources, &mut backup)?;

        if !skip_asar_patch {
            patch_hardcoded_main_process_menu_labels(&paths.resources, language, &mut backup)?;
            patch_custom_3p_model_validation(&paths.resources, &mut backup)?;
        }

        set_claude_locale(language)?;
        claude_desktop::launch_claude()?;
        Ok(())
    }

    pub fn backup() -> Result<(), String> {
        let paths = resolve_claude_paths()?;
        enable_write_access(&paths.resources);
        let mut backup = BackupContext::new(&paths.resources);
        backup_current_claude_files(&paths.resources, &mut backup)?;
        backup
            .backup_set
            .as_ref()
            .map(|_| ())
            .ok_or_else(|| "未找到可备份的 Claude Desktop 资源".to_string())
    }

    pub fn uninstall() -> Result<(), String> {
        let paths = resolve_claude_paths()?;
        claude_desktop::stop_claude_processes()?;
        enable_write_access(&paths.resources);
        restore_latest_backup(&paths.resources)?;
        sync_claude_exe_asar_integrity(&paths.resources, None, None)?;
        remove_language_files(&paths.resources)?;
        unregister_languages(&paths.resources)?;
        set_claude_locale("en-US")?;
        claude_desktop::launch_claude()?;
        Ok(())
    }

    fn backup_current_claude_files(
        resources_path: &Path,
        backup: &mut BackupContext,
    ) -> Result<(), String> {
        for lang in LANGS {
            for path in [
                resources_path
                    .join("ion-dist")
                    .join("i18n")
                    .join(format!("{lang}.json")),
                resources_path.join(format!("{lang}.json")),
                resources_path
                    .join("ion-dist")
                    .join("i18n")
                    .join("statsig")
                    .join(format!("{lang}.json")),
            ] {
                backup.backup_resource(&path)?;
            }
        }

        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        if assets_dir.is_dir() {
            for entry in
                fs::read_dir(&assets_dir).map_err(|e| format!("读取前端资源目录失败: {e}"))?
            {
                let path = entry
                    .map_err(|e| format!("读取前端资源项失败: {e}"))?
                    .path();
                if path.extension().and_then(|e| e.to_str()) == Some("js") {
                    backup.backup_resource(&path)?;
                }
            }
        }

        backup.backup_resource(&resources_path.join("app.asar"))?;
        let app_path = app_path_from_resources(resources_path);
        for path in [app_path.join("Claude.exe"), app_path.join("claude.exe")] {
            backup.backup_app_file(&path)?;
        }
        Ok(())
    }

    fn validate_language(language: &str) -> Result<(), String> {
        if LANGS.contains(&language) {
            Ok(())
        } else {
            Err(format!("不支持的语言: {language}"))
        }
    }

    fn language_pack(language: &str) -> Result<LanguagePack, String> {
        match language {
            "zh-CN" => Ok(LanguagePack {
                frontend: include_str!("../resources/claude-zh/frontend-zh-CN.json"),
                frontend_visible_overrides: include_str!(
                    "../resources/claude-zh/frontend-visible-overrides-zh-CN.json"
                ),
                frontend_hardcoded: include_str!(
                    "../resources/claude-zh/frontend-hardcoded-zh-CN.json"
                ),
                desktop: include_str!("../resources/claude-zh/desktop-zh-CN.json"),
                statsig: include_str!("../resources/claude-zh/statsig-zh-CN.json"),
            }),
            "zh-TW" => Ok(LanguagePack {
                frontend: include_str!("../resources/claude-zh/frontend-zh-TW.json"),
                frontend_visible_overrides: "{}",
                frontend_hardcoded: include_str!(
                    "../resources/claude-zh/frontend-hardcoded-zh-TW.json"
                ),
                desktop: include_str!("../resources/claude-zh/desktop-zh-TW.json"),
                statsig: include_str!("../resources/claude-zh/statsig-zh-TW.json"),
            }),
            "zh-HK" => Ok(LanguagePack {
                frontend: include_str!("../resources/claude-zh/frontend-zh-HK.json"),
                frontend_visible_overrides: "{}",
                frontend_hardcoded: include_str!(
                    "../resources/claude-zh/frontend-hardcoded-zh-HK.json"
                ),
                desktop: include_str!("../resources/claude-zh/desktop-zh-HK.json"),
                statsig: include_str!("../resources/claude-zh/statsig-zh-HK.json"),
            }),
            _ => Err(format!("不支持的语言: {language}")),
        }
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

    fn claude_version(app: &Path, resources_path: &Path) -> Option<String> {
        read_package_json_version(resources_path)
            .or_else(|| read_package_json_version(&app.join("app").join("resources")))
            .or_else(|| read_asar_package_version(resources_path))
            .or_else(|| exe_product_version(app))
    }

    fn read_package_json_version(resources_path: &Path) -> Option<String> {
        let text = fs::read_to_string(resources_path.join("app").join("package.json")).ok()?;
        let value: Value = serde_json::from_str(&text).ok()?;
        value
            .get("version")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|version| !version.is_empty())
            .map(ToOwned::to_owned)
    }

    fn read_asar_package_version(resources_path: &Path) -> Option<String> {
        let data = fs::read(resources_path.join("app.asar")).ok()?;
        let parsed = read_asar_header(&data, Path::new("app.asar")).ok()?;
        let header: Value = serde_json::from_str(&parsed.header_string).ok()?;
        let entry = get_asar_entry(&header, "package.json")?;
        let offset = entry_value_to_usize(entry.get("offset"), "offset").ok()?;
        let size = entry_value_to_usize(entry.get("size"), "size").ok()?;
        let content_offset = 8 + parsed.header_size + offset;
        let content_end = content_offset + size;
        let content = data.get(content_offset..content_end)?;
        let value: Value = serde_json::from_slice(content).ok()?;
        value
            .get("version")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|version| !version.is_empty())
            .map(ToOwned::to_owned)
    }

    fn exe_product_version(app: &Path) -> Option<String> {
        let exe = [app.join("Claude.exe"), app.join("claude.exe")]
            .into_iter()
            .find(|path| path.is_file())?;
        let exe_path = exe.display().to_string().replace('\'', "''");
        let script = format!(
            "(Get-Item -LiteralPath '{exe_path}' -ErrorAction SilentlyContinue).VersionInfo.ProductVersion"
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
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version.is_empty() {
                return Some(version);
            }
        }
        None
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

    fn enable_write_access(resources_path: &Path) {
        let Some(identity) = current_windows_identity() else {
            return;
        };
        for path in [
            app_path_from_resources(resources_path),
            resources_path.to_path_buf(),
            resources_path.join("ion-dist"),
            resources_path.join("ion-dist").join("i18n"),
            resources_path.join("ion-dist").join("i18n").join("statsig"),
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

    fn current_windows_identity() -> Option<String> {
        let output = hidden_command("whoami").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        (!text.is_empty()).then_some(text)
    }

    fn install_language_files(
        resources_path: &Path,
        pack: &LanguagePack,
        language: &str,
        backup: &mut BackupContext,
    ) -> Result<(), String> {
        let i18n_dir = resources_path.join("ion-dist").join("i18n");
        let statsig_dir = i18n_dir.join("statsig");
        fs::create_dir_all(&i18n_dir).map_err(|e| format!("创建前端语言目录失败: {e}"))?;
        fs::create_dir_all(&statsig_dir).map_err(|e| format!("创建 statsig 语言目录失败: {e}"))?;

        let frontend_target = i18n_dir.join(format!("{language}.json"));
        let desktop_target = resources_path.join(format!("{language}.json"));
        let statsig_target = statsig_dir.join(format!("{language}.json"));
        backup.backup_resource(&frontend_target)?;
        backup.backup_resource(&desktop_target)?;
        backup.backup_resource(&statsig_target)?;

        let frontend = merge_frontend_locale(
            &i18n_dir.join("en-US.json"),
            pack.frontend,
            pack.frontend_visible_overrides,
        )?;
        write_utf8(&frontend_target, &frontend)?;
        write_utf8(&desktop_target, pack.desktop)?;
        write_utf8(&statsig_target, pack.statsig)?;
        Ok(())
    }

    fn merge_frontend_locale(
        en_path: &Path,
        zh_pack: &str,
        visible_overrides: &str,
    ) -> Result<String, String> {
        let en_text = fs::read_to_string(en_path)
            .map_err(|e| format!("读取当前 Claude 英文语言文件失败: {e}"))?;
        let en: Value =
            serde_json::from_str(&en_text).map_err(|e| format!("解析英文语言文件失败: {e}"))?;
        let zh: Value =
            serde_json::from_str(zh_pack).map_err(|e| format!("解析中文语言包失败: {e}"))?;
        let overrides: Value = serde_json::from_str(visible_overrides)
            .map_err(|e| format!("解析可见文案覆盖包失败: {e}"))?;
        let en = en
            .as_object()
            .ok_or_else(|| "英文语言文件不是 JSON 对象".to_string())?;
        let zh = zh
            .as_object()
            .ok_or_else(|| "中文语言包不是 JSON 对象".to_string())?;
        let overrides = overrides
            .as_object()
            .ok_or_else(|| "可见文案覆盖包不是 JSON 对象".to_string())?;

        let mut merged = Map::new();
        for (key, value) in en {
            merged.insert(
                key.clone(),
                overrides
                    .get(key)
                    .or_else(|| zh.get(key))
                    .cloned()
                    .unwrap_or_else(|| value.clone()),
            );
        }
        serde_json::to_string(&Value::Object(merged))
            .map_err(|e| format!("生成合并语言文件失败: {e}"))
    }

    fn register_language(
        resources_path: &Path,
        language: &str,
        backup: &mut BackupContext,
    ) -> Result<(), String> {
        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        let regex = Regex::new(
            r#"\["en-US","de-DE","fr-FR","ko-KR","ja-JP","es-419","es-ES","it-IT","hi-IN","pt-BR","id-ID"(?:(?:,"zh-CN")|(?:,"zh-TW")|(?:,"zh-HK"))*\]"#,
        )
        .map_err(|e| format!("创建语言白名单匹配器失败: {e}"))?;
        let replacement = format!(r#"{BASE_LANGUAGE_LIST},"{language}"]"#);
        let mut touched = false;

        for path in js_files(&assets_dir, true)? {
            let text =
                fs::read_to_string(&path).map_err(|e| format!("读取前端入口文件失败: {e}"))?;
            if text.contains(&replacement) {
                touched = true;
                continue;
            }
            if regex.is_match(&text) {
                backup.backup_resource(&path)?;
                let updated = regex.replace(&text, replacement.as_str()).to_string();
                write_utf8(&path, &updated)?;
                touched = true;
            }
        }

        if touched {
            Ok(())
        } else {
            Err("未能注册中文语言，Claude 前端 bundle 格式可能已经变化".to_string())
        }
    }

    fn patch_language_display_names(
        resources_path: &Path,
        backup: &mut BackupContext,
    ) -> Result<(), String> {
        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        let marker = "__claudeZhLabelPatch";
        let patch = r#";(()=>{const e=Intl.DisplayNames&&Intl.DisplayNames.prototype;if(!e||e.__claudeZhLabelPatch)return;const n=e.of;e.of=function(e){const t=String(e);return t==="zh-CN"?"简体中文":t==="zh-HK"?"繁体中文（中国香港）":t==="zh-TW"?"繁体中文（中国台湾）":n.call(this,e)},Object.defineProperty(e,"__claudeZhLabelPatch",{value:!0})})();"#;
        for path in js_files(&assets_dir, true)? {
            let text =
                fs::read_to_string(&path).map_err(|e| format!("读取前端入口文件失败: {e}"))?;
            if text.contains(marker) {
                continue;
            }
            backup.backup_resource(&path)?;
            write_utf8(&path, &(text + patch))?;
        }
        Ok(())
    }

    fn patch_hardcoded_frontend_strings(
        resources_path: &Path,
        pack: &LanguagePack,
        backup: &mut BackupContext,
    ) -> Result<(), String> {
        let replacements: Vec<(String, String)> = serde_json::from_str(pack.frontend_hardcoded)
            .map_err(|e| format!("解析硬编码替换表失败: {e}"))?;
        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        for path in js_files(&assets_dir, false)? {
            let text =
                fs::read_to_string(&path).map_err(|e| format!("读取前端 bundle 失败: {e}"))?;
            let mut updated = repair_hardcoded_identifier_pollution(&text);
            for (source, target) in &replacements {
                updated = replace_hardcoded_frontend_string(&updated, source, target);
            }
            if updated != text {
                backup.backup_resource(&path)?;
                write_utf8(&path, &updated)?;
            }
        }
        Ok(())
    }

    fn replace_hardcoded_frontend_string(text: &str, source: &str, target: &str) -> String {
        if source.is_empty() || !text.contains(source) {
            return text.to_string();
        }

        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        while let Some(relative) = text[cursor..].find(source) {
            let start = cursor + relative;
            let end = start + source.len();
            output.push_str(&text[cursor..start]);
            if hardcoded_match_has_safe_boundaries(text, source, start, end) {
                output.push_str(target);
            } else {
                output.push_str(source);
            }
            cursor = end;
        }
        output.push_str(&text[cursor..]);
        output
    }

    fn repair_hardcoded_identifier_pollution(text: &str) -> String {
        let text = replace_hardcoded_identifier_fragment(text, "来源", "Source");
        replace_hardcoded_identifier_fragment(&text, "扩展", "Extensions")
    }

    fn replace_hardcoded_identifier_fragment(text: &str, source: &str, target: &str) -> String {
        if source.is_empty() || !text.contains(source) {
            return text.to_string();
        }

        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        while let Some(relative) = text[cursor..].find(source) {
            let start = cursor + relative;
            let end = start + source.len();
            output.push_str(&text[cursor..start]);
            if hardcoded_fragment_is_inside_identifier(text, start, end) {
                output.push_str(target);
            } else {
                output.push_str(source);
            }
            cursor = end;
        }
        output.push_str(&text[cursor..]);
        output
    }

    fn hardcoded_fragment_is_inside_identifier(text: &str, start: usize, end: usize) -> bool {
        previous_byte(text.as_bytes(), start)
            .map(is_js_ident_continue)
            .unwrap_or(false)
            || text
                .as_bytes()
                .get(end)
                .copied()
                .map(is_js_ident_continue)
                .unwrap_or(false)
    }

    fn hardcoded_match_has_safe_boundaries(
        text: &str,
        source: &str,
        start: usize,
        end: usize,
    ) -> bool {
        let starts_with_ident = source
            .as_bytes()
            .first()
            .copied()
            .map(is_js_ident_continue)
            .unwrap_or(false);
        let ends_with_ident = source
            .as_bytes()
            .last()
            .copied()
            .map(is_js_ident_continue)
            .unwrap_or(false);

        if starts_with_ident
            && previous_byte(text.as_bytes(), start)
                .map(is_js_ident_continue)
                .unwrap_or(false)
        {
            return false;
        }
        if ends_with_ident
            && text
                .as_bytes()
                .get(end)
                .copied()
                .map(is_js_ident_continue)
                .unwrap_or(false)
        {
            return false;
        }
        true
    }

    fn previous_byte(bytes: &[u8], index: usize) -> Option<u8> {
        index
            .checked_sub(1)
            .and_then(|previous| bytes.get(previous).copied())
    }

    fn is_js_ident_continue(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
    }

    fn patch_hardcoded_main_process_menu_labels(
        resources_path: &Path,
        language: &str,
        backup: &mut BackupContext,
    ) -> Result<(), String> {
        let replacements = match language {
            "zh-CN" => vec![
                ("Enable Main Process Debugger", "启用主进程调试器"),
                ("Record Performance Trace", "记录性能跟踪"),
                ("Write Main Process Heap Snapshot", "写入主进程堆快照"),
                ("Record Memory Trace (auto-stop)", "记录内存跟踪 (自动)"),
            ],
            "zh-TW" | "zh-HK" => vec![
                ("Enable Main Process Debugger", "啟用主行程偵錯器"),
                ("Record Performance Trace", "記錄效能追蹤"),
                ("Write Main Process Heap Snapshot", "寫入主行程堆積快照"),
                ("Record Memory Trace (auto-stop)", "記錄記憶體追蹤 (自動)"),
            ],
            _ => return Err(format!("不支持的语言: {language}")),
        };

        patch_asar_content(resources_path, backup, |content| {
            let text = std::str::from_utf8(content)
                .map_err(|e| format!("app.asar 目标文件不是 UTF-8: {e}"))?;
            let mut patched = text.to_string();
            for (source, target) in replacements {
                if !patched.contains(source) || patched.contains(target) {
                    continue;
                }
                patched = patched.replace(source, &padded_utf8(source, target)?);
            }
            let bytes = patched.into_bytes();
            if bytes == content {
                return Ok(None);
            }
            if bytes.len() != content.len() {
                return Err("主进程菜单汉化改变了 app.asar 文件长度".to_string());
            }
            Ok(Some(bytes))
        })?;
        Ok(())
    }

    fn patch_custom_3p_model_validation(
        resources_path: &Path,
        backup: &mut BackupContext,
    ) -> Result<(), String> {
        patch_asar_content(resources_path, backup, |content| {
            let old_expr = b"process.env.NODE_ENV!==\"production\"";
            let mut new_expr = b"false".to_vec();
            new_expr.resize(old_expr.len(), b' ');

            if let Some((start, end, left, right)) =
                find_custom_3p_validation_toggle(content, old_expr)?
            {
                let mut patched_anchor = Vec::new();
                patched_anchor.extend_from_slice(b"const ");
                patched_anchor.extend_from_slice(&left);
                patched_anchor.extend_from_slice(b"=");
                patched_anchor.extend_from_slice(&new_expr);
                patched_anchor.extend_from_slice(b"||!1,");
                patched_anchor.extend_from_slice(&right);
                patched_anchor.extend_from_slice(b"=");
                if patched_anchor.len() != end - start {
                    return Err("3P 模型校验补丁长度不一致".to_string());
                }
                let mut patched = content.to_vec();
                patched[start..end].copy_from_slice(&patched_anchor);
                return Ok(Some(patched));
            }

            if find_custom_3p_validation_toggle(content, &new_expr)?.is_some()
                || find_custom_3p_name_validator(content, true)?.is_some()
            {
                return Ok(None);
            }

            if let Some((start, end)) = find_custom_3p_name_validator(content, false)? {
                let mut patched = content.to_vec();
                let mut replacement = b"!0".to_vec();
                replacement.resize(end - start, b' ');
                patched[start..end].copy_from_slice(&replacement);
                return Ok(Some(patched));
            }

            if !contains_bytes(
                content,
                b"expected a gateway model route referencing an Anthropic model",
            ) && !contains_bytes(content, b"Bedrock model")
            {
                return Ok(None);
            }

            Err("未能修补 3P 模型名校验，Claude bundle 格式可能已经变化".to_string())
        })?;
        Ok(())
    }

    fn patch_asar_content<F>(
        resources_path: &Path,
        backup: &mut BackupContext,
        patcher: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&[u8]) -> Result<Option<Vec<u8>>, String>,
    {
        let asar_path = resources_path.join("app.asar");
        let mut data = fs::read(&asar_path).map_err(|e| format!("读取 app.asar 失败: {e}"))?;
        let parsed = read_asar_header(&data, &asar_path)?;
        let mut header: Value = serde_json::from_str(&parsed.header_string)
            .map_err(|e| format!("解析 app.asar 头失败: {e}"))?;
        let entry = get_asar_entry_mut(&mut header, ASAR_PATCH_TARGET)?;
        let offset = entry_value_to_usize(entry.get("offset"), "offset")?;
        let size = entry_value_to_usize(entry.get("size"), "size")?;
        let content_offset = 8 + parsed.header_size + offset;
        let content_end = content_offset + size;
        if content_end > data.len() {
            return Err("app.asar 目标文件边界无效".to_string());
        }

        let content = &data[content_offset..content_end];
        let Some(patched_content) = patcher(content)? else {
            sync_claude_exe_asar_integrity(
                resources_path,
                Some(&parsed.header_string),
                Some(backup),
            )?;
            return Ok(());
        };
        if patched_content.len() != content.len() {
            return Err("app.asar 补丁改变了目标文件长度".to_string());
        }

        backup.backup_resource(&asar_path)?;
        data[content_offset..content_end].copy_from_slice(&patched_content);
        entry["integrity"] = asar_file_integrity(&patched_content);
        let header_string =
            serde_json::to_string(&header).map_err(|e| format!("生成 app.asar 头失败: {e}"))?;
        let encoded_header = encode_asar_header(&header_string, parsed.header_size)?;
        data[..encoded_header.len()].copy_from_slice(&encoded_header);
        fs::write(&asar_path, data).map_err(|e| format!("写入 app.asar 失败: {e}"))?;
        sync_claude_exe_asar_integrity(resources_path, Some(&header_string), Some(backup))?;
        Ok(())
    }

    struct AsarHeader {
        header_size: usize,
        header_string: String,
    }

    fn read_asar_header(data: &[u8], path: &Path) -> Result<AsarHeader, String> {
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

    fn encode_asar_header(
        header_string: &str,
        expected_header_size: usize,
    ) -> Result<Vec<u8>, String> {
        let header_bytes = header_string.as_bytes();
        let header_payload_size = align4(4 + header_bytes.len());
        if 4 + header_payload_size != expected_header_size {
            return Err("app.asar 头长度变化，拒绝写入不安全补丁".to_string());
        }
        let mut header_pickle = vec![0u8; expected_header_size];
        header_pickle[0..4].copy_from_slice(&(header_payload_size as u32).to_le_bytes());
        header_pickle[4..8].copy_from_slice(&(header_bytes.len() as i32).to_le_bytes());
        header_pickle[8..8 + header_bytes.len()].copy_from_slice(header_bytes);

        let mut encoded = vec![0u8; 8 + expected_header_size];
        encoded[0..4].copy_from_slice(&4u32.to_le_bytes());
        encoded[4..8].copy_from_slice(&(expected_header_size as u32).to_le_bytes());
        encoded[8..].copy_from_slice(&header_pickle);
        Ok(encoded)
    }

    fn get_asar_entry_mut<'a>(
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
        for key in ["size", "offset", "integrity"] {
            if node.get(key).is_none() {
                return Err(format!("app.asar 目标缺少字段: {key}"));
            }
        }
        Ok(node)
    }

    fn get_asar_entry<'a>(header: &'a Value, file_path: &str) -> Option<&'a Value> {
        let mut node = header;
        for part in file_path.split('/') {
            node = node.get("files")?.get(part)?;
        }
        Some(node)
    }

    fn entry_value_to_usize(value: Option<&Value>, name: &str) -> Result<usize, String> {
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

    fn asar_file_integrity(data: &[u8]) -> Value {
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

    fn sync_claude_exe_asar_integrity(
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

    fn find_custom_3p_validation_toggle(
        content: &[u8],
        expr: &[u8],
    ) -> Result<Option<(usize, usize, Vec<u8>, Vec<u8>)>, String> {
        let pattern = format!(
            r#"const ([A-Za-z_$][A-Za-z0-9_$]*)={}\|\|!1,([A-Za-z_$][A-Za-z0-9_$]*)="#,
            regex::escape(
                std::str::from_utf8(expr).map_err(|e| format!("校验表达式不是 UTF-8: {e}"))?
            )
        );
        let regex =
            BytesRegex::new(&pattern).map_err(|e| format!("创建 3P 校验匹配器失败: {e}"))?;
        let mut found = None;
        for cap in regex.captures_iter(content) {
            let m = cap.get(0).unwrap();
            let flag = cap.get(1).unwrap().as_bytes();
            let window_end = std::cmp::min(content.len(), m.start() + 2500);
            let window = &content[m.start()..window_end];
            let mut return_ok = b"if(!".to_vec();
            return_ok.extend_from_slice(flag);
            return_ok.extend_from_slice(b")return{ok:!0}");
            if contains_bytes(window, &return_ok)
                && contains_bytes(
                    window,
                    b"expected a gateway model route referencing an Anthropic model",
                )
                && contains_bytes(window, b"Bedrock model")
            {
                if found.is_some() {
                    return Err("3P 模型校验匹配到多个位置".to_string());
                }
                found = Some((
                    m.start(),
                    m.end(),
                    cap.get(1).unwrap().as_bytes().to_vec(),
                    cap.get(2).unwrap().as_bytes().to_vec(),
                ));
            }
        }
        Ok(found)
    }

    fn find_custom_3p_name_validator(
        content: &[u8],
        patched: bool,
    ) -> Result<Option<(usize, usize)>, String> {
        let regex = BytesRegex::new(
            r#"function ([A-Za-z_$][A-Za-z0-9_$]*)\(([A-Za-z_$][A-Za-z0-9_$]*)\)\{const ([A-Za-z_$][A-Za-z0-9_$]*)=([A-Za-z_$][A-Za-z0-9_$]*)\.toLowerCase\(\);return ([^{};]+)\}"#,
        )
        .map_err(|e| format!("创建 3P 名称校验匹配器失败: {e}"))?;
        let mut found = None;
        for cap in regex.captures_iter(content) {
            let m = cap.get(0).unwrap();
            if cap.get(2).unwrap().as_bytes() != cap.get(4).unwrap().as_bytes() {
                continue;
            }
            let expr = cap.get(5).unwrap();
            let window_start = m.start().saturating_sub(1500);
            let window_end = std::cmp::min(content.len(), m.start() + 3000);
            let window = &content[window_start..window_end];
            if !contains_bytes(window, b"deepseek")
                || !contains_bytes(
                    window,
                    b"expected a gateway model route referencing an Anthropic model",
                )
            {
                continue;
            }
            let valid = if patched {
                expr.as_bytes()
                    .iter()
                    .copied()
                    .filter(|b| !b.is_ascii_whitespace())
                    .collect::<Vec<_>>()
                    == b"!0"
            } else {
                contains_bytes(expr.as_bytes(), b".test(")
                    && contains_bytes(expr.as_bytes(), b".some(")
                    && contains_bytes(expr.as_bytes(), b".includes(")
            };
            if valid {
                if found.is_some() {
                    return Err("3P 名称校验匹配到多个位置".to_string());
                }
                found = Some((expr.start(), expr.end()));
            }
        }
        Ok(found)
    }

    fn restore_latest_backup(resources_path: &Path) -> Result<(), String> {
        let backup =
            latest_backup(resources_path).ok_or_else(|| "没有可恢复的中文补丁备份".to_string())?;
        let backup_root = backup.clone();
        for path in files_recursive(&backup)? {
            let relative = relative_to(&path, &backup_root)?;
            let target = if relative.starts_with("_app") {
                app_path_from_resources(resources_path).join(relative.strip_prefix("_app").unwrap())
            } else {
                resources_path.join(relative)
            };
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("创建恢复父目录失败: {e}"))?;
            }
            fs::copy(&path, &target).map_err(|e| format!("恢复备份失败: {e}"))?;
        }
        Ok(())
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

    fn remove_language_files(resources_path: &Path) -> Result<(), String> {
        for lang in LANGS {
            for path in [
                resources_path
                    .join("ion-dist")
                    .join("i18n")
                    .join(format!("{lang}.json")),
                resources_path.join(format!("{lang}.json")),
                resources_path
                    .join("ion-dist")
                    .join("i18n")
                    .join("statsig")
                    .join(format!("{lang}.json")),
            ] {
                match fs::remove_file(&path) {
                    Ok(()) => {}
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                    Err(e) => return Err(format!("删除中文资源失败: {e}")),
                }
            }
        }
        Ok(())
    }

    fn unregister_languages(resources_path: &Path) -> Result<(), String> {
        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        for path in js_files(&assets_dir, true)? {
            let text =
                fs::read_to_string(&path).map_err(|e| format!("读取前端入口文件失败: {e}"))?;
            let mut updated = text.clone();
            for lang in LANGS {
                updated = updated.replace(&format!(r#","{lang}""#), "");
            }
            if updated != text {
                write_utf8(&path, &updated)?;
            }
        }
        Ok(())
    }

    fn set_claude_locale(locale: &str) -> Result<(), String> {
        let paths = claude_config_paths();
        if paths.is_empty() {
            return Err("未找到 Claude 用户配置目录".to_string());
        }
        for path in paths {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("创建 Claude 配置目录失败: {e}"))?;
            }
            let mut data = if path.is_file() {
                let text =
                    fs::read_to_string(&path).map_err(|e| format!("读取 Claude 配置失败: {e}"))?;
                serde_json::from_str::<Value>(&text).unwrap_or_else(|_| Value::Object(Map::new()))
            } else {
                Value::Object(Map::new())
            };
            let object = data
                .as_object_mut()
                .ok_or_else(|| "Claude 配置不是 JSON 对象".to_string())?;
            object.insert("locale".to_string(), Value::String(locale.to_string()));
            fs::write(
                &path,
                serde_json::to_string_pretty(&data)
                    .map_err(|e| format!("生成 Claude 配置失败: {e}"))?,
            )
            .map_err(|e| format!("写入 Claude 配置失败: {e}"))?;
        }
        Ok(())
    }

    fn read_current_locale() -> Option<String> {
        for path in claude_config_paths() {
            let text = fs::read_to_string(path).ok()?;
            let value: Value = serde_json::from_str(&text).ok()?;
            if let Some(locale) = value.get("locale").and_then(Value::as_str) {
                return Some(locale.to_string());
            }
        }
        None
    }

    fn claude_config_paths() -> Vec<PathBuf> {
        let Some(local) = env::var_os("LOCALAPPDATA").map(PathBuf::from) else {
            return Vec::new();
        };
        let mut packages = Vec::new();
        let package_root = local.join("Packages");
        if let Ok(entries) = fs::read_dir(&package_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if name.starts_with("Claude_") {
                    packages.push(name.to_string());
                }
            }
        }
        if packages.is_empty() {
            packages.push("Claude_pzs8sxrjxfjjc".to_string());
        }

        let mut paths = Vec::new();
        for package in packages {
            let root = package_root
                .join(package)
                .join("LocalCache")
                .join("Roaming");
            paths.push(root.join("Claude").join("config.json"));
            paths.push(root.join("Claude-3p").join("config.json"));
        }
        paths
    }

    fn js_files(assets_dir: &Path, index_only: bool) -> Result<Vec<PathBuf>, String> {
        let entries = fs::read_dir(assets_dir).map_err(|e| format!("读取前端资源目录失败: {e}"))?;
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

    fn files_recursive(root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();
        if !root.is_dir() {
            return Ok(files);
        }
        for entry in fs::read_dir(root).map_err(|e| format!("读取目录失败: {e}"))? {
            let path = entry.map_err(|e| format!("读取目录项失败: {e}"))?.path();
            if path.is_dir() {
                files.extend(files_recursive(&path)?);
            } else {
                files.push(path);
            }
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
        resources_path.join(".zh-cn-backups")
    }

    fn relative_to(path: &Path, root: &Path) -> Result<PathBuf, String> {
        let full = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        full.strip_prefix(&root)
            .map(Path::to_path_buf)
            .map_err(|_| format!("路径不在预期目录内: {}", path.display()))
    }

    fn write_utf8(path: &Path, text: &str) -> Result<(), String> {
        fs::write(path, text.as_bytes())
            .map_err(|e| format!("写入文件失败 {}: {e}", path.display()))
    }

    fn padded_utf8(source: &str, target: &str) -> Result<String, String> {
        let source_len = source.len();
        let target_len = target.len();
        if target_len > source_len {
            return Err(format!("替换文本比原文更长: {source}"));
        }
        Ok(format!("{target}{}", " ".repeat(source_len - target_len)))
    }

    fn align4(value: usize) -> usize {
        value + ((4 - (value % 4)) % 4)
    }

    fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, String> {
        let slice = bytes
            .get(offset..offset + 4)
            .ok_or_else(|| "读取 u32 越界".to_string())?;
        Ok(u32::from_le_bytes(slice.try_into().unwrap()))
    }

    fn read_i32_le(bytes: &[u8], offset: usize) -> Result<i32, String> {
        let slice = bytes
            .get(offset..offset + 4)
            .ok_or_else(|| "读取 i32 越界".to_string())?;
        Ok(i32::from_le_bytes(slice.try_into().unwrap()))
    }

    fn sha256_hex(data: &[u8]) -> String {
        let digest = Sha256::digest(data);
        let mut out = String::with_capacity(64);
        for byte in digest {
            out.push_str(&format!("{byte:02x}"));
        }
        out
    }

    fn find_pattern(data: &[u8], pattern: &[u8]) -> Vec<usize> {
        if pattern.is_empty() || data.len() < pattern.len() {
            return Vec::new();
        }
        data.windows(pattern.len())
            .enumerate()
            .filter_map(|(index, window)| (window == pattern).then_some(index))
            .collect()
    }

    fn contains_bytes(data: &[u8], needle: &[u8]) -> bool {
        !needle.is_empty() && data.windows(needle.len()).any(|window| window == needle)
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

    #[cfg(test)]
    mod tests {
        use super::{repair_hardcoded_identifier_pollution, replace_hardcoded_frontend_string};

        #[test]
        fn hardcoded_frontend_replacements_skip_js_identifiers() {
            let text = r#"const shaderSource=e.shaderSource;function supportedExtensions(){return true}setSourceBranch("main");"#;
            let text = replace_hardcoded_frontend_string(text, "Source", "来源");
            let text = replace_hardcoded_frontend_string(&text, "Extensions", "扩展");

            assert!(text.contains("shaderSource"));
            assert!(text.contains("supportedExtensions"));
            assert!(text.contains("setSourceBranch"));
            assert!(!text.contains("shader来源"));
            assert!(!text.contains("supported扩展"));
            assert!(!text.contains("set来源Branch"));
        }

        #[test]
        fn hardcoded_frontend_replacements_keep_literal_copy() {
            let text =
                r#"{title:"Source",label:"Extensions",description:"Show extension directory"}"#;
            let text = replace_hardcoded_frontend_string(text, "Source", "来源");
            let text = replace_hardcoded_frontend_string(&text, "Extensions", "扩展");
            let text = replace_hardcoded_frontend_string(
                &text,
                "Show extension directory",
                "显示扩展目录",
            );

            assert!(text.contains(r#"title:"来源""#));
            assert!(text.contains(r#"label:"扩展""#));
            assert!(text.contains(r#"description:"显示扩展目录""#));
        }

        #[test]
        fn hardcoded_frontend_repair_restores_polluted_identifiers_only() {
            let text =
                r#"shader来源 supported扩展 set来源Branch trust来源s title:"来源" label:"扩展""#;
            let text = repair_hardcoded_identifier_pollution(text);

            assert!(text.contains("shaderSource"));
            assert!(text.contains("supportedExtensions"));
            assert!(text.contains("setSourceBranch"));
            assert!(text.contains("trustSources"));
            assert!(text.contains(r#"title:"来源""#));
            assert!(text.contains(r#"label:"扩展""#));
            assert!(!text.contains("shader来源"));
            assert!(!text.contains("supported扩展"));
        }

        #[test]
        #[ignore = "writes Claude Desktop resources; set CLAUDE_PLUS_VERIFY_INSTALL=1"]
        fn verify_install_zh_cn_keeps_cowork_identifiers() {
            if std::env::var("CLAUDE_PLUS_VERIFY_INSTALL").ok().as_deref() != Some("1") {
                eprintln!("skipping install verification; set CLAUDE_PLUS_VERIFY_INSTALL=1");
                return;
            }

            super::install("zh-CN", false).expect("install zh-CN localization");
            let paths = super::resolve_claude_paths().expect("Claude Desktop paths");
            let assets_dir = paths.resources.join("ion-dist").join("assets").join("v1");
            let mut all_text = String::new();
            for path in super::js_files(&assets_dir, false).expect("frontend JS files") {
                all_text.push_str(
                    &std::fs::read_to_string(&path).expect("installed frontend JS should be UTF-8"),
                );
                all_text.push('\n');
            }

            assert!(all_text.contains("shaderSource"));
            assert!(all_text.contains("supportedExtensions"));
            assert!(!all_text.contains("shader来源"));
            assert!(!all_text.contains("supported扩展"));
            assert!(!all_text.contains("set来源Branch"));
            assert!(!all_text.contains("trust来源s"));
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct ClaudeZhStatus {
        pub supported: bool,
        pub claude_found: bool,
        pub installed: bool,
        pub backup_available: bool,
        pub claude_version: Option<String>,
        pub install_path: Option<String>,
        pub resources_path: Option<String>,
        pub locale: Option<String>,
        pub language_files: Vec<String>,
    }

    pub fn status() -> ClaudeZhStatus {
        ClaudeZhStatus {
            supported: false,
            claude_found: false,
            installed: false,
            backup_available: false,
            claude_version: None,
            install_path: None,
            resources_path: None,
            locale: None,
            language_files: Vec::new(),
        }
    }

    pub fn install(_language: &str, _skip_asar_patch: bool) -> Result<(), String> {
        Err("当前只支持 Windows Claude Desktop 汉化".to_string())
    }

    pub fn backup() -> Result<(), String> {
        Err("当前只支持 Windows Claude Desktop 汉化".to_string())
    }

    pub fn uninstall() -> Result<(), String> {
        Err("当前只支持 Windows Claude Desktop 汉化".to_string())
    }
}

pub use imp::{backup, install, status, uninstall, ClaudeZhStatus};
