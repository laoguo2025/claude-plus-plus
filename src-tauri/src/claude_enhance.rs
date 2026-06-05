#[cfg(target_os = "windows")]
mod enhance_asar;
#[cfg(target_os = "windows")]
mod enhance_bridge_ops;
#[cfg(target_os = "windows")]
mod enhance_bridge_scripts;
#[cfg(target_os = "windows")]
mod enhance_injected;

#[cfg(target_os = "windows")]
pub(crate) use enhance_injected::EnhanceScriptLocale;

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const SCRIPT_MARKER: &str = "__claudePlusEnhanceNavV2";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const NAV_API_MARKER: &str = "__claudePlusEnhanceThirdPartyApiV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const NAV_PLUGINS_MARKER: &str = "__claudePlusEnhancePluginsV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const NAV_MCP_MARKER: &str = "__claudePlusEnhanceMcpV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const CONVERSATION_TITLE_I18N_MARKER: &str = "__claudePlusEnhanceConversationTitleI18nV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const MARKDOWN_EXPORT_MARKER: &str = "__claudePlusEnhanceMarkdownExportV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const TIMELINE_MARKER: &str = "__claudePlusEnhanceTimelineV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const TOKEN_USAGE_MARKER: &str = "__claudePlusEnhanceTokenUsageV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const SKILLS_BRIDGE_MARKER: &str = "__claudePlusSkillsBridgeV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const SKILLS_MAIN_BRIDGE_MARKER: &str = "__claudePlusSkillsMainBridgeV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const SKILLS_MAIN_BRIDGE_TARGET: &str = ".vite/build/index.js";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const SKILLS_PRELOAD_BRIDGE_TARGET: &str = ".vite/build/mainView.js";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const SKILLS_LIST_CHANNEL: &str = "claude-plus:skills:list";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const SKILLS_TRASH_CHANNEL: &str = "claude-plus:skills:trash";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const TITLE_I18N_BRIDGE_MARKER: &str = "__claudePlusTitleI18nBridgeV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const TITLE_I18N_MAIN_BRIDGE_MARKER: &str = "__claudePlusTitleI18nMainBridgeV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const TITLE_I18N_CHANNEL: &str = "claude-plus:title-i18n";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const TOKEN_USAGE_BRIDGE_MARKER: &str = "__claudePlusTokenUsageBridgeV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const TOKEN_USAGE_MAIN_BRIDGE_MARKER: &str = "__claudePlusTokenUsageMainBridgeV1";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const TOKEN_USAGE_CHANNEL: &str = "claude-plus:token-usage";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const BACKUP_DIR_NAME: &str = ".claude-plus-enhance-backups";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const ENHANCE_FEATURES_JSON: &str = include_str!("../../src/shared/enhance-features.json");

#[cfg(target_os = "windows")]
mod imp {
    use super::{
        enhance_bridge_ops::{
            skills_bridge_installed, title_i18n_bridge_installed, token_usage_bridge_installed,
            update_skills_bridge, update_title_i18n_bridge, update_token_usage_bridge,
        },
        enhance_injected::{current_script_locale, inject_script_for_locale},
        EnhanceScriptLocale, BACKUP_DIR_NAME, CONVERSATION_TITLE_I18N_MARKER,
        ENHANCE_FEATURES_JSON, MARKDOWN_EXPORT_MARKER, NAV_API_MARKER, NAV_MCP_MARKER,
        NAV_PLUGINS_MARKER, SCRIPT_MARKER, TIMELINE_MARKER, TOKEN_USAGE_MARKER,
    };
    use crate::claude_desktop;
    use crate::claude_patch_common as patch;
    use serde::{Deserialize, Serialize};
    use std::{fs, path::Path};

    #[cfg(test)]
    pub(super) use super::{
        enhance_asar::{patch_bridge_files_for_test, read_asar_file},
        enhance_bridge_ops::remove_skills_bridge,
        enhance_bridge_scripts::{
            skills_bridge_script, skills_main_bridge_script, title_i18n_main_bridge_script,
            title_i18n_preload_bridge_script, token_usage_main_bridge_script,
            token_usage_preload_bridge_script,
        },
        enhance_injected::inject_script_for_locale_with_tuning,
        SKILLS_LIST_CHANNEL, SKILLS_MAIN_BRIDGE_TARGET, SKILLS_PRELOAD_BRIDGE_TARGET,
        SKILLS_TRASH_CHANNEL, TOKEN_USAGE_MAIN_BRIDGE_MARKER,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum EnhanceFeatureId {
        ThirdPartyApi,
        Plugins,
        Mcp,
        ConversationTitleI18n,
        Markdown,
        Timeline,
        TokenUsage,
    }

    impl EnhanceFeatureId {
        fn parse(value: &str) -> Option<Self> {
            match value {
                "third_party_api" => Some(Self::ThirdPartyApi),
                "plugins" => Some(Self::Plugins),
                "mcp" => Some(Self::Mcp),
                "conversation_title_i18n" => Some(Self::ConversationTitleI18n),
                "markdown" => Some(Self::Markdown),
                "timeline" => Some(Self::Timeline),
                "token_usage" => Some(Self::TokenUsage),
                _ => None,
            }
        }

        fn marker(self) -> &'static str {
            match self {
                Self::ThirdPartyApi => NAV_API_MARKER,
                Self::Plugins => NAV_PLUGINS_MARKER,
                Self::Mcp => NAV_MCP_MARKER,
                Self::ConversationTitleI18n => CONVERSATION_TITLE_I18N_MARKER,
                Self::Markdown => MARKDOWN_EXPORT_MARKER,
                Self::Timeline => TIMELINE_MARKER,
                Self::TokenUsage => TOKEN_USAGE_MARKER,
            }
        }

        fn id(self) -> &'static str {
            match self {
                Self::ThirdPartyApi => "third_party_api",
                Self::Plugins => "plugins",
                Self::Mcp => "mcp",
                Self::ConversationTitleI18n => "conversation_title_i18n",
                Self::Markdown => "markdown",
                Self::Timeline => "timeline",
                Self::TokenUsage => "token_usage",
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
        pub id: String,
        pub category: String,
        pub label: String,
        pub version: String,
        pub description: String,
        pub enabled: bool,
        pub available: bool,
        pub note: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct FeatureState {
        feature: EnhanceFeatureId,
        enabled: bool,
        installed_version: Option<String>,
        current_version: String,
    }

    impl FeatureState {
        #[cfg(test)]
        fn needs_upgrade(&self) -> bool {
            self.enabled && self.installed_version.as_deref() != Some(self.current_version.as_str())
        }
    }

    #[derive(Deserialize)]
    struct EnhanceFeatureDefinition {
        id: String,
        category: String,
        label: String,
        version: String,
        description: String,
        available: bool,
        note: String,
    }

    pub fn status() -> ClaudeEnhanceStatus {
        status_from_paths(patch::resolve_claude_paths().ok())
    }

    fn status_from_paths(paths: Option<patch::ClaudePaths>) -> ClaudeEnhanceStatus {
        let resources_path = paths.as_ref().map(|p| p.resources.clone());
        let enabled = resources_path
            .as_ref()
            .map(|path| feature_states(path))
            .unwrap_or_default();
        let installed = enabled.iter().any(|state| state.enabled);

        ClaudeEnhanceStatus {
            supported: true,
            claude_found: paths.is_some(),
            installed,
            backup_available: resources_path
                .as_ref()
                .map(|path| patch::latest_backup(path, BACKUP_DIR_NAME).is_some())
                .unwrap_or(false),
            install_path: paths.as_ref().map(|p| p.app.display().to_string()),
            resources_path: resources_path.as_ref().map(|p| p.display().to_string()),
            features: feature_list(&enabled),
        }
    }

    pub fn install(feature: &str) -> Result<(), String> {
        let feature =
            EnhanceFeatureId::parse(feature).ok_or_else(|| format!("未知页面增强项: {feature}"))?;
        let paths = patch::resolve_claude_paths()?;
        claude_desktop::stop_claude_processes()?;
        patch::enable_write_access(&paths.resources, false);

        let mut backup = patch::BackupContext::new(&paths.resources, BACKUP_DIR_NAME);
        update_feature_marker(
            &paths.resources,
            &mut backup,
            feature,
            true,
            current_script_locale(),
        )?;
        if matches!(feature, EnhanceFeatureId::Plugins) {
            update_skills_bridge(&paths.resources, &mut backup, true, current_script_locale())?;
        }
        if matches!(feature, EnhanceFeatureId::ConversationTitleI18n) {
            update_title_i18n_bridge(&paths.resources, &mut backup, true)?;
        }
        if matches!(feature, EnhanceFeatureId::TokenUsage) {
            update_token_usage_bridge(&paths.resources, &mut backup, true)?;
        }
        Ok(())
    }

    pub fn uninstall(feature: &str) -> Result<(), String> {
        let feature =
            EnhanceFeatureId::parse(feature).ok_or_else(|| format!("未知页面增强项: {feature}"))?;
        let paths = patch::resolve_claude_paths()?;
        claude_desktop::stop_claude_processes()?;
        patch::enable_write_access(&paths.resources, false);
        let mut backup = patch::BackupContext::new(&paths.resources, BACKUP_DIR_NAME);
        update_feature_marker(
            &paths.resources,
            &mut backup,
            feature,
            false,
            current_script_locale(),
        )?;
        if matches!(feature, EnhanceFeatureId::Plugins) {
            update_skills_bridge(
                &paths.resources,
                &mut backup,
                false,
                current_script_locale(),
            )?;
        }
        if matches!(feature, EnhanceFeatureId::ConversationTitleI18n) {
            update_title_i18n_bridge(&paths.resources, &mut backup, false)?;
        }
        if matches!(feature, EnhanceFeatureId::TokenUsage) {
            update_token_usage_bridge(&paths.resources, &mut backup, false)?;
        }
        Ok(())
    }

    fn feature_list(enabled: &[FeatureState]) -> Vec<EnhanceFeature> {
        feature_definitions_from_json(ENHANCE_FEATURES_JSON)
            .into_iter()
            .filter_map(|definition| {
                let feature = EnhanceFeatureId::parse(&definition.id)?;
                Some(EnhanceFeature {
                    id: definition.id,
                    category: definition.category,
                    label: definition.label,
                    version: definition.version,
                    description: definition.description,
                    enabled: is_enabled(enabled, feature),
                    available: definition.available,
                    note: definition.note,
                })
            })
            .collect()
    }

    fn feature_definitions_from_json(text: &str) -> Vec<EnhanceFeatureDefinition> {
        match serde_json::from_str(text) {
            Ok(definitions) => definitions,
            Err(error) => {
                tracing::error!("embedded enhance feature definitions invalid: {error}");
                Vec::new()
            }
        }
    }

    fn is_enabled(enabled: &[FeatureState], feature: EnhanceFeatureId) -> bool {
        enabled
            .iter()
            .find_map(|state| (state.feature == feature).then_some(state.enabled))
            .unwrap_or(false)
    }

    fn feature_states(resources_path: &Path) -> Vec<FeatureState> {
        let text = read_index_bundle(resources_path).unwrap_or_default();
        let mut states = feature_states_from_text(&text);
        if let Some(state) = states
            .iter_mut()
            .find(|state| state.feature == EnhanceFeatureId::Plugins)
        {
            state.enabled = state.enabled && skills_bridge_installed(resources_path);
        }
        if let Some(state) = states
            .iter_mut()
            .find(|state| state.feature == EnhanceFeatureId::ConversationTitleI18n)
        {
            state.enabled = state.enabled && title_i18n_bridge_installed(resources_path);
        }
        if let Some(state) = states
            .iter_mut()
            .find(|state| state.feature == EnhanceFeatureId::TokenUsage)
        {
            state.enabled = state.enabled && token_usage_bridge_installed(resources_path);
        }
        states
    }

    fn feature_states_from_text(text: &str) -> Vec<FeatureState> {
        [
            EnhanceFeatureId::ThirdPartyApi,
            EnhanceFeatureId::Plugins,
            EnhanceFeatureId::Mcp,
            EnhanceFeatureId::ConversationTitleI18n,
            EnhanceFeatureId::Markdown,
            EnhanceFeatureId::Timeline,
            EnhanceFeatureId::TokenUsage,
        ]
        .into_iter()
        .map(|feature| {
            let current_version = feature_version(feature);
            let installed_version = feature_payload_version(text, feature.marker());
            let enabled = installed_version.is_some()
                || text.contains(&legacy_feature_payload(feature.marker()));
            FeatureState {
                feature,
                enabled,
                installed_version,
                current_version,
            }
        })
        .collect()
    }

    #[cfg(test)]
    fn apply_feature_version_upgrades(
        resources_path: &Path,
        needs_upgrade: &[EnhanceFeatureId],
    ) -> Result<(), String> {
        let mut backup = patch::BackupContext::new(resources_path, BACKUP_DIR_NAME);
        for feature in needs_upgrade {
            update_feature_marker(
                resources_path,
                &mut backup,
                *feature,
                true,
                current_script_locale(),
            )?;
            if matches!(feature, EnhanceFeatureId::Plugins) {
                update_skills_bridge(resources_path, &mut backup, true, current_script_locale())?;
            }
            if matches!(feature, EnhanceFeatureId::ConversationTitleI18n) {
                update_title_i18n_bridge(resources_path, &mut backup, true)?;
            }
            if matches!(feature, EnhanceFeatureId::TokenUsage) {
                update_token_usage_bridge(resources_path, &mut backup, true)?;
            }
        }
        Ok(())
    }

    pub(crate) fn refresh_enabled_features_for_locale(
        resources_path: &Path,
        locale: EnhanceScriptLocale,
    ) -> Result<(), String> {
        let states =
            feature_states_from_text(&read_index_bundle(resources_path).unwrap_or_default());
        if !states.iter().any(|state| state.enabled) {
            return Ok(());
        }

        let mut backup = patch::BackupContext::new(resources_path, BACKUP_DIR_NAME);
        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        let mut patched = false;
        for path in patch::js_files(&assets_dir, true)? {
            let text =
                fs::read_to_string(&path).map_err(|e| format!("读取 Claude 前端入口失败: {e}"))?;
            let next = ensure_or_remove_script(remove_old_script(&text), locale);
            if next != text {
                backup.backup_resource(&path)?;
                patch::atomic_write(&path, next.as_bytes())
                    .map_err(|e| format!("写入 Claude 页面增强入口失败: {e}"))?;
            }
            patched = true;
        }

        if patched {
            if is_enabled(&states, EnhanceFeatureId::Plugins) {
                let mut backup = patch::BackupContext::new(resources_path, BACKUP_DIR_NAME);
                update_skills_bridge(resources_path, &mut backup, true, locale)?;
            }
            Ok(())
        } else {
            Err("未找到可刷新语言的 Claude 前端入口".to_string())
        }
    }

    fn update_feature_marker(
        resources_path: &Path,
        backup: &mut patch::BackupContext,
        feature: EnhanceFeatureId,
        enabled: bool,
        locale: EnhanceScriptLocale,
    ) -> Result<(), String> {
        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        let mut patched = false;
        for path in patch::js_files(&assets_dir, true)? {
            let text =
                fs::read_to_string(&path).map_err(|e| format!("读取 Claude 前端入口失败: {e}"))?;
            let mut next = remove_old_script(&text);
            next = set_marker(next, feature.marker(), enabled);
            next = ensure_or_remove_script(next, locale);
            if next == text {
                patched = true;
                continue;
            }
            backup.backup_resource(&path)?;
            patch::atomic_write(&path, next.as_bytes())
                .map_err(|e| format!("写入 Claude 页面增强入口失败: {e}"))?;
            patched = true;
        }

        if patched {
            Ok(())
        } else {
            Err("未找到可注入的 Claude 前端入口".to_string())
        }
    }

    fn ensure_or_remove_script(mut text: String, locale: EnhanceScriptLocale) -> String {
        let has_enabled_feature = feature_states_from_text(&text)
            .into_iter()
            .any(|state| state.enabled);
        if has_enabled_feature && !text.contains(SCRIPT_MARKER) {
            text.push_str(&inject_script_for_locale(locale));
        }
        if !has_enabled_feature {
            text = remove_old_script(&text);
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
        text = remove_feature_payloads(&text, marker);
        if enabled {
            text.push_str(&feature_payload(marker));
        }
        text
    }

    fn feature_payload(marker: &str) -> String {
        format!(
            r#";window.{marker}={{version:"{}"}};"#,
            feature_version_by_marker(marker)
        )
    }

    fn legacy_feature_payload(marker: &str) -> String {
        format!(";window.{marker}=true;")
    }

    fn remove_feature_payloads(text: &str, marker: &str) -> String {
        let prefix = format!(";window.{marker}=");
        let mut next = text.to_string();
        while let Some(start) = next.find(&prefix) {
            let Some(relative_end) = next[start + prefix.len()..].find(';') else {
                break;
            };
            let end = start + prefix.len() + relative_end + 1;
            next.replace_range(start..end, "");
        }
        next
    }

    fn feature_payload_version(text: &str, marker: &str) -> Option<String> {
        let prefix = format!(";window.{marker}={{version:\"");
        let start = text.find(&prefix)? + prefix.len();
        let end = text[start..].find('"')?;
        Some(text[start..start + end].to_string())
    }

    fn feature_version_by_marker(marker: &str) -> String {
        [
            EnhanceFeatureId::ThirdPartyApi,
            EnhanceFeatureId::Plugins,
            EnhanceFeatureId::Mcp,
            EnhanceFeatureId::ConversationTitleI18n,
            EnhanceFeatureId::Markdown,
            EnhanceFeatureId::Timeline,
            EnhanceFeatureId::TokenUsage,
        ]
        .into_iter()
        .find(|feature| feature.marker() == marker)
        .map(feature_version)
        .unwrap_or_else(|| "v0.2".to_string())
    }

    fn feature_version(feature: EnhanceFeatureId) -> String {
        let id = feature.id();
        feature_definitions_from_json(ENHANCE_FEATURES_JSON)
            .into_iter()
            .find_map(|definition| (definition.id == id).then_some(definition.version))
            .unwrap_or_else(|| "v0.2".to_string())
    }

    #[cfg(test)]
    mod enhance_tests;

    fn read_index_bundle(resources_path: &Path) -> Result<String, String> {
        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        let mut output = String::new();
        for path in patch::js_files(&assets_dir, true)? {
            let text =
                fs::read_to_string(&path).map_err(|e| format!("读取 Claude 前端入口失败: {e}"))?;
            output.push_str(&text);
        }
        Ok(output)
    }
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
        pub id: String,
        pub label: String,
        pub category: String,
        pub version: String,
        pub description: String,
        pub enabled: bool,
        pub available: bool,
        pub note: String,
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

    pub fn install(_feature: &str) -> Result<(), String> {
        Err("当前只支持在 Windows 上安装 Claude Desktop 页面增强".to_string())
    }

    pub fn uninstall(_feature: &str) -> Result<(), String> {
        Err("当前只支持在 Windows 上恢复 Claude Desktop 页面增强".to_string())
    }
}

pub use imp::*;
