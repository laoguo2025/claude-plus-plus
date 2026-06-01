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
    const INJECT_SCRIPT: &str = r##";(()=>{const m="__claudePlusEnhanceNavV2";
if(window[m])return;
Object.defineProperty(window,m,{value:!0});
const v="3.7",n=[
{id:"third_party_api",marker:"__claudePlusEnhanceThirdPartyApiV1",label:"第三方API",path:"/setup-desktop-3p",open:"custom3p",icon:'<svg width="16" height="16" style="width:1em;height:1em;min-width:1em;max-width:1em;min-height:1em;max-height:1em;flex:none;display:block" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M8 8h8"/><path d="M8 12h8"/><path d="M8 16h8"/><rect x="4" y="5" width="16" height="14" rx="2"/></svg>'},
{id:"plugins",marker:"__claudePlusEnhancePluginsV1",label:"技能",path:"/customize/plugins/new?marketplace&plugin",open:"skills",icon:'<svg width="16" height="16" style="width:1em;height:1em;min-width:1em;max-width:1em;min-height:1em;max-height:1em;flex:none;display:block" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M7 7h10v10H7z"/><path d="M10 3h4v4"/><path d="M10 21h4v-4"/><path d="M3 10h4"/><path d="M17 14h4"/></svg>'},
{id:"mcp",marker:"__claudePlusEnhanceMcpV1",label:"MCP",path:"/customize/connectors",icon:'<svg width="16" height="16" style="width:1em;height:1em;min-width:1em;max-width:1em;min-height:1em;max-height:1em;flex:none;display:block" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="12" r="2.5"/><circle cx="18" cy="7" r="2.5"/><circle cx="18" cy="17" r="2.5"/><path d="M8.3 10.9 15.7 8.1"/><path d="M8.3 13.1 15.7 15.9"/></svg>'}
];
let q=0,b=!1;
function o(e){return(e.textContent||"").replace(/\s+/g," ").trim()}
function i(e){return Array.from(e.querySelectorAll("a,button,[role=button]"))}
function p(){return n.filter(e=>window[e.marker])}
function d(e){return e&&e.dataset&&e.dataset.claudePlusEnhance==="1"}
function c(e){return!d(e)&&/计划任务|Scheduled/.test(o(e))}
function k(e){e.setAttribute("aria-hidden","true");e.setAttribute("focusable","false");e.style.width="1em";e.style.height="1em";e.style.minWidth="1em";e.style.maxWidth="1em";e.style.minHeight="1em";e.style.maxHeight="1em";e.style.flex="none";e.style.display=e.tagName==="SVG"?"block":"inline-flex";if(e.tagName!=="SVG"){e.style.alignItems="center";e.style.justifyContent="center"}}
function l(e){const n=document.createElement("template");n.innerHTML=e.trim();const t=n.content.firstElementChild;if(!t)return null;k(t);return t}
function w(e){const n=document.createElement("span");n.dataset.claudePlusEnhanceIcon="1";k(n);const t=l(e);return t&&n.appendChild(t),n}
function f(e,n){let t;const r=document.createTreeWalker(e,NodeFilter.SHOW_TEXT,{acceptNode:e=>{const t=e.nodeValue||"";return t.includes(n)?NodeFilter.FILTER_ACCEPT:NodeFilter.FILTER_REJECT}});return t=r.nextNode(),t||null}
function r(e,n){const t=w(n.icon),r=f(e,n.label);if(!r)return void e.insertBefore(t,e.firstChild);const a=Array.from(e.querySelectorAll("svg,[aria-hidden='true'],span,i,div")).filter(e=>e!==t&&!e.contains(t)&&!e.contains(r)&&(e.compareDocumentPosition(r)&Node.DOCUMENT_POSITION_FOLLOWING));const s=a.find(e=>e.tagName==="SVG"||e.getAttribute("aria-hidden")==="true"||!o(e));if(s){let n=s.closest("[aria-hidden='true'],span,i,div")||s;(n===e||n.contains(r))&&(n=s),n.replaceWith(t)}else{let n=r.parentElement;for(;n&&n.parentElement!==e;)n=n.parentElement;e.insertBefore(t,n||e.firstChild)}e.querySelectorAll("svg").forEach(e=>{t.contains(e)||e.remove()});const i=f(e,n.label);Array.from(e.querySelectorAll("[aria-hidden='true'],span,i,div")).forEach(e=>{if(e===t||e.contains(t)||t.contains(e)||e.contains(i))return;i&&(e.compareDocumentPosition(i)&Node.DOCUMENT_POSITION_FOLLOWING)&&(!o(e)||/^[+🕙⏰⏱⏲⏳⌛]+$/.test(o(e)))&&e.remove()});const c=[];let l;const u=document.createTreeWalker(e,NodeFilter.SHOW_TEXT,{acceptNode:e=>/[🕙⏰⏱⏲⏳⌛]/.test(e.nodeValue||"")?NodeFilter.FILTER_ACCEPT:NodeFilter.FILTER_REJECT});for(;l=u.nextNode();)c.push(l);c.forEach(e=>e.remove())}
function g(e,n){const t=[];let r;const a=document.createTreeWalker(e,NodeFilter.SHOW_TEXT,{acceptNode:e=>{const n=e.parentElement;if(!n||n.closest("svg,[aria-hidden='true']"))return NodeFilter.FILTER_REJECT;return e.nodeValue&&e.nodeValue.trim()?NodeFilter.FILTER_ACCEPT:NodeFilter.FILTER_REJECT}});for(;r=a.nextNode();)t.push(r);if(t.length)return void t.forEach((e,t)=>{e.nodeValue=t===0?n:""});const s=document.createElement("span");s.textContent=n;e.appendChild(s)}
function a(e){for(const n of ["aria-current","data-current","data-selected","data-active"])e.removeAttribute(n);e.getAttribute("aria-selected")==="true"&&e.setAttribute("aria-selected","false");e.removeAttribute("disabled");e.querySelectorAll("[aria-current]").forEach(e=>e.removeAttribute("aria-current"))}
function u(e,n){const t=e.cloneNode(!0);return a(t),t.dataset.claudePlusEnhance="1",t.dataset.target=n.id,t.dataset.version=v,t.setAttribute("aria-label",n.label),t.setAttribute("title",n.label),t.tagName==="A"?t.setAttribute("href",n.path):t.removeAttribute("href"),t.tagName==="BUTTON"&&t.setAttribute("type","button"),g(t,n.label),r(t,n),t.addEventListener("click",e=>{e.preventDefault(),e.stopPropagation(),s(n)},!0),t}
function h(e){const n=i(e).find(c);if(!n)return!1;const t=n.parentElement||n.parentNode;if(!t||!t.children)return!1;const r=p(),a=Array.from(t.children).filter(e=>d(e)||e.classList?.contains("claude-plus-enhance-nav")),l=r.map(e=>e.id).join("|"),g=a.map(e=>e.dataset.target||"").join("|");if(a.length&&g===l&&a.every(e=>e.dataset.version===v))return!1;b=!0;a.forEach(e=>e.remove());r.slice().reverse().forEach(e=>{t.insertBefore(u(n,e),n.nextSibling)});return b=!1,!0}
function x(){if(b)return;const e=document.getElementById("claude-plus-enhance-style");e&&e.remove();let n=!1;return document.querySelectorAll("nav,aside,[role=navigation]").forEach(e=>{n=h(e)||n}),n}
function y(){b||q||(q=setTimeout(()=>{q=0,x()},250))}
function z(e){return String(e==null?"":e).replace(/[&<>"']/g,e=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[e]))}
function A(){let e=document.getElementById("claude-plus-skills-modal");if(e)return e.remove();e=document.createElement("div");e.id="claude-plus-skills-modal";e.innerHTML='<div class="cps-backdrop"></div><section class="cps-panel" role="dialog" aria-modal="true" aria-label="Claude skills"><header><div><strong>技能</strong><span>全局与历史项目中的本地 Claude skills</span></div><button type="button" data-cps-close>关闭</button></header><main><p class="cps-loading">正在读取 skills...</p></main></section>';document.body.appendChild(e);const n=document.createElement("style");n.id="claude-plus-skills-style";n.textContent="#claude-plus-skills-modal{position:fixed;inset:0;z-index:2147483647;color:#f4f1ea;font:13px/1.45 system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}#claude-plus-skills-modal .cps-backdrop{position:absolute;inset:0;background:rgba(0,0,0,.52)}#claude-plus-skills-modal .cps-panel{position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:min(900px,calc(100vw - 48px));max-height:min(720px,calc(100vh - 48px));display:grid;grid-template-rows:auto 1fr;background:#171717;border:1px solid #3d3a35;border-radius:10px;box-shadow:0 22px 80px rgba(0,0,0,.48);overflow:hidden}#claude-plus-skills-modal header{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:14px 16px;border-bottom:1px solid #2f2d2a;background:#1f1e1b}#claude-plus-skills-modal header div{display:grid;gap:2px}#claude-plus-skills-modal header strong{font-size:16px}#claude-plus-skills-modal header span,.cps-meta,.cps-path,.cps-empty,.cps-loading,.cps-error{color:#a9a39a}#claude-plus-skills-modal button{border:1px solid #5a544b;background:#2b2925;color:#f4f1ea;border-radius:7px;min-height:30px;padding:0 10px;cursor:pointer}#claude-plus-skills-modal button:hover{border-color:#d97745}#claude-plus-skills-modal button.cps-danger{border-color:#7f2d22;background:#4a1f1a;color:#ffd8cf}#claude-plus-skills-modal button:disabled{opacity:.55;cursor:default}#claude-plus-skills-modal main{overflow:auto;padding:14px;display:grid;gap:10px}#claude-plus-skills-modal .cps-card{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:12px;padding:12px;border:1px solid #34312d;border-radius:8px;background:#20201d}#claude-plus-skills-modal .cps-title{display:flex;flex-wrap:wrap;gap:8px;align-items:center;margin-bottom:5px}#claude-plus-skills-modal .cps-title strong{font-size:14px}#claude-plus-skills-modal .cps-badge{font-size:12px;color:#f2d2bd;border:1px solid #7d553d;border-radius:999px;padding:1px 7px;background:#33261f}#claude-plus-skills-modal .cps-summary{margin:0 0 6px;color:#e7e0d4}#claude-plus-skills-modal .cps-path{font-size:12px;word-break:break-all}#claude-plus-skills-modal .cps-actions{display:flex;align-items:flex-start}.cps-toast{position:absolute;right:16px;bottom:14px;background:#2b2925;border:1px solid #5a544b;border-radius:8px;padding:8px 10px;color:#f4f1ea}";document.head.appendChild(n);function t(){e.remove();n.remove()}e.querySelector("[data-cps-close]").addEventListener("click",t);e.querySelector(".cps-backdrop").addEventListener("click",t);return e}
async function B(){const e=A(),n=e.querySelector("main");try{const t=await fetch("http://127.0.0.1:15722/claude-plus/skills",{cache:"no-store"});if(!t.ok)throw new Error("HTTP "+t.status);const r=await t.json(),a=r.skills||[];if(!a.length){n.innerHTML='<p class="cps-empty">没有找到本地 skills。</p>';return}n.innerHTML=a.map(e=>'<article class="cps-card" data-id="'+z(e.id)+'"><div><div class="cps-title"><strong>'+z(e.name)+'</strong><span class="cps-badge">'+z(e.source_label)+'</span></div><p class="cps-summary">'+z(e.summary_zh)+'</p><div class="cps-meta">'+z(e.description||"")+'</div><div class="cps-path">'+z(e.project_path?("项目："+e.project_path):"全局")+'</div><div class="cps-path">'+z(e.path)+'</div></div><div class="cps-actions"><button type="button" class="cps-danger" data-cps-trash>删除</button></div></article>').join("");n.querySelectorAll("[data-cps-trash]").forEach(t=>t.addEventListener("click",async()=>{const r=t.closest(".cps-card"),a=a=>{let n=e.querySelector(".cps-toast");n||(n=document.createElement("div"),n.className="cps-toast",e.appendChild(n));n.textContent=a;setTimeout(()=>n&&n.remove(),2600)},s=r?.dataset.id,l=r?.querySelector(".cps-title strong")?.textContent||"该 skill";if(!s)return;if(!confirm("确认删除 skill “"+l+"”？\n\n该操作会把对应 skill 目录移动到回收站。"))return;t.disabled=!0;try{const e=await fetch("http://127.0.0.1:15722/claude-plus/skills/"+encodeURIComponent(s)+"/trash",{method:"POST"}),n=await e.json().catch(()=>({}));if(!e.ok||n.ok===false)throw new Error(n.error||("HTTP "+e.status));r.remove();a("已移动到回收站")}catch(e){t.disabled=!1;a(e.message||String(e))}}))}catch(t){n.innerHTML='<p class="cps-error">无法连接 Claude++ 本地服务。请先启动 Claude++ 后重试。</p><p class="cps-path">'+z(t.message||String(t))+"</p>"}}
async function s(e){if(e.open==="custom3p"){const n=window["claude.settings"]?.Custom3pSetup?.openSetupWindow||window.claude?.settings?.Custom3pSetup?.openSetupWindow;if(typeof n==="function"){try{await n();return}catch(t){}}return}if(e.open==="skills"){B();return}const n=new URL(e.path,location.origin),t=n.pathname+n.search+n.hash;try{history.pushState(null,"",t);window.dispatchEvent(new PopStateEvent("popstate",{state:history.state}));window.dispatchEvent(new Event("pushstate"));window.dispatchEvent(new Event("locationchange"))}catch(r){location.assign(n.toString())}}
new MutationObserver(y).observe(document.documentElement,{childList:!0,subtree:!0});
document.readyState==="loading"?document.addEventListener("DOMContentLoaded",x,{once:!0}):x();
})();"##;

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
        pub category: &'static str,
        pub label: &'static str,
        pub description: &'static str,
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
        let feature =
            EnhanceFeatureId::parse(feature).ok_or_else(|| format!("未知页面增强项: {feature}"))?;
        if matches!(
            feature,
            EnhanceFeatureId::Markdown | EnhanceFeatureId::Timeline
        ) {
            return Err("该增强项将在下一阶段接入".to_string());
        }
        let paths = resolve_claude_paths()?;
        claude_desktop::stop_claude_processes()?;
        enable_write_access(&paths.resources);

        let mut backup = BackupContext::new(&paths.resources);
        update_feature_marker(&paths.resources, &mut backup, feature, true)?;
        Ok(())
    }

    pub fn uninstall(feature: &str) -> Result<(), String> {
        let feature =
            EnhanceFeatureId::parse(feature).ok_or_else(|| format!("未知页面增强项: {feature}"))?;
        if matches!(
            feature,
            EnhanceFeatureId::Markdown | EnhanceFeatureId::Timeline
        ) {
            return Err("该增强项尚未接入，无需取消".to_string());
        }
        let paths = resolve_claude_paths()?;
        claude_desktop::stop_claude_processes()?;
        enable_write_access(&paths.resources);
        let mut backup = BackupContext::new(&paths.resources);
        update_feature_marker(&paths.resources, &mut backup, feature, false)?;
        Ok(())
    }

    fn feature_list(enabled: &[(EnhanceFeatureId, bool)]) -> Vec<EnhanceFeature> {
        vec![
            EnhanceFeature {
                id: "third_party_api",
                category: "菜单栏增强",
                label: "第三方API",
                description: "在 Claude Desktop 左侧菜单“计划任务”下方增加第三方API快捷入口。",
                enabled: is_enabled(enabled, EnhanceFeatureId::ThirdPartyApi),
                available: true,
                note: "侧边栏软入口",
            },
            EnhanceFeature {
                id: "plugins",
                category: "菜单栏增强",
                label: "技能",
                description: "在 Claude Desktop 左侧菜单中直达技能设置页。",
                enabled: is_enabled(enabled, EnhanceFeatureId::Plugins),
                available: true,
                note: "侧边栏软入口",
            },
            EnhanceFeature {
                id: "mcp",
                category: "菜单栏增强",
                label: "MCP",
                description: "在 Claude Desktop 左侧菜单中直达 MCP 管理页。",
                enabled: is_enabled(enabled, EnhanceFeatureId::Mcp),
                available: true,
                note: "侧边栏软入口",
            },
            EnhanceFeature {
                id: "markdown",
                category: "对话栏增强",
                label: "导出对话为 Markdown",
                description: "在对话页面增加 Markdown 导出入口，把当前对话保存为 Markdown 文件。",
                enabled: is_enabled(enabled, EnhanceFeatureId::Markdown),
                available: true,
                note: "待接入导出逻辑",
            },
            EnhanceFeature {
                id: "timeline",
                category: "状态增强",
                label: "显示对话时间线",
                description: "在对话页面显示问题时间线，方便快速定位上下文进度。",
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
        feature_states_from_text(&text)
    }

    fn feature_states_from_text(text: &str) -> Vec<(EnhanceFeatureId, bool)> {
        [
            EnhanceFeatureId::ThirdPartyApi,
            EnhanceFeatureId::Plugins,
            EnhanceFeatureId::Mcp,
            EnhanceFeatureId::Markdown,
            EnhanceFeatureId::Timeline,
        ]
        .into_iter()
        .map(|feature| (feature, text.contains(&feature_payload(feature.marker()))))
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
            let text =
                fs::read_to_string(&path).map_err(|e| format!("读取 Claude 前端入口失败: {e}"))?;
            let mut next = remove_old_script(&text);
            next = set_marker(next, feature.marker(), enabled);
            next = ensure_script(next);
            if next == text {
                patched = true;
                continue;
            }
            backup.backup_resource(&path)?;
            fs::write(&path, next).map_err(|e| format!("写入 Claude 页面增强入口失败: {e}"))?;
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
        let payload = feature_payload(marker);
        if enabled {
            if !text.contains(marker) {
                text.push_str(&payload);
            }
            return text;
        }
        text.replace(&payload, "")
    }

    fn feature_payload(marker: &str) -> String {
        format!(";window.{marker}=true;")
    }

    #[cfg(test)]
    mod tests {
        use super::{
            feature_payload, feature_states_from_text, EnhanceFeatureId, INJECT_SCRIPT,
            NAV_API_MARKER, NAV_MCP_MARKER, NAV_PLUGINS_MARKER,
        };

        fn state(states: &[(EnhanceFeatureId, bool)], feature: EnhanceFeatureId) -> bool {
            states
                .iter()
                .find_map(|(candidate, enabled)| (*candidate == feature).then_some(*enabled))
                .unwrap_or(false)
        }

        #[test]
        fn script_markers_do_not_count_as_enabled_features() {
            let states = feature_states_from_text(INJECT_SCRIPT);

            assert!(!state(&states, EnhanceFeatureId::ThirdPartyApi));
            assert!(!state(&states, EnhanceFeatureId::Plugins));
            assert!(!state(&states, EnhanceFeatureId::Mcp));
        }

        #[test]
        fn feature_payload_controls_enabled_state() {
            let text = format!(
                "{}{}{}",
                INJECT_SCRIPT,
                feature_payload(NAV_API_MARKER),
                feature_payload(NAV_MCP_MARKER)
            );
            let states = feature_states_from_text(&text);

            assert!(state(&states, EnhanceFeatureId::ThirdPartyApi));
            assert!(!state(&states, EnhanceFeatureId::Plugins));
            assert!(state(&states, EnhanceFeatureId::Mcp));
            assert!(!text.contains(&feature_payload(NAV_PLUGINS_MARKER)));
        }
    }

    fn read_index_bundle(resources_path: &Path) -> Result<String, String> {
        let assets_dir = resources_path.join("ion-dist").join("assets").join("v1");
        let mut output = String::new();
        for path in js_files(&assets_dir, true)? {
            let text =
                fs::read_to_string(&path).map_err(|e| format!("读取 Claude 前端入口失败: {e}"))?;
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
        pub id: &'static str,
        pub label: &'static str,
        pub category: &'static str,
        pub description: &'static str,
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
