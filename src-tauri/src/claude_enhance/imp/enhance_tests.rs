use crate::claude_patch_common::ClaudePaths;
use std::{
    fs,
    path::PathBuf,
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    feature_payload, feature_states_from_text, remove_skills_bridge, EnhanceFeatureId,
    CONVERSATION_TITLE_I18N_MARKER, MARKDOWN_EXPORT_MARKER, NAV_API_MARKER, NAV_MCP_MARKER,
    NAV_PLUGINS_MARKER, SCRIPT_MARKER, SKILLS_LIST_CHANNEL, SKILLS_MAIN_BRIDGE_TARGET,
    SKILLS_PRELOAD_BRIDGE_TARGET, SKILLS_TRASH_CHANNEL, TIMELINE_MARKER,
    TOKEN_USAGE_MAIN_BRIDGE_MARKER, TOKEN_USAGE_MARKER,
};

static INJECT_SCRIPT: LazyLock<String> =
    LazyLock::new(|| super::inject_script_for_locale(super::EnhanceScriptLocale::ZhCn));

fn temp_resources(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("claude-plus-{name}-{unique}"));
    let assets = root.join("ion-dist").join("assets").join("v1");
    fs::create_dir_all(&assets).unwrap();
    root
}

fn state(states: &[super::FeatureState], feature: EnhanceFeatureId) -> bool {
    states
        .iter()
        .find_map(|state| (state.feature == feature).then_some(state.enabled))
        .unwrap_or(false)
}

fn feature_state(states: &[super::FeatureState], feature: EnhanceFeatureId) -> super::FeatureState {
    states
        .iter()
        .find(|state| state.feature == feature)
        .expect("feature state")
        .clone()
}

#[test]
fn script_markers_do_not_count_as_enabled_features() {
    let states = feature_states_from_text(&INJECT_SCRIPT);

    assert!(!state(&states, EnhanceFeatureId::ThirdPartyApi));
    assert!(!state(&states, EnhanceFeatureId::Plugins));
    assert!(!state(&states, EnhanceFeatureId::Mcp));
    assert!(!state(&states, EnhanceFeatureId::TokenUsage));
}

#[test]
fn english_inject_script_uses_english_visible_copy() {
    let script = super::inject_script_for_locale(super::EnhanceScriptLocale::EnUs);

    assert!(script.contains(r#"label:"Third-party API""#));
    assert!(script.contains(r#"label:"Skills""#));
    assert!(script.contains(r#"n.textContent="Export Markdown""#));
    assert!(script.contains("Data for reference only"));
    assert!(!script.contains(r#"label:"第三方API""#));
    assert!(!script.contains(r#"label:"技能""#));
    assert!(!script.contains(r#"n.textContent="导出 Markdown""#));
    assert!(!script.contains("本轮调用合计 "));
}

#[test]
fn zh_cn_inject_script_uses_chinese_visible_copy() {
    let script = super::inject_script_for_locale(super::EnhanceScriptLocale::ZhCn);

    assert!(script.contains(r#"label:"第三方API""#));
    assert!(script.contains(r#"label:"技能""#));
    assert!(script.contains(r#"n.textContent="导出 Markdown""#));
    assert!(script.contains("本轮调用合计 "));
    assert!(!script.contains(r#"label:"Third-party API""#));
    assert!(!script.contains(r#"n.textContent="Export Markdown""#));
}

#[test]
fn refresh_enabled_features_rewrites_script_locale_without_changing_markers() {
    let resources = temp_resources("enhance-locale-refresh");
    let bundle = resources
        .join("ion-dist")
        .join("assets")
        .join("v1")
        .join("index-test.js");
    fs::write(
        &bundle,
        format!(
            "const app=true;{}{}{}",
            super::inject_script_for_locale(super::EnhanceScriptLocale::ZhCn),
            feature_payload(NAV_API_MARKER),
            feature_payload(MARKDOWN_EXPORT_MARKER)
        ),
    )
    .unwrap();

    super::refresh_enabled_features_for_locale(&resources, super::EnhanceScriptLocale::EnUs)
        .expect("refresh enabled feature locale");
    let text = fs::read_to_string(&bundle).unwrap();
    fs::remove_dir_all(&resources).ok();

    assert!(text.contains(r#"label:"Third-party API""#));
    assert!(text.contains(r#"n.textContent="Export Markdown""#));
    assert!(text.contains(&feature_payload(NAV_API_MARKER)));
    assert!(text.contains(&feature_payload(MARKDOWN_EXPORT_MARKER)));
    assert!(!text.contains(r#"label:"第三方API""#));
    assert!(!text.contains(r#"n.textContent="导出 Markdown""#));
    assert!(!text.contains(&feature_payload(NAV_PLUGINS_MARKER)));
}

#[test]
fn feature_payload_controls_enabled_state() {
    let text = format!(
        "{}{}{}",
        &*INJECT_SCRIPT,
        feature_payload(NAV_API_MARKER),
        feature_payload(NAV_MCP_MARKER)
    );
    let states = feature_states_from_text(&text);

    assert!(state(&states, EnhanceFeatureId::ThirdPartyApi));
    assert!(!state(&states, EnhanceFeatureId::Plugins));
    assert!(state(&states, EnhanceFeatureId::Mcp));
    assert!(!state(&states, EnhanceFeatureId::ConversationTitleI18n));
    assert!(!state(&states, EnhanceFeatureId::Markdown));
    assert!(!state(&states, EnhanceFeatureId::Timeline));
    assert!(!state(&states, EnhanceFeatureId::TokenUsage));
    assert!(!text.contains(&feature_payload(NAV_PLUGINS_MARKER)));
    assert!(!text.contains(&feature_payload(CONVERSATION_TITLE_I18N_MARKER)));
}

#[test]
fn uninstalling_last_feature_removes_shared_script() {
    let text = format!("base{}", feature_payload(TOKEN_USAGE_MARKER));
    let text = super::set_marker(text, TOKEN_USAGE_MARKER, false);
    let text = super::ensure_or_remove_script(text, super::EnhanceScriptLocale::EnUs);

    assert!(!text.contains(SCRIPT_MARKER));
    assert!(!text.contains("cpuInstallFetchObserver"));
    assert!(!text.contains(TOKEN_USAGE_MARKER));
}

#[test]
fn feature_definitions_include_versions() {
    let list = super::feature_list(&[]);

    assert_eq!(list.len(), 7);
    let token_usage = list
        .iter()
        .find(|feature| feature.id == "token_usage")
        .expect("token usage feature");
    assert_eq!(token_usage.category, "对话增强");
    assert_eq!(token_usage.label, "Token 使用信息");
    assert!(token_usage.description.contains("本轮调用合计"));
    assert!(token_usage.available);
    for feature in list {
        let expected = match feature.id.as_str() {
            "plugins" | "token_usage" => "v0.3",
            _ => "v0.2",
        };
        assert_eq!(feature.version, expected, "{}", feature.id);
    }
}

#[test]
fn feature_payload_writes_current_version() {
    let payload = feature_payload(NAV_API_MARKER);

    assert!(payload.contains(NAV_API_MARKER));
    assert!(payload.contains("version:\"v0.2\""));
    assert!(!payload.contains("=true"));
}

#[test]
fn legacy_feature_payload_still_counts_as_enabled() {
    let legacy = format!(";window.{NAV_API_MARKER}=true;");
    let states = feature_states_from_text(&legacy);

    assert!(state(&states, EnhanceFeatureId::ThirdPartyApi));
}

#[test]
fn legacy_feature_payload_needs_upgrade_when_enabled() {
    let legacy = format!(";window.{NAV_API_MARKER}=true;");
    let states = feature_states_from_text(&legacy);
    let nav_api = feature_state(&states, EnhanceFeatureId::ThirdPartyApi);
    let mcp = feature_state(&states, EnhanceFeatureId::Mcp);

    assert!(nav_api.enabled);
    assert_eq!(nav_api.installed_version, None);
    assert!(nav_api.needs_upgrade());
    assert!(!mcp.enabled);
    assert!(!mcp.needs_upgrade());
}

#[test]
fn outdated_feature_payload_needs_upgrade_when_enabled() {
    let old = format!(r#";window.{NAV_API_MARKER}={{version:"v0.0"}};"#);
    let states = feature_states_from_text(&old);
    let nav_api = feature_state(&states, EnhanceFeatureId::ThirdPartyApi);

    assert!(nav_api.enabled);
    assert_eq!(nav_api.installed_version.as_deref(), Some("v0.0"));
    assert_eq!(nav_api.current_version, "v0.2");
    assert!(nav_api.needs_upgrade());
}

#[test]
fn current_feature_payload_does_not_need_upgrade() {
    let text = feature_payload(NAV_API_MARKER);
    let states = feature_states_from_text(&text);
    let nav_api = feature_state(&states, EnhanceFeatureId::ThirdPartyApi);

    assert!(nav_api.enabled);
    assert_eq!(nav_api.installed_version.as_deref(), Some("v0.2"));
    assert!(!nav_api.needs_upgrade());
}

#[test]
fn feature_states_does_not_upgrade_outdated_payloads_during_status_reads() {
    let resources = temp_resources("enhance-status-read-only");
    let bundle = resources
        .join("ion-dist")
        .join("assets")
        .join("v1")
        .join("index-test.js");
    let old_payload = format!(r#";window.{NAV_API_MARKER}={{version:"v0.0"}};"#);
    fs::write(&bundle, format!("const app=true;{old_payload}")).unwrap();

    let states = super::feature_states(&resources);
    let text = fs::read_to_string(&bundle).unwrap();
    fs::remove_dir_all(&resources).ok();

    let nav_api = feature_state(&states, EnhanceFeatureId::ThirdPartyApi);
    assert!(nav_api.enabled);
    assert!(nav_api.needs_upgrade());
    assert!(text.contains(&old_payload));
    assert!(!text.contains("version:\"v0.2\""));
}

#[test]
fn status_does_not_upgrade_outdated_payloads_during_status_reads() {
    let resources = temp_resources("enhance-status-entry-read-only");
    let bundle = resources
        .join("ion-dist")
        .join("assets")
        .join("v1")
        .join("index-test.js");
    let old_payload = format!(r#";window.{NAV_API_MARKER}={{version:"v0.0"}};"#);
    fs::write(&bundle, format!("const app=true;{old_payload}")).unwrap();

    let status = super::status_from_paths(Some(ClaudePaths {
        app: resources.clone(),
        resources: resources.clone(),
    }));
    let text = fs::read_to_string(&bundle).unwrap();
    fs::remove_dir_all(&resources).ok();

    let nav_api = status
        .features
        .iter()
        .find(|feature| feature.id == "third_party_api")
        .expect("third-party API feature status");
    assert!(nav_api.enabled);
    assert!(text.contains(&old_payload));
    assert!(!text.contains("version:\"v0.2\""));
}

#[test]
fn set_marker_replaces_old_payloads_without_touching_other_features() {
    let text = format!(
        "{}{}",
        format!(";window.{NAV_API_MARKER}=true;"),
        feature_payload(NAV_MCP_MARKER)
    );
    let next = super::set_marker(text, NAV_API_MARKER, true);

    assert_eq!(next.matches(NAV_API_MARKER).count(), 1);
    assert!(next.contains(&feature_payload(NAV_API_MARKER)));
    assert!(next.contains(&feature_payload(NAV_MCP_MARKER)));
    assert!(!next.contains(&format!(";window.{NAV_API_MARKER}=true;")));
}

#[test]
fn migration_replaces_only_enabled_outdated_markers() {
    let resources = temp_resources("enhance-migration");
    let bundle = resources
        .join("ion-dist")
        .join("assets")
        .join("v1")
        .join("index-test.js");
    fs::write(
        &bundle,
        format!(
            "const app=true;{}{}",
            format!(r#";window.{NAV_API_MARKER}={{version:"v0.0"}};"#),
            feature_payload(NAV_MCP_MARKER)
        ),
    )
    .unwrap();

    super::apply_feature_version_upgrades(&resources, &[EnhanceFeatureId::ThirdPartyApi])
        .expect("migrate features");
    let text = fs::read_to_string(&bundle).unwrap();
    fs::remove_dir_all(&resources).ok();

    assert!(text.contains(&feature_payload(NAV_API_MARKER)));
    assert!(text.contains(&feature_payload(NAV_MCP_MARKER)));
    assert!(!text.contains(r#"version:"v0.0""#));
    assert!(!text.contains(&feature_payload(NAV_PLUGINS_MARKER)));
}

#[test]
fn conversation_enhance_payload_controls_markdown_and_timeline_state() {
    let text = format!(
        "{}{}{}",
        &*INJECT_SCRIPT,
        feature_payload(MARKDOWN_EXPORT_MARKER),
        feature_payload(TIMELINE_MARKER)
    );
    let states = feature_states_from_text(&text);

    assert!(state(&states, EnhanceFeatureId::Markdown));
    assert!(state(&states, EnhanceFeatureId::Timeline));
    assert!(!state(&states, EnhanceFeatureId::ThirdPartyApi));
}

#[test]
fn token_usage_payload_controls_state() {
    let text = format!("{}{}", &*INJECT_SCRIPT, feature_payload(TOKEN_USAGE_MARKER));
    let states = feature_states_from_text(&text);

    assert!(state(&states, EnhanceFeatureId::TokenUsage));
    assert!(!state(&states, EnhanceFeatureId::Markdown));
    assert!(!state(&states, EnhanceFeatureId::Timeline));
}

#[test]
fn conversation_title_i18n_feature_is_inserted_before_markdown_export() {
    let list = super::feature_list(&[]);
    let title_i18n = list
        .iter()
        .position(|feature| feature.id == "conversation_title_i18n")
        .expect("conversation title i18n feature");
    let markdown = list
        .iter()
        .position(|feature| feature.id == "markdown")
        .expect("markdown feature");

    assert!(title_i18n < markdown);
    assert_eq!(list[title_i18n].category, "对话增强");
    assert_eq!(list[title_i18n].label, "对话列表中文化");
    assert!(list[title_i18n].description.contains("自动翻译为中文"));
    assert!(list[title_i18n].available);
}

#[test]
fn conversation_title_i18n_inject_script_calls_local_translate_endpoint() {
    assert!(INJECT_SCRIPT.contains("__claudePlusEnhanceConversationTitleI18nV1"));
    assert!(INJECT_SCRIPT.contains("window.claudePlusTitleI18n"));
    assert!(INJECT_SCRIPT.contains("data-claude-plus-original-title"));
    assert!(INJECT_SCRIPT.contains("data-claude-plus-title-i18n"));
    assert!(super::title_i18n_main_bridge_script().contains("/claude-plus/conversation-title-i18n"));
}

#[test]
fn conversation_title_i18n_uses_preload_bridge_instead_of_page_fetch() {
    assert!(INJECT_SCRIPT.contains("window.claudePlusTitleI18n"));
    assert!(!INJECT_SCRIPT.contains("fetch(\"http://127.0.0.1:"));
    assert!(!INJECT_SCRIPT.contains("/claude-plus/conversation-title-i18n"));

    let preload = super::title_i18n_preload_bridge_script();
    let main = super::title_i18n_main_bridge_script();
    assert!(preload.contains("contextBridge.exposeInMainWorld"));
    assert!(preload.contains("claudePlusTitleI18n"));
    assert!(main.contains("ipcMain.handle"));
    assert!(main.contains("function cppPort()"));
    assert!(main.contains("CLAUDE_PLUS_PROXY_PORT"));
    assert!(main.contains("settings.json"));
    assert!(main.contains("proxyPort??e.proxy_port"));
    assert!(main.contains("cppUrl(\"/claude-plus/conversation-title-i18n\")"));
    assert!(main.contains("/claude-plus/conversation-title-i18n"));
    assert!(!main.contains("http://127.0.0.1:15722/claude-plus"));
}

#[test]
fn conversation_title_i18n_avoids_regex_literal_slash_escape_crash() {
    assert!(!INJECT_SCRIPT.contains(r"/(^|\\/)"));
    assert!(!INJECT_SCRIPT.contains(r"(\\/|\\?|#|$)"));
    assert!(INJECT_SCRIPT.contains("new RegExp"));
}

#[test]
fn conversation_title_i18n_scans_plain_sidebar_list_items() {
    assert!(INJECT_SCRIPT.contains("aside div"));
    assert!(INJECT_SCRIPT.contains("nav div"));
    assert!(INJECT_SCRIPT.contains("aside li"));
    assert!(INJECT_SCRIPT.contains("aside [role=listitem]"));
    assert!(INJECT_SCRIPT.contains("return!!J(e)"));
    assert!(!INJECT_SCRIPT.contains("(/^[A-Za-z0-9][\\\\s\\\\S]{3,90}$/.test(r)&&s"));
}

#[test]
fn conversation_title_i18n_excludes_sidebar_shortcuts() {
    assert!(INJECT_SCRIPT.contains("Ctrl\\\\+"));
    assert!(INJECT_SCRIPT.contains("Cowork|Ctrl\\\\+B"));
    assert!(!INJECT_SCRIPT.contains("[Claude++] title i18n "));
    assert!(!INJECT_SCRIPT.contains(r#"O("scan""#));
    assert!(INJECT_SCRIPT.contains("function ac(e)"));
    assert!(INJECT_SCRIPT.contains(r#"[role="menu"]"#));
}

#[test]
fn conversation_title_i18n_does_not_skip_titles_containing_code_word() {
    assert!(!INJECT_SCRIPT.contains(
        "/新会话|计划任务|第三方API|技能|MCP|自定义|更多|Code|Drag to pin|已固定|最近使用|Ctrl\\\\+/i.test(r)"
    ));
    assert!(INJECT_SCRIPT.contains("function N(e){return/^("));
    assert!(INJECT_SCRIPT.contains("Code|Drag to pin"));
    assert!(INJECT_SCRIPT.contains("if(!s||N(r)||e.closest"));
}

#[test]
fn skills_popup_uses_preload_bridge_not_local_service() {
    assert!(INJECT_SCRIPT.contains("window.claudePlusSkills"));
    assert!(INJECT_SCRIPT.contains("width:min(886px,calc(100vw - 48px))"));
    assert!(INJECT_SCRIPT.contains("height:min(713px,calc(100vh - 48px))"));
    assert!(!INJECT_SCRIPT.contains("/claude-plus/skills"));
    assert!(!INJECT_SCRIPT.contains("无法连接 Claude++ 本地服务"));
}

#[test]
fn skills_popup_cards_use_compact_layout_with_details_action() {
    assert!(INJECT_SCRIPT.contains("cps-name"));
    assert!(INJECT_SCRIPT.contains("cps-brief"));
    assert!(INJECT_SCRIPT.contains("cps-file"));
    assert!(INJECT_SCRIPT.contains("data-cps-detail"));
    assert!(INJECT_SCRIPT.contains("cps-detail"));
    assert!(INJECT_SCRIPT.contains("原始描述"));
    assert!(INJECT_SCRIPT.contains("文件地址"));
    assert!(INJECT_SCRIPT.contains("目录地址"));
    assert!(INJECT_SCRIPT.contains("querySelector(\".cps-name\")"));
    assert!(!INJECT_SCRIPT.contains("适用于"));
    assert!(!INJECT_SCRIPT.contains("该技能用于"));
    assert!(!INJECT_SCRIPT.contains("cps-meta"));
    assert!(!INJECT_SCRIPT.contains("cps-summary"));
    assert!(!INJECT_SCRIPT.contains("project_path?"));
}

#[test]
fn mcp_nav_opens_custom3p_connectors_dialog() {
    assert!(INJECT_SCRIPT.contains(r#"open:"custom3p_connectors""#));
    assert!(INJECT_SCRIPT.contains("连接器与扩展"));
    assert!(INJECT_SCRIPT.contains("Connectors"));
    assert!(INJECT_SCRIPT.contains("Custom3pSetup"));
    assert!(!INJECT_SCRIPT.contains(
        r#"id:"mcp",marker:"__claudePlusEnhanceMcpV1",label:"MCP",path:"/customize/connectors""#
    ));
}

#[test]
fn markdown_export_uses_renderer_dom_and_blob_download() {
    assert!(INJECT_SCRIPT.contains("__claudePlusEnhanceMarkdownExportV1"));
    assert!(INJECT_SCRIPT.contains("claude-plus-markdown-menu-item"));
    assert!(INJECT_SCRIPT.contains("导出范围：当前页面已加载并渲染的对话内容"));
    assert!(INJECT_SCRIPT.contains("new Blob"));
    assert!(INJECT_SCRIPT.contains("download="));
    assert!(!INJECT_SCRIPT.contains("/export-markdown"));
}

#[test]
fn markdown_export_is_inserted_into_conversation_menu() {
    assert!(INJECT_SCRIPT.contains(r#"[role="menu"]"#));
    assert!(INJECT_SCRIPT.contains("data-claude-plus-markdown-menu-item"));
    assert!(INJECT_SCRIPT.contains("导出 Markdown"));
    assert!(INJECT_SCRIPT.contains("insertBefore(a,r)"));
    assert!(INJECT_SCRIPT.contains("n.slice(1).forEach"));
    assert!(INJECT_SCRIPT.contains(r#"querySelectorAll('button,[role="menuitem"],[cmdk-item]')"#));
    assert!(
        !INJECT_SCRIPT.contains(r#"querySelectorAll('button,[role="menuitem"],[cmdk-item],div')"#)
    );
    assert!(!INJECT_SCRIPT.contains("position:fixed;right:22px;top:74px"));
}

#[test]
fn timeline_uses_renderer_dom_markers_without_backend_bridge() {
    assert!(INJECT_SCRIPT.contains("__claudePlusEnhanceTimelineV1"));
    assert!(INJECT_SCRIPT.contains("claude-plus-timeline"));
    assert!(INJECT_SCRIPT.contains("claude-plus-timeline-marker"));
    assert!(INJECT_SCRIPT.contains("scrollIntoView"));
    assert!(INJECT_SCRIPT.contains("[data-message-author-role]"));
    assert!(!INJECT_SCRIPT.contains("claudePlusTimeline"));
}

#[test]
fn token_usage_polls_local_proxy_usage_and_renders_badge() {
    assert!(INJECT_SCRIPT.contains("__claudePlusEnhanceTokenUsageV1"));
    assert!(INJECT_SCRIPT.contains("claude-plus-token-usage"));
    assert!(INJECT_SCRIPT.contains("window.claudePlusTokenUsage"));
    assert!(!INJECT_SCRIPT.contains("/claude-plus/token-usage"));
    assert!(INJECT_SCRIPT.contains("cpuPoll"));
    assert!(INJECT_SCRIPT.contains("cpuLatestAssistant"));
    assert!(INJECT_SCRIPT.contains("[data-message-author-role=\"assistant\"]"));
    assert!(INJECT_SCRIPT.contains("Good response"));
    assert!(INJECT_SCRIPT.contains("inputTokens"));
    assert!(INJECT_SCRIPT.contains("cachedTokens"));
    assert!(INJECT_SCRIPT.contains("r=e.cachedReadTokens||e.cacheReadTokens||0"));
    assert!(!INJECT_SCRIPT.contains("Math.min(e.cachedReadTokens||e.cacheReadTokens||0,t)"));
    assert!(INJECT_SCRIPT.contains("本轮调用合计 "));
    assert!(INJECT_SCRIPT.contains("function cpuMount"));
    assert!(INJECT_SCRIPT.contains("function cpuClear"));
    assert!(INJECT_SCRIPT.contains("function cpuBusy"));
    assert!(INJECT_SCRIPT.contains("function cpuSend"));
    assert!(INJECT_SCRIPT.contains("function cpuStart"));
    assert!(INJECT_SCRIPT.contains("s.insertBefore(e,a?a.nextSibling:t.nextSibling)"));
    assert!(INJECT_SCRIPT.contains("display:flex;flex-direction:column"));
    assert!(!INJECT_SCRIPT.contains("if(!t){cpuClear();return}"));
    assert!(!INJECT_SCRIPT.contains("if(cpuBusy()){cpuClear();cpuPoll(!0);return}"));
    assert!(INJECT_SCRIPT.contains("if(!t)return"));
    assert!(INJECT_SCRIPT.contains("cpu.pending"));
    assert!(INJECT_SCRIPT.contains("const busy=cpuBusy()"));
    assert!(INJECT_SCRIPT.contains("if(cpu.wasBusy&&!busy)cpuScheduleFinalRender()"));
    assert!(INJECT_SCRIPT.contains("a=cpu.currentTurn?.startedAt||0;if(!a)return"));
    assert!(INJECT_SCRIPT.contains("s={sinceMs:a}"));
    assert!(INJECT_SCRIPT.contains("await e.get(s)"));
    assert!(INJECT_SCRIPT.contains(r#"e&&typeof e.get==="function"?await e.get(s):null"#));
    assert!(!INJECT_SCRIPT.contains("a?{sinceMs:a}:{}"));
    assert!(!INJECT_SCRIPT.contains("a?(\"?sinceMs=\"+encodeURIComponent(a))"));
    assert!(INJECT_SCRIPT.contains("cpu.currentTurn&&cpuPoll()"));
    assert!(INJECT_SCRIPT.contains("setInterval(()=>cpu.currentTurn&&cpuPoll(!0),1200)"));
    assert!(INJECT_SCRIPT.contains("l=cpuApiUrl(a)&&!!cpu.currentTurn"));
    assert!(INJECT_SCRIPT
        .contains("r=this.__claudePlusTokenUsageUrl,a=cpuApiUrl(r)&&!!cpu.currentTurn"));
    assert!(!INJECT_SCRIPT.contains("cpuApiUrl(a)&&(cpu.pending=!0,cpuEnsureTurn())"));
    assert!(!INJECT_SCRIPT.contains("cpuApiUrl(r)&&(cpu.pending=!0,cpuEnsureTurn())"));
    assert!(!INJECT_SCRIPT.contains("if(busy){if(!cpu.pending)cpuBeginTurn()"));
    assert!(INJECT_SCRIPT.contains(
        "[\"submit\",\"click\",\"keydown\"].forEach(e=>document.addEventListener(e,cpuStart,!0))"
    ));
    assert!(INJECT_SCRIPT.contains("cpu.pollBusy||(!e&&n-cpu.lastPollAt<350)"));
    assert!(!INJECT_SCRIPT.contains("setInterval(()=>cpuPoll(!0),1200)"));
    assert!(!INJECT_SCRIPT.contains("http://127.0.0.1:"));
    assert!(!INJECT_SCRIPT.contains("__CPP_TOKEN_USAGE_URL__"));
    assert!(!INJECT_SCRIPT.contains("function cpuHost"));
    assert!(!INJECT_SCRIPT.contains("n.appendChild(e)"));
    assert!(!INJECT_SCRIPT.contains("Token 使用信息：等待下一次"));
    assert!(
        !INJECT_SCRIPT.contains("document.querySelector(\"textarea,[contenteditable='true']\")")
    );

    let preload = super::token_usage_preload_bridge_script();
    let main = super::token_usage_main_bridge_script();
    assert!(preload.contains("claudePlusTokenUsage"));
    assert!(preload.contains("get:e=>ipcRenderer.invoke"));
    assert!(preload.contains("e||{}"));
    assert!(main.contains("function cppPort()"));
    assert!(main.contains("CLAUDE_PLUS_PROXY_PORT"));
    assert!(main.contains("settings.json"));
    assert!(main.contains("proxyPort??e.proxy_port"));
    assert!(main.contains("cppUrl(\"/claude-plus/token-usage\")"));
    assert!(main.contains("t+\"?sinceMs=\"+encodeURIComponent(e.sinceMs)"));
    assert!(!main.contains("http://127.0.0.1:15722/claude-plus"));
    assert!(main.contains("ipcMain.handle(\"claude-plus:token-usage\",(e,n)=>getUsage(n))"));
    assert!(!main.contains("__CPP_TOKEN_USAGE_URL__"));
    assert!(main.contains("ipcMain.handle"));
}

#[test]
fn token_usage_captures_page_network_like_codex_plus_script() {
    assert!(INJECT_SCRIPT.contains("window.__claudePlusTokenUsage"));
    assert!(INJECT_SCRIPT.contains("function cpuNormalizeUsage"));
    assert!(INJECT_SCRIPT.contains("function cpuExtractUsages"));
    assert!(INJECT_SCRIPT.contains("CPU_MAX_CAPTURE_TEXT_LENGTH"));
    assert!(INJECT_SCRIPT.contains("CPU_MAX_CAPTURE_BLOB_BYTES"));
    assert!(INJECT_SCRIPT.contains("CPU_MAX_COLLECT_DEPTH"));
    assert!(INJECT_SCRIPT.contains("function cpuInstallFetchObserver"));
    assert!(INJECT_SCRIPT.contains("function cpuInstallXhrObserver"));
    assert!(INJECT_SCRIPT.contains("function cpuInstallWebSocketObserver"));
    assert!(INJECT_SCRIPT.contains("function cpuInstallPostMessageObserver"));
    assert!(INJECT_SCRIPT.contains("response.clone().text()"));
    assert!(INJECT_SCRIPT.contains("XMLHttpRequest.prototype.send"));
    assert!(INJECT_SCRIPT.contains("new NativeWebSocket"));
    assert!(INJECT_SCRIPT.contains("cacheReadTokens"));
    assert!(INJECT_SCRIPT.contains("cacheCreationTokens"));
    assert!(INJECT_SCRIPT.contains("cachedReadTokens"));
    assert!(INJECT_SCRIPT.contains(r"!/\/claude-plus\/token-usage\b/i.test(t)"));
}

#[test]
fn skills_modal_uses_dom_builders_for_dynamic_content() {
    assert!(INJECT_SCRIPT.contains("function cpsText"));
    assert!(INJECT_SCRIPT.contains("function cpsRenderSections"));
    assert!(INJECT_SCRIPT.contains("function cpsSetStatus"));
    assert!(!INJECT_SCRIPT.contains("n.innerHTML=C(a,\"global\")+C(a,\"project\")"));
    assert!(!INJECT_SCRIPT.contains("function z(e)"));
}

#[test]
fn token_usage_observers_are_installed_only_when_feature_enabled() {
    assert!(INJECT_SCRIPT.contains(
        "function cpuInstallObservers(){if(!window.__claudePlusEnhanceTokenUsageV1)return;"
    ));
    assert!(INJECT_SCRIPT.contains("cpuInstallObservers();"));
    assert!(!INJECT_SCRIPT.contains(
        "cpuInstallFetchObserver();cpuInstallXhrObserver();cpuInstallWebSocketObserver();cpuInstallPostMessageObserver();\nnew MutationObserver"
    ));
    assert!(!INJECT_SCRIPT.contains(
        "[\"submit\",\"click\",\"keydown\"].forEach(e=>document.addEventListener(e,cpuStart,!0));\ndocument.readyState"
    ));
}

#[test]
fn token_usage_aggregates_multiple_calls_into_one_turn() {
    assert!(INJECT_SCRIPT.contains("currentTurn"));
    assert!(INJECT_SCRIPT.contains("function cpuBeginTurn"));
    assert!(INJECT_SCRIPT.contains("function cpuEnsureTurn"));
    assert!(INJECT_SCRIPT.contains("function cpuAggregateTurn"));
    assert!(INJECT_SCRIPT.contains("callCount"));
    assert!(INJECT_SCRIPT.contains("e.call_count??e.callCount"));
    assert!(INJECT_SCRIPT.contains("calls.push"));
    assert!(INJECT_SCRIPT.contains("function cpuRememberUsage"));
    assert!(INJECT_SCRIPT.contains("const r=cpu.currentTurn;if(!r)return!1"));
    assert!(INJECT_SCRIPT.contains("if(r.scopeKey!==a.scopeKey)return!1"));
    assert!(!INJECT_SCRIPT.contains("let r=cpuEnsureTurn()"));
    assert!(INJECT_SCRIPT.contains("cpu.lastProxyId"));
    assert!(INJECT_SCRIPT.contains("cpuRememberUsage(t,t.elapsed,t.source||\"proxy\")"));
    assert!(INJECT_SCRIPT.contains("callCount:e.callCount+(n.callCount||1)"));
    assert!(INJECT_SCRIPT.contains("调用 \"+cpuF(e.callCount)+\" 次"));
}

#[test]
fn token_usage_matches_codex_plus_scope_context_and_debug_contract() {
    assert!(INJECT_SCRIPT.contains("const CPU_RECENT_LIMIT=20"));
    assert!(INJECT_SCRIPT.contains("const CPU_DEBUG_LIMIT=50"));
    assert!(INJECT_SCRIPT.contains("const CPU_LEDGER_LIMIT=500"));
    assert!(INJECT_SCRIPT.contains("const CPU_CONTEXT_POLL_INTERVAL_MS=1000"));
    assert!(INJECT_SCRIPT.contains("const CPU_TURN_IDLE_TIMEOUT_MS=120000"));
    assert!(INJECT_SCRIPT.contains("const CPU_CONTEXT_MERGE_WINDOW_MS=30000"));
    assert!(INJECT_SCRIPT.contains("const CPU_CROSS_SOURCE_DEDUPE_WINDOW_MS=3000"));
    assert!(INJECT_SCRIPT.contains("function cpuCurrentProjectId"));
    assert!(INJECT_SCRIPT.contains("function cpuCurrentConversationId"));
    assert!(INJECT_SCRIPT.contains("function cpuCurrentScopeKey"));
    assert!(INJECT_SCRIPT.contains("projectId"));
    assert!(INJECT_SCRIPT.contains("conversationId"));
    assert!(INJECT_SCRIPT.contains("scopeKey"));
    assert!(INJECT_SCRIPT.contains("function cpuInstallContextMeterObserver"));
    assert!(INJECT_SCRIPT.contains("window.__codexContextMeter"));
    assert!(INJECT_SCRIPT.contains("captureState.inspectText"));
    assert!(INJECT_SCRIPT.contains("captureState.inspectValue"));
    assert!(INJECT_SCRIPT.contains("function cpuSameUsage"));
    assert!(INJECT_SCRIPT.contains("CPU_CROSS_SOURCE_DEDUPE_WINDOW_MS"));
    assert!(INJECT_SCRIPT.contains("totalEstimated"));
    assert!(INJECT_SCRIPT.contains("(估算)"));
    assert!(INJECT_SCRIPT.contains("window.__claudePlusTokenUsageDebug"));
    assert!(INJECT_SCRIPT.contains("export:()=>"));
    assert!(INJECT_SCRIPT.contains("ledgerEvents"));
    assert!(INJECT_SCRIPT.contains("cpu.ledger.slice(-CPU_LEDGER_LIMIT)"));
}

#[test]
fn token_usage_capture_tuning_is_injected_from_runtime_settings() {
    let mut tuning = crate::settings::ProxyRuntimeTuning::default();
    tuning.token_usage_capture.recent_limit = 3;
    tuning.token_usage_capture.debug_limit = 4;
    tuning.token_usage_capture.ledger_limit = 5;
    tuning.token_usage_capture.context_poll_interval_ms = 600;
    tuning.token_usage_capture.turn_idle_timeout_ms = 700;
    tuning.token_usage_capture.context_merge_window_ms = 800;
    tuning.token_usage_capture.cross_source_dedupe_window_ms = 900;
    tuning.token_usage_capture.final_render_delay_ms = 1000;
    tuning.token_usage_capture.max_capture_text_length = 1100;
    tuning.token_usage_capture.max_capture_blob_bytes = 1200;
    tuning.token_usage_capture.max_collect_depth = 2;

    let script =
        super::inject_script_for_locale_with_tuning(super::EnhanceScriptLocale::ZhCn, &tuning);

    assert!(script.contains("const CPU_RECENT_LIMIT=3"));
    assert!(script.contains("const CPU_DEBUG_LIMIT=4"));
    assert!(script.contains("const CPU_LEDGER_LIMIT=5"));
    assert!(script.contains("const CPU_CONTEXT_POLL_INTERVAL_MS=600"));
    assert!(script.contains("const CPU_TURN_IDLE_TIMEOUT_MS=700"));
    assert!(script.contains("const CPU_CONTEXT_MERGE_WINDOW_MS=800"));
    assert!(script.contains("const CPU_CROSS_SOURCE_DEDUPE_WINDOW_MS=900"));
    assert!(script.contains("const CPU_FINAL_RENDER_DELAY_MS=1000"));
    assert!(script.contains("const CPU_MAX_CAPTURE_TEXT_LENGTH=1100"));
    assert!(script.contains("const CPU_MAX_CAPTURE_BLOB_BYTES=1200"));
    assert!(script.contains("const CPU_MAX_COLLECT_DEPTH=2"));
    assert!(!script.contains("__CPU_"));
}

#[test]
fn token_usage_waits_for_turn_end_and_uses_requested_display_contract() {
    assert!(INJECT_SCRIPT.contains("const CPU_FINAL_RENDER_DELAY_MS=900"));
    assert!(INJECT_SCRIPT.contains("function cpuScheduleFinalRender"));
    assert!(INJECT_SCRIPT.contains("function cpuFinalizeTurnRender"));
    assert!(INJECT_SCRIPT.contains("cpu.renderTimer=setTimeout"));
    assert!(INJECT_SCRIPT.contains("cpu.renderReady"));
    assert!(INJECT_SCRIPT.contains("cpu.wasBusy&&!busy"));
    assert!(!INJECT_SCRIPT.contains("cpuPublish();cpuRender();return!0"));

    assert!(INJECT_SCRIPT.contains("本轮调用合计 "));
    assert!(INJECT_SCRIPT.contains("输入 "));
    assert!(INJECT_SCRIPT.contains("输出 "));
    assert!(INJECT_SCRIPT.contains("缓存写 "));
    assert!(INJECT_SCRIPT.contains("缓存读 "));
    assert!(INJECT_SCRIPT.contains("缓存命中率 "));
    assert!(INJECT_SCRIPT.contains("上下文 "));
    assert!(!INJECT_SCRIPT.contains("上下文占比 "));
    assert!(INJECT_SCRIPT.contains("数据仅供参考"));
    assert!(INJECT_SCRIPT.contains("调用 "));
    assert!(INJECT_SCRIPT.contains("耗时 "));
    assert!(INJECT_SCRIPT.contains("cacheCreationTokens"));
    assert!(INJECT_SCRIPT.contains("cachedReadTokens"));
    assert!(INJECT_SCRIPT.contains("e.callCount"));
    assert!(!INJECT_SCRIPT.contains("缓存创建 "));
    assert!(!INJECT_SCRIPT.contains("缓存读取 "));
}

#[test]
fn token_usage_cache_hit_rate_uses_only_cache_read_tokens() {
    let normalize_start = INJECT_SCRIPT.find("function cpuNormalizeUsage").unwrap();
    let normalize_end = INJECT_SCRIPT[normalize_start..]
        .find("function cpuCollectUsages")
        .map(|offset| normalize_start + offset)
        .unwrap();
    let normalize = &INJECT_SCRIPT[normalize_start..normalize_end];

    assert!(normalize.contains("cache_read_input_tokens"));
    assert!(normalize.contains("cacheReadInputTokens"));
    assert!(normalize.contains("l!=null&&o?e.cachedTokens"));
    assert!(normalize.contains("m!=null&&d?e.cacheCreationTokens"));
    assert!(normalize.contains("prompt_tokens_details?.cached_tokens"));
    assert!(normalize.contains("input_tokens_details?.cached_tokens"));
    assert!(normalize.contains("e.cache_read_known??e.cacheReadKnown"));
    assert!(!normalize.contains("e.cached_tokens"));
    assert!(!normalize.contains("??e.cachedTokens"));
    assert!(!normalize.contains("cached_tokens\",\"cachedTokens"));
    assert!(!normalize.contains("cached_input_tokens"));
    assert!(!normalize.contains("cachedInputTokens"));
    assert!(!INJECT_SCRIPT.contains("e.cachedReadTokens||e.cacheReadTokens||e.cached"));
}

#[test]
fn token_usage_mounts_after_assistant_footer_not_run_status() {
    assert!(INJECT_SCRIPT.contains("function cpuAssistantFooter"));
    assert!(INJECT_SCRIPT.contains("function cpuInsertAfter"));
    assert!(INJECT_SCRIPT.contains("const a=cpuAssistantFooter(t)"));
    assert!(INJECT_SCRIPT.contains("const s=a?.parentElement||r"));
    assert!(INJECT_SCRIPT.contains("cpuLooksLikeRunStatus"));
    assert!(INJECT_SCRIPT.contains("!cpuLooksLikeRunStatus(e)"));
    assert!(INJECT_SCRIPT.contains("s.insertBefore(e,a?a.nextSibling:t.nextSibling)"));
    assert!(!INJECT_SCRIPT.contains("r.insertBefore(e,t.nextSibling)"));
}

#[test]
fn token_usage_distinguishes_zero_from_unknown_cache_and_clears_on_next_turn() {
    assert!(INJECT_SCRIPT.contains("cacheReadKnown"));
    assert!(INJECT_SCRIPT.contains("cacheCreationKnown"));
    assert!(INJECT_SCRIPT.contains("cpuKnownAdd"));
    assert!(INJECT_SCRIPT.contains("function cpuCacheText"));
    assert!(INJECT_SCRIPT.contains("function cpuRateText"));
    assert!(INJECT_SCRIPT.contains("缓存写 "));
    assert!(INJECT_SCRIPT.contains("缓存读 "));
    assert!(!INJECT_SCRIPT.contains("缓存创建 "));
    assert!(!INJECT_SCRIPT.contains("缓存读取 "));
    assert!(INJECT_SCRIPT.contains("function cpuRateText"));
    assert!(INJECT_SCRIPT.contains("const r=t+n+(o||0)"));
    assert!(INJECT_SCRIPT.contains("return e&&r?cpuPct(Math.min(n,r),r):\"未知\""));
    assert!(!INJECT_SCRIPT.contains("return e&&t?cpuPct(n,t):\"未知\""));
    assert!(INJECT_SCRIPT.contains("cpuRateText(e.cacheReadKnown,r,t,e.cacheCreationTokens||0)"));
    assert!(INJECT_SCRIPT.contains("function cpuContextText"));
    assert!(INJECT_SCRIPT.contains("if(!a)return \"\""));
    assert!(INJECT_SCRIPT.contains("return \"上下文 \"+cpuF(r)+\"/\"+cpuF(a)"));
    assert!(INJECT_SCRIPT.contains("s?\" · \"+s:\"\""));
    assert!(!INJECT_SCRIPT.contains("(0%)"));
    assert!(INJECT_SCRIPT.contains("cpuClear();return cpu.currentTurn"));
    assert!(INJECT_SCRIPT.contains("width:min(592px,calc(100% - 48px))"));
    assert!(INJECT_SCRIPT.contains("font:11.5px/1.35"));
}

#[test]
fn token_usage_ignores_ccswitch_fallback_after_page_capture_or_final_render() {
    assert!(INJECT_SCRIPT.contains("const weak=/^(cc-switch|proxy)$/.test(a.source||\"\")"));
    assert!(INJECT_SCRIPT.contains("if(cpu.renderReady&&weak)return!1"));
    assert!(INJECT_SCRIPT.contains(
        "if(weak&&r.calls.some(e=>!/^(cc-switch|proxy)$/.test(e.source||\"\")))return!1"
    ));
    assert!(INJECT_SCRIPT.contains(
        "if(!weak)r.calls=r.calls.filter(e=>!/^(cc-switch|proxy)$/.test(e.source||\"\"))"
    ));
    assert!(INJECT_SCRIPT.contains("source:e.source||\"\""));
}

#[test]
fn token_usage_badge_uses_translucent_blue_and_reference_hint() {
    assert!(INJECT_SCRIPT.contains("background:rgba(37,99,235,.12)"));
    assert!(INJECT_SCRIPT.contains("border:1px solid rgba(37,99,235,.35)"));
    assert!(INJECT_SCRIPT.contains("数据仅供参考"));
}

#[test]
fn nsis_installer_uses_simplified_chinese_language() {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../../../tauri.conf.json")).unwrap();
    let languages = config
        .pointer("/bundle/windows/nsis/languages")
        .and_then(|value| value.as_array())
        .expect("nsis languages");

    assert_eq!(
        languages.first().and_then(|value| value.as_str()),
        Some("SimpChinese")
    );

    assert_eq!(
        config
            .pointer("/bundle/windows/nsis/customLanguageFiles/SimpChinese")
            .and_then(|value| value.as_str()),
        Some("resources/nsis/SimpChinese.nsh")
    );

    let language = include_str!("../../../resources/nsis/SimpChinese.nsh");
    assert!(language.contains("删除应用程序数据"));
    assert!(language.contains("正在运行"));
    assert!(language.contains("点击确定以终止运行"));
    assert!(language.contains("卸载"));
}

#[test]
fn preload_bridge_exposes_skills_api() {
    let script = super::skills_bridge_script();
    assert!(script.contains("contextBridge.exposeInMainWorld"));
    assert!(script.contains("claudePlusSkills"));
    assert!(script.contains("ipcRenderer.invoke"));
    assert!(script.contains(SKILLS_LIST_CHANNEL));
    assert!(script.contains(SKILLS_TRASH_CHANNEL));
}

#[test]
fn main_bridge_handles_skills_filesystem_api() {
    let script = super::skills_main_bridge_script(super::EnhanceScriptLocale::ZhCn);
    assert!(script.contains("ipcMain.handle"));
    assert!(script.contains("require(\"fs\")"));
    assert!(script.contains("shell.trashItem"));
    assert!(script.contains("fetch(cppUrl(\"/claude-plus/skills\")"));
    assert!(script.contains("x-claude-plus-gateway-token"));
    assert!(script.contains("local-gateway-token"));
    assert!(script.contains("async function listSkillsFast"));
    assert!(script.contains("async function trashSkillFast"));
    assert!(script.contains("return listSkills()"));
    assert!(!script.contains("return trashSkill(e)"));
    assert!(script.contains("listSkills"));
    assert!(script.contains(SKILLS_LIST_CHANNEL));
    assert!(script.contains(SKILLS_TRASH_CHANNEL));
    assert!(!script.contains("__CPP_GATEWAY_RUNTIME__"));
    assert!(!script.contains("__CPP_TOKEN_READER__"));
}

#[test]
fn main_bridge_visible_copy_tracks_locale() {
    let english = super::skills_main_bridge_script(super::EnhanceScriptLocale::EnUs);
    let chinese = super::skills_main_bridge_script(super::EnhanceScriptLocale::ZhCn);

    assert!(english.contains("No description provided."));
    assert!(english.contains(r#"collectRoot(r,"global","Global",null,e)"#));
    assert!(english.contains("Skill not found"));
    assert!(!english.contains("未提供描述"));
    assert!(!english.contains(r#"collectRoot(r,"global","全局",null,e)"#));

    assert!(chinese.contains("未提供描述。"));
    assert!(chinese.contains(r#"collectRoot(r,"global","全局",null,e)"#));
    assert!(chinese.contains("未找到该 skill"));
    assert!(!chinese.contains("No description provided."));
}

#[test]
fn title_i18n_bridge_uses_local_gateway_without_filesystem_access() {
    let preload = super::title_i18n_preload_bridge_script();
    let main = super::title_i18n_main_bridge_script();

    assert!(preload.contains("claudePlusTitleI18n"));
    assert!(preload.contains("ipcRenderer.invoke"));
    assert!(!preload.contains("require(\"fs\")"));
    assert!(main.contains("fetch("));
    assert!(main.contains("local-gateway-token"));
    assert!(main.contains("x-claude-plus-gateway-token"));
    assert!(main.contains("function cppPort()"));
    assert!(main.contains("CLAUDE_PLUS_PROXY_PORT"));
    assert!(main.contains("settings.json"));
    assert!(main.contains("cppUrl(\"/claude-plus/conversation-title-i18n\")"));
    assert!(!main.contains("http://127.0.0.1:15722/claude-plus"));
    assert!(!main.contains("__CPP_TITLE_I18N_URL__"));
    assert!(!main.contains("__CPP_GATEWAY_RUNTIME__"));
    assert!(!main.contains("__CPP_TOKEN_READER__"));
    assert!(!main.contains("shell.trashItem"));
}

#[test]
fn token_usage_bridge_sends_local_gateway_token_header() {
    let main = super::token_usage_main_bridge_script();

    assert!(main.contains("local-gateway-token"));
    assert!(main.contains("x-claude-plus-gateway-token"));
    assert!(main.contains("function cppPort()"));
    assert!(main.contains("CLAUDE_PLUS_PROXY_PORT"));
    assert!(main.contains("settings.json"));
    assert!(main.contains("cppUrl(\"/claude-plus/token-usage\")"));
    assert!(!main.contains("http://127.0.0.1:15722/claude-plus"));
    assert!(!main.contains("__CPP_TOKEN_USAGE_URL__"));
    assert!(!main.contains("__CPP_GATEWAY_RUNTIME__"));
    assert!(!main.contains("__CPP_TOKEN_READER__"));
}

#[test]
fn preload_bridge_is_sandbox_safe() {
    let script = super::skills_bridge_script();
    assert!(!script.contains("require(\"fs\")"));
    assert!(!script.contains("require(\"path\")"));
    assert!(!script.contains("require(\"crypto\")"));
    assert!(script.contains("ipcRenderer.invoke"));
}

#[test]
fn skills_bridge_targets_main_view_preload() {
    assert_eq!(SKILLS_PRELOAD_BRIDGE_TARGET, ".vite/build/mainView.js");
    assert_eq!(SKILLS_MAIN_BRIDGE_TARGET, ".vite/build/index.js");
}

#[test]
fn skills_bridge_is_inserted_before_source_map_comment() {
    let preload = "const ready=true;\n//# sourceMappingURL=mainView.js.map";
    let mut next = remove_skills_bridge(preload);
    next.insert_str(0, &super::skills_bridge_script());

    let bridge_index = next.find("__claudePlusSkillsBridgeV1").unwrap();
    let source_map_index = next.find("sourceMappingURL").unwrap();
    assert!(bridge_index < source_map_index);
    assert!(next.starts_with(";(()=>{const MARK=\"__claudePlusSkillsBridgeV1\""));
}

#[test]
fn remove_skills_bridge_cleans_multiple_residues() {
    let text = format!(
        "{}const ready=true;{}//# sourceMappingURL=mainView.js.map",
        super::skills_bridge_script(),
        super::skills_main_bridge_script(super::EnhanceScriptLocale::ZhCn)
    );
    let cleaned = remove_skills_bridge(&text);

    assert_eq!(
        cleaned,
        "const ready=true;//# sourceMappingURL=mainView.js.map"
    );
}

#[test]
fn batch_bridge_patch_updates_main_and_preload_content() {
    let script = ";(()=>{const MARK=\"__claudePlusBatchTest\";})();";
    let old = ";(()=>{const MARK=\"__claudePlusSkillsBridgeV1\";})();";
    let patched = super::patch_bridge_files_for_test(
        vec![
            (SKILLS_MAIN_BRIDGE_TARGET, "const main=true;"),
            (
                SKILLS_PRELOAD_BRIDGE_TARGET,
                &format!("{old}const preload=true;"),
            ),
        ],
        true,
        remove_skills_bridge,
        script,
    )
    .expect("patch bridge contents");

    assert_eq!(patched.len(), 2);
    assert!(patched[0].starts_with(script));
    assert!(patched[0].contains("const main=true;"));
    assert!(patched[1].starts_with(script));
    assert!(patched[1].contains("const preload=true;"));
    assert!(!patched[1].contains(old));
}

#[test]
#[ignore = "writes Claude Desktop resources; set CLAUDE_PLUS_VERIFY_INSTALL=1"]
fn verify_install_plugins_enhance_writes_skills_bridges() {
    assert_eq!(
        std::env::var("CLAUDE_PLUS_VERIFY_INSTALL")
            .as_deref()
            .map(str::trim),
        Ok("1")
    );

    super::install("plugins").expect("install plugins enhance");
    let status = super::status();
    let plugins = status
        .features
        .iter()
        .find(|feature| feature.id == "plugins")
        .expect("plugins feature status");

    assert!(plugins.enabled);
}

#[test]
#[ignore = "writes Claude Desktop resources; set CLAUDE_PLUS_VERIFY_INSTALL=1"]
fn verify_install_title_i18n_enhance_writes_bridge() {
    assert_eq!(
        std::env::var("CLAUDE_PLUS_VERIFY_INSTALL")
            .as_deref()
            .map(str::trim),
        Ok("1")
    );

    super::install("conversation_title_i18n").expect("install title i18n enhance");
    let status = super::status();
    let feature = status
        .features
        .iter()
        .find(|feature| feature.id == "conversation_title_i18n")
        .expect("title i18n feature status");

    assert!(feature.enabled);
}

#[test]
#[ignore = "writes Claude Desktop resources; set CLAUDE_PLUS_VERIFY_INSTALL=1"]
fn verify_install_token_usage_enhance_writes_bridge() {
    assert_eq!(
        std::env::var("CLAUDE_PLUS_VERIFY_INSTALL")
            .as_deref()
            .map(str::trim),
        Ok("1")
    );

    super::install("token_usage").expect("install token usage enhance");
    let status = super::status();
    let feature = status
        .features
        .iter()
        .find(|feature| feature.id == "token_usage")
        .expect("token usage feature status");

    assert!(feature.enabled);
}

#[test]
#[ignore = "writes Claude Desktop resources; set CLAUDE_PLUS_VERIFY_INSTALL=1"]
fn verify_uninstall_token_usage_enhance_removes_observers() {
    assert_eq!(
        std::env::var("CLAUDE_PLUS_VERIFY_INSTALL")
            .as_deref()
            .map(str::trim),
        Ok("1")
    );

    super::uninstall("token_usage").expect("uninstall token usage enhance");
    let paths =
        crate::claude_patch_common::resolve_claude_paths().expect("resolve Claude Desktop paths");
    let text = super::read_index_bundle(&paths.resources).expect("read installed index bundle");
    let status = super::status();
    let feature = status
        .features
        .iter()
        .find(|feature| feature.id == "token_usage")
        .expect("token usage feature status");
    let main_bridge = super::read_asar_file(&paths.resources, SKILLS_MAIN_BRIDGE_TARGET)
        .expect("read installed main bridge");
    let main_bridge = String::from_utf8(main_bridge).expect("main bridge is utf8");

    assert!(!feature.enabled);
    assert!(!text.contains(&feature_payload(TOKEN_USAGE_MARKER)));
    assert!(!main_bridge.contains(TOKEN_USAGE_MAIN_BRIDGE_MARKER));
    if !text.contains("__claudePlusEnhanceNavV2") {
        assert!(!text.contains("cpuInstallFetchObserver"));
    } else {
        assert!(text.contains(
            "function cpuInstallObservers(){if(!window.__claudePlusEnhanceTokenUsageV1)return;"
        ));
    }
}
