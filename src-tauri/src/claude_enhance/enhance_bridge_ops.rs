use crate::claude_patch_common as patch;
use std::path::Path;

use super::{
    enhance_asar::{patch_bridge_files, read_asar_file, BridgePatch},
    enhance_bridge_scripts::{
        skills_bridge_script, skills_main_bridge_script, title_i18n_main_bridge_script,
        title_i18n_preload_bridge_script, token_usage_main_bridge_script,
        token_usage_preload_bridge_script,
    },
    enhance_injected::EnhanceScriptLocale,
    SKILLS_BRIDGE_MARKER, SKILLS_MAIN_BRIDGE_MARKER, SKILLS_MAIN_BRIDGE_TARGET,
    SKILLS_PRELOAD_BRIDGE_TARGET, TITLE_I18N_BRIDGE_MARKER, TITLE_I18N_MAIN_BRIDGE_MARKER,
    TOKEN_USAGE_BRIDGE_MARKER, TOKEN_USAGE_MAIN_BRIDGE_MARKER,
};

pub(crate) fn update_skills_bridge(
    resources_path: &Path,
    backup: &mut patch::BackupContext,
    enabled: bool,
    locale: EnhanceScriptLocale,
) -> Result<(), String> {
    let main_script = skills_main_bridge_script(locale);
    let preload_script = skills_bridge_script();
    patch_bridge_files(
        resources_path,
        &[
            BridgePatch {
                file_path: SKILLS_MAIN_BRIDGE_TARGET,
                script: &main_script,
                remover: remove_skills_bridge,
            },
            BridgePatch {
                file_path: SKILLS_PRELOAD_BRIDGE_TARGET,
                script: &preload_script,
                remover: remove_skills_bridge,
            },
        ],
        backup,
        enabled,
    )
}

pub(crate) fn update_title_i18n_bridge(
    resources_path: &Path,
    backup: &mut patch::BackupContext,
    enabled: bool,
) -> Result<(), String> {
    let main_script = title_i18n_main_bridge_script();
    let preload_script = title_i18n_preload_bridge_script();
    patch_bridge_files(
        resources_path,
        &[
            BridgePatch {
                file_path: SKILLS_MAIN_BRIDGE_TARGET,
                script: &main_script,
                remover: remove_title_i18n_bridge,
            },
            BridgePatch {
                file_path: SKILLS_PRELOAD_BRIDGE_TARGET,
                script: &preload_script,
                remover: remove_title_i18n_bridge,
            },
        ],
        backup,
        enabled,
    )
}

pub(crate) fn update_token_usage_bridge(
    resources_path: &Path,
    backup: &mut patch::BackupContext,
    enabled: bool,
) -> Result<(), String> {
    let main_script = token_usage_main_bridge_script();
    let preload_script = token_usage_preload_bridge_script();
    patch_bridge_files(
        resources_path,
        &[
            BridgePatch {
                file_path: SKILLS_MAIN_BRIDGE_TARGET,
                script: &main_script,
                remover: remove_token_usage_bridge,
            },
            BridgePatch {
                file_path: SKILLS_PRELOAD_BRIDGE_TARGET,
                script: &preload_script,
                remover: remove_token_usage_bridge,
            },
        ],
        backup,
        enabled,
    )
}

pub(crate) fn remove_skills_bridge(text: &str) -> String {
    let mut next = text.to_string();
    for marker in [
        ";(()=>{const MARK=\"__claudePlusSkillsBridgeV1\"",
        ";(()=>{const MARK=\"__claudePlusSkillsMainBridgeV1\"",
    ] {
        while let Some(start) = next.find(marker) {
            let Some(relative_end) = next[start..].find("})();") else {
                break;
            };
            let end = start + relative_end + "})();".len();
            next.replace_range(start..end, "");
        }
    }
    next
}

pub(crate) fn remove_title_i18n_bridge(text: &str) -> String {
    let mut next = text.to_string();
    for marker in [
        ";(()=>{const MARK=\"__claudePlusTitleI18nBridgeV1\"",
        ";(()=>{const MARK=\"__claudePlusTitleI18nMainBridgeV1\"",
    ] {
        while let Some(start) = next.find(marker) {
            let Some(relative_end) = next[start..].find("})();") else {
                break;
            };
            let end = start + relative_end + "})();".len();
            next.replace_range(start..end, "");
        }
    }
    next
}

pub(crate) fn remove_token_usage_bridge(text: &str) -> String {
    let mut next = text.to_string();
    for marker in [
        ";(()=>{const MARK=\"__claudePlusTokenUsageBridgeV1\"",
        ";(()=>{const MARK=\"__claudePlusTokenUsageMainBridgeV1\"",
    ] {
        while let Some(start) = next.find(marker) {
            let Some(relative_end) = next[start..].find("})();") else {
                break;
            };
            let end = start + relative_end + "})();".len();
            next.replace_range(start..end, "");
        }
    }
    next
}

pub(crate) fn skills_bridge_installed(resources_path: &Path) -> bool {
    let preload_installed = read_asar_file(resources_path, SKILLS_PRELOAD_BRIDGE_TARGET)
        .ok()
        .and_then(|content| String::from_utf8(content).ok())
        .map(|text| text.contains(SKILLS_BRIDGE_MARKER))
        .unwrap_or(false);
    let main_installed = read_asar_file(resources_path, SKILLS_MAIN_BRIDGE_TARGET)
        .ok()
        .and_then(|content| String::from_utf8(content).ok())
        .map(|text| text.contains(SKILLS_MAIN_BRIDGE_MARKER))
        .unwrap_or(false);
    preload_installed && main_installed
}

pub(crate) fn title_i18n_bridge_installed(resources_path: &Path) -> bool {
    let preload_installed = read_asar_file(resources_path, SKILLS_PRELOAD_BRIDGE_TARGET)
        .ok()
        .and_then(|content| String::from_utf8(content).ok())
        .map(|text| text.contains(TITLE_I18N_BRIDGE_MARKER))
        .unwrap_or(false);
    let main_installed = read_asar_file(resources_path, SKILLS_MAIN_BRIDGE_TARGET)
        .ok()
        .and_then(|content| String::from_utf8(content).ok())
        .map(|text| text.contains(TITLE_I18N_MAIN_BRIDGE_MARKER))
        .unwrap_or(false);
    preload_installed && main_installed
}

pub(crate) fn token_usage_bridge_installed(resources_path: &Path) -> bool {
    let preload_installed = read_asar_file(resources_path, SKILLS_PRELOAD_BRIDGE_TARGET)
        .ok()
        .and_then(|content| String::from_utf8(content).ok())
        .map(|text| text.contains(TOKEN_USAGE_BRIDGE_MARKER))
        .unwrap_or(false);
    let main_installed = read_asar_file(resources_path, SKILLS_MAIN_BRIDGE_TARGET)
        .ok()
        .and_then(|content| String::from_utf8(content).ok())
        .map(|text| text.contains(TOKEN_USAGE_MAIN_BRIDGE_MARKER))
        .unwrap_or(false);
    preload_installed && main_installed
}
