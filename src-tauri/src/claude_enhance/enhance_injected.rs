#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EnhanceScriptLocale {
    EnUs,
    ZhCn,
}

impl EnhanceScriptLocale {
    pub(crate) fn from_claude_locale(locale: Option<&str>) -> Self {
        match locale.map(str::trim) {
            Some("zh-CN") => Self::ZhCn,
            _ => Self::EnUs,
        }
    }
}

pub(crate) fn current_script_locale() -> EnhanceScriptLocale {
    EnhanceScriptLocale::from_claude_locale(crate::claude_zh::status().locale.as_deref())
}

pub(crate) fn inject_script_for_locale(locale: EnhanceScriptLocale) -> String {
    inject_script_for_locale_with_tuning(locale, &crate::settings::proxy_runtime_tuning())
}

pub(crate) fn inject_script_for_locale_with_tuning(
    locale: EnhanceScriptLocale,
    tuning: &crate::settings::ProxyRuntimeTuning,
) -> String {
    let script = inject_script_template_with_tuning(tuning);
    match locale {
        EnhanceScriptLocale::ZhCn => script,
        EnhanceScriptLocale::EnUs => english_inject_script(script),
    }
}

fn inject_script_template_with_tuning(tuning: &crate::settings::ProxyRuntimeTuning) -> String {
    let capture = &tuning.token_usage_capture;
    INJECT_SCRIPT_TEMPLATE
        .replace("__CPU_RECENT_LIMIT__", &capture.recent_limit.to_string())
        .replace("__CPU_DEBUG_LIMIT__", &capture.debug_limit.to_string())
        .replace("__CPU_LEDGER_LIMIT__", &capture.ledger_limit.to_string())
        .replace(
            "__CPU_CONTEXT_POLL_INTERVAL_MS__",
            &capture.context_poll_interval_ms.to_string(),
        )
        .replace(
            "__CPU_TURN_IDLE_TIMEOUT_MS__",
            &capture.turn_idle_timeout_ms.to_string(),
        )
        .replace(
            "__CPU_CONTEXT_MERGE_WINDOW_MS__",
            &capture.context_merge_window_ms.to_string(),
        )
        .replace(
            "__CPU_CROSS_SOURCE_DEDUPE_WINDOW_MS__",
            &capture.cross_source_dedupe_window_ms.to_string(),
        )
        .replace(
            "__CPU_FINAL_RENDER_DELAY_MS__",
            &capture.final_render_delay_ms.to_string(),
        )
        .replace(
            "__CPU_MAX_CAPTURE_TEXT_LENGTH__",
            &capture.max_capture_text_length.to_string(),
        )
        .replace(
            "__CPU_MAX_CAPTURE_BLOB_BYTES__",
            &capture.max_capture_blob_bytes.to_string(),
        )
        .replace(
            "__CPU_MAX_COLLECT_DEPTH__",
            &capture.max_collect_depth.to_string(),
        )
}

fn english_inject_script(mut script: String) -> String {
    for (source, target) in [
        (r#"label:"第三方API""#, r#"label:"Third-party API""#),
        (r#"label:"技能""#, r#"label:"Skills""#),
        ("计划任务|Scheduled", "Scheduled tasks|Scheduled"),
        ("button[aria-label*='更多']", "button[aria-label*='more' i]"),
        (
            "新会话|计划任务|第三方API|技能|MCP|自定义|更多|Code|Drag to pin|已固定|最近使用",
            "New chat|Scheduled tasks|Third-party API|Skills|MCP|Customize|More|Code|Drag to pin|Pinned|Recents",
        ),
        (
            r#"aria-label="技能"><header><strong>技能</strong><button type="button" data-cps-close>关闭</button></header><main><p class="cps-loading">正在读取 skills...</p>"#,
            r#"aria-label="Skills"><header><strong>Skills</strong><button type="button" data-cps-close>Close</button></header><main><p class="cps-loading">Loading skills...</p>"#,
        ),
        (r#"n==="global"?"全局 skills":"项目 skills""#, r#"n==="global"?"Global skills":"Project skills""#),
        (r#"e.summary_zh||e.description||"未提供描述。""#, r#"e.description||"No description provided.""#),
        (r#"cpsButton("详情")"#, r#"cpsButton("Details")"#),
        (r#"cpsButton("删除","cps-danger")"#, r#"cpsButton("Delete","cps-danger")"#),
        (r#""原始描述""#, r#""Original description""#),
        (r#""文件地址""#, r#""Skill file""#),
        (r#""目录地址""#, r#""Directory""#),
        (r#""未提供描述。""#, r#""No description provided.""#),
        (r#""暂无"+r+"。""#, r#""No "+r+".""#),
        (
            r#""本地 skills 桥未安装或尚未生效。","请在 Claude++ 中重新安装“技能”页面增强，并重启 Claude Desktop。""#,
            r#""The local skills bridge is not installed or not active yet.","Reinstall the Skills enhancement in Claude++ and restart Claude Desktop.""#,
        ),
        (r#"e.textContent=t?"收起":"详情""#, r#"e.textContent=t?"Hide":"Details""#),
        (r#""该 skill""#, r#""this skill""#),
        (
            r#""确认删除 skill “"+o+"”？\n\n该操作会把对应 skill 目录移动到回收站。""#,
            r#""Delete skill \""+o+"\"?\n\nThis moves the skill directory to the Recycle Bin.""#,
        ),
        (r#"s("已移动到回收站")"#, r#"s("Moved to the Recycle Bin")"#),
        (
            r#""读取本地 skills 失败。""#,
            r#""Failed to read local skills.""#,
        ),
        ("未找到已渲染的对话消息", "No rendered conversation messages found"),
        (
            "_导出范围：当前页面已加载并渲染的对话内容。_",
            "_Export scope: conversation content currently loaded and rendered on this page._",
        ),
        ("已导出当前页面已加载的对话", "Exported the currently loaded conversation"),
        ("导出 Markdown", "Export Markdown"),
        ("跳转到问题 ", "Jump to question "),
        ("本轮调用合计 ", "Turn calls total "),
        (" · 输入 ", " · input "),
        (" · 输出 ", " · output "),
        (" · 缓存写 ", " · cache write "),
        (" · 缓存读 ", " · cache read "),
        (" · 缓存命中率 ", " · cache hit rate "),
        ("return \"上下文 \"+cpuF(r)+\"/\"+cpuF(a)", "return \"context \"+cpuF(r)+\"/\"+cpuF(a)"),
        (" · 调用 ", " · calls "),
        (" 次 · 耗时 ", " · time "),
        (" · 数据仅供参考", " · Data for reference only"),
        ("(估算)", "(estimated)"),
        ("\"未知\"", "\"unknown\""),
        ("运行中|Running|tokens|List all|source code files|正在", "Running|tokens|List all|source code files"),
    ] {
        script = script.replace(source, target);
    }
    script
}

const INJECT_SCRIPT_TEMPLATE: &str = r####";(()=>{const m="__claudePlusEnhanceNavV2";
if(window[m])return;
Object.defineProperty(window,m,{value:!0});
const v="3.9",n=[
{id:"third_party_api",marker:"__claudePlusEnhanceThirdPartyApiV1",label:"第三方API",path:"/setup-desktop-3p",open:"custom3p",icon:'<svg width="16" height="16" style="width:1em;height:1em;min-width:1em;max-width:1em;min-height:1em;max-height:1em;flex:none;display:block" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M8 8h8"/><path d="M8 12h8"/><path d="M8 16h8"/><rect x="4" y="5" width="16" height="14" rx="2"/></svg>'},
{id:"plugins",marker:"__claudePlusEnhancePluginsV1",label:"技能",path:"/customize/plugins/new?marketplace&plugin",open:"skills",icon:'<svg width="16" height="16" style="width:1em;height:1em;min-width:1em;max-width:1em;min-height:1em;max-height:1em;flex:none;display:block" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M7 7h10v10H7z"/><path d="M10 3h4v4"/><path d="M10 21h4v-4"/><path d="M3 10h4"/><path d="M17 14h4"/></svg>'},
{id:"mcp",marker:"__claudePlusEnhanceMcpV1",label:"MCP",path:"/setup-desktop-3p",open:"custom3p_connectors",icon:'<svg width="16" height="16" style="width:1em;height:1em;min-width:1em;max-width:1em;min-height:1em;max-height:1em;flex:none;display:block" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="12" r="2.5"/><circle cx="18" cy="7" r="2.5"/><circle cx="18" cy="17" r="2.5"/><path d="M8.3 10.9 15.7 8.1"/><path d="M8.3 13.1 15.7 15.9"/></svg>'}
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
function j(e){[e,...e.querySelectorAll("a,button,[role=link],[role=button]")].forEach(e=>{["href","target","rel","download","to","data-href","data-to","data-path","data-route"].forEach(n=>e.removeAttribute(n));e.getAttribute("role")==="link"&&e.setAttribute("role","button")})}
function u(e,n){const t=e.cloneNode(!0);return a(t),t.dataset.claudePlusEnhance="1",t.dataset.target=n.id,t.dataset.version=v,t.setAttribute("aria-label",n.label),t.setAttribute("title",n.label),n.open?(j(t),t.setAttribute("role","button"),t.setAttribute("tabindex","0")):t.tagName==="A"?t.setAttribute("href",n.path):t.removeAttribute("href"),t.tagName==="BUTTON"&&t.setAttribute("type","button"),g(t,n.label),r(t,n),t.addEventListener("click",e=>{e.preventDefault(),e.stopImmediatePropagation(),e.stopPropagation(),s(n)},!0),t.addEventListener("keydown",e=>{n.open&&(e.key==="Enter"||e.key===" ")&&(e.preventDefault(),e.stopImmediatePropagation(),s(n))},!0),t}
function h(e){const n=i(e).find(c);if(!n)return!1;const t=n.parentElement||n.parentNode;if(!t||!t.children)return!1;const r=p(),a=Array.from(t.children).filter(e=>d(e)||e.classList?.contains("claude-plus-enhance-nav")),l=r.map(e=>e.id).join("|"),g=a.map(e=>e.dataset.target||"").join("|");if(a.length&&g===l&&a.every(e=>e.dataset.version===v))return!1;b=!0;a.forEach(e=>e.remove());r.slice().reverse().forEach(e=>{t.insertBefore(u(n,e),n.nextSibling)});return b=!1,!0}
function x(){if(b)return;const e=document.getElementById("claude-plus-enhance-style");e&&e.remove();let n=!1;return document.querySelectorAll("nav,aside,[role=navigation]").forEach(e=>{n=h(e)||n}),n}
function E(e){return String(e==null?"":e).replace(/\s+/g," ").trim()}
const H=new Map;
function I(e){return/[A-Za-z]/.test(e)&&!/[\u4e00-\u9fff]/.test(e)&&e.length>=4&&e.length<=90&&!/^(Claude|Claude\\+\\+|New chat|Recents|Scheduled tasks|Projects|Chats|Search chats|Search projects|Cowork|Ctrl\\+B)$/i.test(e)&&!/\\bCtrl\\+/.test(e)}
function ac(e){return e?.closest?.('[role="menu"],[data-radix-popper-content-wrapper],[data-radix-menu-content],[cmdk-list],.claude-plus-markdown-menu-item')}
function J(e){if(!e||ac(e)||e.closest("svg,[aria-hidden='true'],button[aria-label*='more' i],button[aria-label*='更多']"))return null;const n=[];let t;const r=document.createTreeWalker(e,NodeFilter.SHOW_TEXT,{acceptNode:e=>{const n=e.parentElement;if(!n||ac(n)||n.closest("svg,[aria-hidden='true']"))return NodeFilter.FILTER_REJECT;const t=E(e.nodeValue);return I(t)?NodeFilter.FILTER_ACCEPT:NodeFilter.FILTER_REJECT}});for(;t=r.nextNode();)n.push(t);return n.sort((e,n)=>E(n.nodeValue).length-E(e.nodeValue).length)[0]||null}
function N(e){return/^(新会话|计划任务|第三方API|技能|MCP|自定义|更多|Code|Drag to pin|已固定|最近使用)$/i.test(e)||/Ctrl\\+/i.test(e)}
function K(e){const n=e.getAttribute("href")||e.getAttribute("data-href")||e.getAttribute("data-to")||"",t=e.getAttribute("aria-label")||"",r=E(e.textContent),a=new RegExp("(^|/)chat(s)?(/|\\\\?|#|$)|conversation","i"),s=e.closest("aside,nav,[role=navigation]");if(!s||N(r)||e.closest("[data-claude-plus-enhance],#claude-plus-skills-modal")||ac(e))return!1;if(a.test(n)||/open .*chat|open .*conversation|select .*chat|rename chat|打开会话|选择.*会话/i.test(t))return!0;return!!J(e)}
async function L(e,n){const t=E(n.nodeValue);if(!I(t)||e.getAttribute("data-claude-plus-original-title")===t)return;if(H.has(t)){const r=H.get(t);r&&(n.nodeValue=r,e.setAttribute("data-claude-plus-original-title",t),e.setAttribute("data-claude-plus-title-i18n",r));return}e.setAttribute("data-claude-plus-original-title",t);try{const a=window.claudePlusTitleI18n;if(!a||typeof a.translate!=="function"){H.set(t,"");return}const s=E(await a.translate(t));if(s&&s!==t&&/[\u4e00-\u9fff]/.test(s)){H.set(t,s);n.nodeValue=s;e.setAttribute("data-claude-plus-title-i18n",s)}else H.set(t,"")}catch(r){H.set(t,"")}}
function M(){if(!window.__claudePlusEnhanceConversationTitleI18nV1)return;document.querySelectorAll("aside a,nav a,aside button,nav button,aside li,nav li,aside [role=link],nav [role=link],aside [role=button],nav [role=button],aside [role=listitem],nav [role=listitem],aside div[role],nav div[role]").forEach(e=>{if(!K(e))return;const n=J(e);n&&L(e,n)})}
function y(){b||q||(q=setTimeout(()=>{q=0,x();M();P();Y();cpuTick()},250))}
function D(){try{if(localStorage.getItem("claudePlusCustom3pPane")!=="connectors")return}catch(e){return}for(let e=0;e<14;e++)setTimeout(()=>{const e=Array.from(document.querySelectorAll("button,a,[role=button],[role=tab],[role=menuitem],nav *,aside *")).find(e=>/连接器与扩展|Connectors|MCP servers/i.test(o(e)));if(e){e.click();try{localStorage.removeItem("claudePlusCustom3pPane")}catch(t){}}},220+e*250)}
function A(){let e=document.getElementById("claude-plus-skills-modal");if(e)return e.remove();e=document.createElement("div");e.id="claude-plus-skills-modal";e.innerHTML='<div class="cps-backdrop"></div><section class="cps-panel" role="dialog" aria-modal="true" aria-label="技能"><header><strong>技能</strong><button type="button" data-cps-close>关闭</button></header><main><p class="cps-loading">正在读取 skills...</p></main></section>';document.body.appendChild(e);const n=document.createElement("style");n.id="claude-plus-skills-style";n.textContent="#claude-plus-skills-modal{position:fixed;inset:0;z-index:2147483647;color:#f4f1ea;font:13px/1.45 system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}#claude-plus-skills-modal .cps-backdrop{position:absolute;inset:0;background:rgba(0,0,0,.52)}#claude-plus-skills-modal .cps-panel{position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:min(886px,calc(100vw - 48px));height:min(713px,calc(100vh - 48px));display:grid;grid-template-rows:auto 1fr;background:#171717;border:1px solid #3d3a35;border-radius:10px;box-shadow:0 22px 80px rgba(0,0,0,.48);overflow:hidden}#claude-plus-skills-modal header{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:18px 20px 12px;border-bottom:1px solid #2f2d2a;background:#1f1e1b}#claude-plus-skills-modal header strong{font-size:18px;font-weight:650}#claude-plus-skills-modal button{border:1px solid #5a544b;background:#2b2925;color:#f4f1ea;border-radius:7px;min-height:30px;padding:0 10px;cursor:pointer}#claude-plus-skills-modal button:hover{border-color:#d97745}#claude-plus-skills-modal button.cps-danger{border-color:#7f2d22;background:#4a1f1a;color:#ffd8cf}#claude-plus-skills-modal button:disabled{opacity:.55;cursor:default}#claude-plus-skills-modal main{overflow:auto;padding:18px 20px 20px;display:flex;flex-direction:column;gap:18px}#claude-plus-skills-modal .cps-section{display:flex;flex-direction:column;gap:10px}#claude-plus-skills-modal .cps-section-title{font-size:14px;font-weight:650;color:#f4f1ea}#claude-plus-skills-modal .cps-container{display:grid;gap:8px;border:1px solid #34312d;border-radius:8px;background:#1f1f1c;padding:10px}#claude-plus-skills-modal .cps-card{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:12px;padding:10px 12px;border:1px solid #34312d;border-radius:8px;background:#262521}#claude-plus-skills-modal .cps-main{display:flex;min-width:0;flex-direction:column;gap:4px}#claude-plus-skills-modal .cps-name{font-size:14px;font-weight:650;color:#f4f1ea}#claude-plus-skills-modal .cps-brief{color:#e7e0d4}#claude-plus-skills-modal .cps-file,#claude-plus-skills-modal .cps-empty,#claude-plus-skills-modal .cps-loading,#claude-plus-skills-modal .cps-error{color:#a9a39a}#claude-plus-skills-modal .cps-file{font-size:12px;word-break:break-all}#claude-plus-skills-modal .cps-actions{display:flex;align-items:flex-start;gap:8px}#claude-plus-skills-modal .cps-detail{grid-column:1/-1;border-top:1px solid #34312d;margin-top:4px;padding-top:10px;color:#d8d0c4;display:grid;gap:8px}#claude-plus-skills-modal .cps-detail[hidden]{display:none}#claude-plus-skills-modal .cps-detail strong{display:block;color:#f4f1ea;font-size:12px;margin-bottom:2px}.cps-toast{position:absolute;right:16px;bottom:14px;background:#2b2925;border:1px solid #5a544b;border-radius:8px;padding:8px 10px;color:#f4f1ea}";document.head.appendChild(n);function t(){e.remove();n.remove()}e.querySelector("[data-cps-close]").addEventListener("click",t);e.querySelector(".cps-backdrop").addEventListener("click",t);return e}
function cpsText(e,n,t){const r=document.createElement(e);n&&(r.className=n);r.textContent=String(t==null?"":t);return r}
function cpsButton(e,n){const t=document.createElement("button");t.type="button";t.textContent=e;n&&(t.className=n);return t}
function cpsDetailRow(e,n,t){const r=document.createElement("div"),a=document.createElement("strong");a.textContent=e;const s=cpsText("div",t||"",n);r.append(a,s);return r}
function cpsCard(e){const n=document.createElement("article");n.className="cps-card";n.dataset.id=String(e.id||"");const t=document.createElement("div");t.className="cps-main";const r=cpsText("div","cps-name",e.name),a=cpsText("div","cps-brief",e.summary_zh||e.description||"未提供描述。"),s=cpsText("div","cps-file",e.skill_file||e.path);t.append(r,a,s);const l=document.createElement("div");l.className="cps-actions";const i=cpsButton("详情"),m=cpsButton("删除","cps-danger");i.dataset.cpsDetail="";m.dataset.cpsTrash="";l.append(i,m);const d=document.createElement("div");d.className="cps-detail";d.hidden=!0;d.append(cpsDetailRow("原始描述",e.description||"未提供描述。"),cpsDetailRow("文件地址",e.skill_file||e.path,"cps-file"),cpsDetailRow("目录地址",e.path,"cps-file"));n.append(t,l,d);return n}
function cpsSection(e,n){const t=e.filter(e=>e.scope===n),r=n==="global"?"全局 skills":"项目 skills",a=document.createElement("section");a.className="cps-section";a.appendChild(cpsText("div","cps-section-title",r));const s=document.createElement("div");s.className="cps-container";if(t.length)t.forEach(e=>s.appendChild(cpsCard(e)));else s.appendChild(cpsText("p","cps-empty","暂无"+r+"。"));a.appendChild(s);return a}
function cpsRenderSections(e,n){e.replaceChildren(cpsSection(n,"global"),cpsSection(n,"project"))}
function cpsSetStatus(e,n,t){e.replaceChildren(cpsText("p",n,t),...(arguments.length>3?[cpsText("p","cps-path",arguments[3])]:[]))}
async function B(){const e=A(),n=e.querySelector("main"),t=window.claudePlusSkills;if(!t||typeof t.list!=="function"||typeof t.trash!=="function"){cpsSetStatus(n,"cps-error","本地 skills 桥未安装或尚未生效。","请在 Claude++ 中重新安装“技能”页面增强，并重启 Claude Desktop。");return}try{const r=await t.list(),a=r.skills||[];cpsRenderSections(n,a);n.querySelectorAll("[data-cps-detail]").forEach(e=>e.addEventListener("click",()=>{const n=e.closest(".cps-card")?.querySelector(".cps-detail");if(!n)return;const t=n.hasAttribute("hidden");t?n.removeAttribute("hidden"):n.setAttribute("hidden","");e.textContent=t?"收起":"详情"}));n.querySelectorAll("[data-cps-trash]").forEach(r=>r.addEventListener("click",async()=>{const a=r.closest(".cps-card"),s=s=>{let n=e.querySelector(".cps-toast");n||(n=document.createElement("div"),n.className="cps-toast",e.appendChild(n));n.textContent=s;setTimeout(()=>n&&n.remove(),2600)},l=a?.dataset.id,o=a?.querySelector(".cps-name")?.textContent||"该 skill";if(!l)return;if(!confirm("确认删除 skill “"+o+"”？\n\n该操作会把对应 skill 目录移动到回收站。"))return;r.disabled=!0;try{await t.trash(l);a.remove();s("已移动到回收站")}catch(e){r.disabled=!1;s(e.message||String(e))}}))}catch(r){cpsSetStatus(n,"cps-error","读取本地 skills 失败。",r.message||String(r))}}
function O(){let e=document.getElementById("claude-plus-conversation-enhance-style");if(e)return;e=document.createElement("style");e.id="claude-plus-conversation-enhance-style";e.textContent=".claude-plus-markdown-menu-item{color:#f4f1ea!important}.claude-plus-markdown-menu-item:hover{background:rgba(255,255,255,.08)!important}.claude-plus-export-toast{position:fixed;right:22px;top:112px;z-index:2147482001;max-width:min(360px,calc(100vw - 44px));border:1px solid #5a544b;background:#2b2925;color:#f4f1ea;border-radius:8px;padding:8px 10px;font:13px/1.4 system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;box-shadow:0 8px 28px rgba(0,0,0,.24)}.claude-plus-timeline{position:fixed;right:10px;top:18vh;bottom:18vh;width:28px;z-index:2147481999;pointer-events:none}.claude-plus-timeline-track{position:absolute;left:13px;top:0;bottom:0;width:2px;border-radius:999px;background:rgba(214,119,69,.32)}.claude-plus-timeline-marker{position:absolute;left:7px;width:14px;height:14px;border:2px solid #d97745;border-radius:999px;background:#1f1e1b;box-shadow:0 0 0 2px rgba(31,30,27,.9);pointer-events:auto;cursor:pointer}.claude-plus-timeline-marker:hover{background:#d97745}.claude-plus-timeline-tip{position:absolute;right:24px;top:50%;transform:translateY(-50%);display:none;min-width:180px;max-width:min(320px,calc(100vw - 80px));border:1px solid #5a544b;background:#2b2925;color:#f4f1ea;border-radius:8px;padding:8px 10px;font:12px/1.35 system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;box-shadow:0 8px 28px rgba(0,0,0,.24)}.claude-plus-timeline-marker:hover .claude-plus-timeline-tip{display:block}.claude-plus-timeline-target{outline:2px solid #d97745;outline-offset:4px;transition:outline-color .9s ease}.claude-plus-token-usage{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:2px;width:min(592px,calc(100% - 48px));margin:8px auto 12px;border:1px solid rgba(37,99,235,.35);background:rgba(37,99,235,.12);color:inherit;border-radius:7px;padding:6px 10px;font:11.5px/1.35 system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;opacity:.92;letter-spacing:0;text-align:center;word-break:break-word}.claude-plus-token-usage strong{font-weight:650;color:inherit}.claude-plus-token-usage .cpu-line{display:block;max-width:100%}.claude-plus-token-usage .cpu-muted{color:inherit;opacity:.75}main>.claude-plus-token-usage,body>.claude-plus-token-usage{display:none!important}";document.head.appendChild(e)}
function S(e){try{return Array.from(document.querySelectorAll(e))}catch(n){return[]}}
function T(e){const n=e.cloneNode(!0);n.querySelectorAll(".claude-plus-markdown-menu-item,.claude-plus-export-toast,.claude-plus-timeline,button,svg,style,script,textarea,input,nav,aside").forEach(e=>e.remove());return String(n.innerText||n.textContent||"").replace(/\r\n/g,"\n").replace(/\r/g,"\n").replace(/[ \t]+\n/g,"\n").replace(/\n{3,}/g,"\n\n").trim()}
function U(e){return e&&e.nodeType===1&&!e.closest(".claude-plus-markdown-menu-item,.claude-plus-export-toast,.claude-plus-timeline,nav,aside")}
function V(e,n,t,r){if(!U(e))return;const a=e.closest('[data-testid="conversation-turn"],[data-message-author-role],article,section')||e,s=T(a);if(!s||s.length<2)return;if(t.has(a))return;t.set(a);n.push({role:r,a:s,node:a})}
function W(){const e=[],n=new Map;S('[data-message-author-role]').forEach(t=>{const r=String(t.getAttribute("data-message-author-role")||"").toLowerCase();r==="user"?V(t,e,n,"User"):r==="assistant"&&V(t,e,n,"Assistant")});S('[class*="user-message" i],[class*="UserMessage"]').forEach(t=>V(t,e,n,"User"));S('[class*="assistant-message" i],[class*="AssistantMessage"]').forEach(t=>V(t,e,n,"Assistant"));return e.sort((e,n)=>e.node.compareDocumentPosition(n.node)&Node.DOCUMENT_POSITION_PRECEDING?1:-1)}
function X(e){const n=document.createElement("div");n.className="claude-plus-export-toast";n.textContent=e;document.body.appendChild(n);setTimeout(()=>n.remove(),2800)}
function F(e){return String(e||"").replace(/[<>:\"/\\|?*\x00-\x1f]/g," ").replace(/\s+/g," ").trim().slice(0,80)||"Claude conversation"}
function G(){const e=E(document.querySelector("main h1,h1,[data-testid='conversation-title']")?.textContent)||E(document.title).replace(/\s*[-|].*$/,"");return F(e||"Claude conversation")}
function Q(){const e=W();if(!e.length){X("未找到已渲染的对话消息");return}const n=G(),t=["# "+n,"","_导出范围：当前页面已加载并渲染的对话内容。_",""];e.forEach(e=>{t.push("### "+e.role,"",e.a,"")});const r=new Blob([t.join("\n").trimEnd()+"\n"],{type:"text/markdown;charset=utf-8"}),a=URL.createObjectURL(r),s=document.createElement("a");s.href=a;s.download=F(n+" "+new Date().toISOString().slice(0,19).replace(/[:T]/g,"-"))+".md";document.body.appendChild(s);s.click();s.remove();setTimeout(()=>URL.revokeObjectURL(a),1200);X("已导出当前页面已加载的对话")}
function Z(){return!!(document.querySelector('[data-message-author-role],[data-testid="conversation-turn"],[class*="user-message" i],[class*="assistant-message" i]')||/\/chat|\/conversation/i.test(location.pathname))}
function aa(e){const n=E(e.textContent);return e&&e.nodeType===1&&!e.dataset?.claudePlusMarkdownMenuItem&&!e.closest(".claude-plus-markdown-menu-item")&&/(打开于|Open|固定|Pin|标记为未读|Unread|重命名|Rename|分叉|Fork|存档|Archive|删除|Delete)/i.test(n)&&!/(导出 Markdown|Export Markdown)/i.test(n)}
function ab(e){const n=document.createElement("button");n.type="button";n.className=e?.className||"claude-plus-markdown-menu-item";n.classList.add("claude-plus-markdown-menu-item");n.setAttribute("role",e?.getAttribute("role")||"menuitem");n.setAttribute("tabindex",e?.getAttribute("tabindex")||"0");n.dataset.claudePlusMarkdownMenuItem="1";n.textContent="导出 Markdown";n.addEventListener("click",e=>{e.preventDefault();e.stopPropagation();e.stopImmediatePropagation?.();Q()},!0);n.addEventListener("keydown",e=>{(e.key==="Enter"||e.key===" ")&&(e.preventDefault(),Q())},!0);return n}
function P(){document.querySelectorAll(".claude-plus-markdown-export").forEach(e=>e.remove());if(!window.__claudePlusEnhanceMarkdownExportV1)return;O();document.querySelectorAll('[role="menu"],[data-radix-menu-content]').forEach(e=>{const n=Array.from(e.querySelectorAll("[data-claude-plus-markdown-menu-item]"));n.slice(1).forEach(e=>e.remove());if(n.length)return;const t=Array.from(e.querySelectorAll('button,[role="menuitem"],[cmdk-item]')).filter(aa);if(t.length<3)return;const r=t.find(e=>/存档|Archive/i.test(E(e.textContent)))||t.find(e=>/删除|Delete/i.test(E(e.textContent)));if(!r)return;const a=ab(r);r.parentElement?r.parentElement.insertBefore(a,r):e.appendChild(a)})}
function Y(){const e=document.querySelector(".claude-plus-timeline");if(!window.__claudePlusEnhanceTimelineV1){e&&e.remove();return}O();const n=W().filter(e=>e.role==="User").slice(0,40);if(!n.length){e&&e.remove();return}const t=n.map(e=>e.a.slice(0,80)).join("|");if(e&&e.dataset.signature===t)return;e&&e.remove();const r=document.createElement("div");r.className="claude-plus-timeline";r.dataset.signature=t;r.innerHTML='<div class="claude-plus-timeline-track"></div>';n.forEach((e,t)=>{const a=document.createElement("button");a.type="button";a.className="claude-plus-timeline-marker";a.style.top=(n.length===1?50:2+t*(96/(n.length-1)))+"%";a.setAttribute("aria-label","跳转到问题 "+(t+1));const s=document.createElement("span");s.className="claude-plus-timeline-tip";s.textContent=(t+1)+". "+e.a.replace(/\s+/g," ").slice(0,60);a.appendChild(s);a.addEventListener("click",n=>{n.preventDefault();n.stopPropagation();e.node.scrollIntoView({behavior:"smooth",block:"center"});e.node.classList.add("claude-plus-timeline-target");setTimeout(()=>e.node.classList.remove("claude-plus-timeline-target"),1300)},!0);r.appendChild(a)});document.body.appendChild(r)}
const CPU_RECENT_LIMIT=__CPU_RECENT_LIMIT__;
const CPU_DEBUG_LIMIT=__CPU_DEBUG_LIMIT__;
const CPU_LEDGER_LIMIT=__CPU_LEDGER_LIMIT__;
const CPU_CONTEXT_POLL_INTERVAL_MS=__CPU_CONTEXT_POLL_INTERVAL_MS__;
const CPU_TURN_IDLE_TIMEOUT_MS=__CPU_TURN_IDLE_TIMEOUT_MS__;
const CPU_CONTEXT_MERGE_WINDOW_MS=__CPU_CONTEXT_MERGE_WINDOW_MS__;
const CPU_CROSS_SOURCE_DEDUPE_WINDOW_MS=__CPU_CROSS_SOURCE_DEDUPE_WINDOW_MS__;
const CPU_FINAL_RENDER_DELAY_MS=__CPU_FINAL_RENDER_DELAY_MS__;
const CPU_MAX_CAPTURE_TEXT_LENGTH=__CPU_MAX_CAPTURE_TEXT_LENGTH__;
const CPU_MAX_CAPTURE_BLOB_BYTES=__CPU_MAX_CAPTURE_BLOB_BYTES__;
const CPU_MAX_COLLECT_DEPTH=__CPU_MAX_COLLECT_DEPTH__;
const cpu={last:null,lastId:0,polling:!1,pollBusy:!1,lastPollAt:0,pending:!1,seq:0,turnSeq:0,lastProxyId:0,currentTurn:null,recent:[],ledger:[],debug:[],byScope:Object.create(null),activeProjectId:"",activeConversationId:"",contextPollTimer:0,renderTimer:0,renderReady:!1,wasBusy:!1};
window.__claudePlusTokenUsageDebug=cpu.debug;
function cpuN(e){const n=Number(e);return Number.isFinite(n)&&n>=0?Math.round(n):0}
function cpuF(e){return cpuN(e).toLocaleString("en-US")}
function cpuPct(e,n){return n?((e/n)*100).toFixed(1)+"%":""}
function cpuNormId(e){return String(e||"").trim().replace(/^\/+|\/+$/g,"").slice(0,120)}
function cpuProjectFromLocation(){const e=String(location.pathname||"").match(/\/(?:project|projects)\/([^/?#]+)/i);return cpuNormId(e&&e[1])}
function cpuConversationFromLocation(){const e=String(location.pathname||"").match(/\/(?:chat|chats|conversation|conversations|thread|threads)\/([^/?#]+)/i);return cpuNormId(e&&e[1])}
function cpuAttrId(e,n){for(const t of n){const n=e?.getAttribute?.(t)||e?.dataset?.[t.replace(/^data-/,'').replace(/-([a-z])/g,(e,n)=>n.toUpperCase())];if(n)return cpuNormId(n)}return""}
function cpuProjectFromDom(){const e=document.querySelector('[data-project-id],[data-projectid],[data-testid*="project" i][data-id],a[href*="/project"],a[href*="/projects"]');return cpuAttrId(e,["data-project-id","data-projectid","data-id"])||cpuNormId(String(e?.getAttribute?.("href")||"").match(/\/(?:project|projects)\/([^/?#]+)/i)?.[1])}
function cpuConversationFromDom(){const e=document.querySelector('[data-conversation-id],[data-conversationid],[data-thread-id],[data-threadid],[data-testid*="conversation" i][data-id],[data-testid*="chat" i][data-id],a[href*="/chat"],a[href*="/conversation"],a[href*="/thread"]');return cpuAttrId(e,["data-conversation-id","data-conversationid","data-thread-id","data-threadid","data-id"])||cpuNormId(String(e?.getAttribute?.("href")||"").match(/\/(?:chat|chats|conversation|conversations|thread|threads)\/([^/?#]+)/i)?.[1])}
function cpuCurrentProjectId(){const e=cpuProjectFromDom()||cpuProjectFromLocation();return e||(cpu.activeProjectId||"")}
function cpuCurrentConversationId(){const e=cpuConversationFromDom()||cpuConversationFromLocation();return e||(cpu.activeConversationId||"")}
function cpuScopeKey(e,n){const t=cpuNormId(e),r=cpuNormId(n);return(t||"default-project")+":"+(r||"default-conversation")}
function cpuCurrentScopeKey(){return cpuScopeKey(cpuCurrentProjectId(),cpuCurrentConversationId())}
function cpuStampScope(e){const n=cpuCurrentProjectId(),t=cpuCurrentConversationId(),r=cpuScopeKey(n,t);cpu.activeProjectId=n;cpu.activeConversationId=t;return Object.assign(e||{},{projectId:n,conversationId:t,scopeKey:r})}
function cpuPublicTurn(){const e=cpu.currentTurn;return e?{id:e.id,turnId:e.id,callCount:e.calls.length,startedAt:e.startedAt,projectId:e.projectId,conversationId:e.conversationId,scopeKey:e.scopeKey,calls:e.calls.slice()}:null}
function cpuExport(){return{last:cpu.last,currentTurn:cpuPublicTurn(),recent:cpu.recent.slice(),debug:cpu.debug.slice(),ledgerEvents:cpu.ledger.slice(),activeProjectId:cpuCurrentProjectId(),activeConversationId:cpuCurrentConversationId(),activeScopeKey:cpuCurrentScopeKey()}}
function cpuPublish(){window.__claudePlusTokenUsageDebug=cpu.debug.slice();window.__claudePlusTokenUsage={last:cpu.last,currentTurn:cpuPublicTurn(),recent:cpu.recent.slice(),debug:cpu.debug.slice(),export:()=>cpuExport()}}
function cpuClear(){cpu.last=null;cpu.lastId=0;cpu.renderReady=!1;clearTimeout(cpu.renderTimer);cpuPublish();cpuRender()}
function cpuHas(e,...n){return n.some(n=>e&&e[n]!=null)}
function cpuNestedHas(e,n,t){return e&&e[n]&&e[n][t]!=null}
function cpuNormalizeUsage(e){if(!e||typeof e!=="object")return null;const t=cpuN(e.input_tokens??e.inputTokens??e.prompt_tokens??e.promptTokens),n=cpuN(e.output_tokens??e.outputTokens??e.completion_tokens??e.completionTokens),r=e.total_tokens??e.totalTokens??e.used_tokens??e.usedTokens??e.used,a=r==null&&!!(t||n),s=cpuN(r??t+n),l=e.cache_read_known??e.cacheReadKnown,o=l!=null?!!l:cpuHas(e,"cache_read_input_tokens","cacheReadInputTokens")||cpuNestedHas(e,"prompt_tokens_details","cached_tokens")||cpuNestedHas(e,"promptTokensDetails","cachedTokens")||cpuNestedHas(e,"input_tokens_details","cached_tokens")||cpuNestedHas(e,"inputTokensDetails","cachedTokens"),i=cpuN(e.cache_read_input_tokens??e.cacheReadInputTokens??(l!=null&&o?e.cachedTokens:void 0)??e.prompt_tokens_details?.cached_tokens??e.promptTokensDetails?.cachedTokens??e.input_tokens_details?.cached_tokens??e.inputTokensDetails?.cachedTokens),m=e.cache_creation_known??e.cacheCreationKnown,d=m!=null?!!m:cpuHas(e,"cache_creation_input_tokens","cacheCreationInputTokens"),u=cpuN(e.cache_creation_input_tokens??e.cacheCreationInputTokens??(m!=null&&d?e.cacheCreationTokens:void 0)),p=cpuN(e.context_used??e.contextUsed??e.used_tokens??e.usedTokens??e.used),g=cpuN(e.context_limit??e.contextLimit??e.model_context_window??e.modelContextWindow??e.context_window??e.contextWindow??e.limit),h=cpuN(e.call_count??e.callCount);if(!t&&!n&&!s&&!i&&!u&&!g)return null;return{id:cpuN(e.id),input:t,output:n,total:s,cached:0,cacheReadTokens:i,cacheCreationTokens:u,cachedReadTokens:i,cacheCreated:u,cacheReadKnown:o,cacheCreationKnown:d,contextUsed:p||s,contextLimit:g,elapsed:cpuN(e.elapsedMs??e.elapsed_ms),updatedAt:cpuN(e.updatedAtMs??e.updated_at_ms),callCount:h,totalEstimated:a,source:e.source||""}}
function cpuCollectUsages(e,t=0,n=[]){if(!e||t>CPU_MAX_COLLECT_DEPTH)return n;if(Array.isArray(e)){e.slice(0,100).forEach(e=>cpuCollectUsages(e,t+1,n));return n}if(typeof e!=="object")return n;for(const r of["usage","token_usage","tokenUsage","last","lastUsage","last_token_usage","lastTokenUsage"]){const t=cpuNormalizeUsage(e[r]);t&&n.push(t)}const r=cpuNormalizeUsage(e);if(r){n.push(r);return n}for(const r of["response","data","body","message","result","event","params","context_usage","contextUsage","info","completion","delta"])cpuCollectUsages(e[r],t+1,n);return n}
function cpuCaptureText(e){const n=String(e||"");return n.length>CPU_MAX_CAPTURE_TEXT_LENGTH?n.slice(0,CPU_MAX_CAPTURE_TEXT_LENGTH):n}
function cpuExtractUsages(e){if(typeof e==="string"){const t=[],r=cpuCaptureText(e);try{cpuCollectUsages(JSON.parse(r),0,t)}catch(n){}r.split(/\r?\n/).map(e=>e.trim()).filter(e=>e.startsWith("data:")).map(e=>e.slice(5).trim()).filter(e=>e&&e!=="[DONE]").slice(0,200).forEach(e=>{try{cpuCollectUsages(JSON.parse(e),0,t)}catch(n){}});return t}return cpuCollectUsages(e)}
function cpuMap(e){return cpuNormalizeUsage(e)}
function cpuBeginTurn(){const e=Date.now(),n=cpuCurrentProjectId(),t=cpuCurrentConversationId(),r=cpuScopeKey(n,t);cpu.currentTurn={id:++cpu.turnSeq,calls:[],startedAt:e,lastUpdatedAt:e,elapsed:0,projectId:n,conversationId:t,scopeKey:r};cpu.lastProxyId=0;cpu.pending=!0;cpuClear();return cpu.currentTurn}
function cpuEnsureTurn(){const e=Date.now(),n=cpuCurrentScopeKey();return!cpu.currentTurn||cpu.currentTurn.calls.length&&e-cpu.currentTurn.lastUpdatedAt>CPU_TURN_IDLE_TIMEOUT_MS||cpu.currentTurn.scopeKey!==n?cpuBeginTurn():cpu.currentTurn}
function cpuSameUsage(e,n){if(!e||!n)return!1;if(e.scopeKey&&n.scopeKey&&e.scopeKey!==n.scopeKey)return!1;const t=Math.abs((e.observedAt||e.updatedAt||0)-(n.observedAt||n.updatedAt||0));if(t>CPU_CROSS_SOURCE_DEDUPE_WINDOW_MS)return!1;return e.total===n.total&&e.input===n.input&&e.output===n.output&&e.cachedReadTokens===n.cachedReadTokens&&e.cacheCreationTokens===n.cacheCreationTokens}
function cpuKnownAdd(e,n,t){return e||t?e+n:e}
function cpuAggregateTurn(){const e=cpuEnsureTurn(),n=e.calls.reduce((e,n)=>({id:e.id,input:e.input+n.input,output:e.output+n.output,total:e.total+n.total,cached:0,cacheReadTokens:cpuKnownAdd(e.cacheReadTokens,n.cacheReadTokens,n.cacheReadKnown),cacheCreationTokens:cpuKnownAdd(e.cacheCreationTokens,n.cacheCreationTokens,n.cacheCreationKnown),cachedReadTokens:cpuKnownAdd(e.cachedReadTokens,n.cachedReadTokens,n.cacheReadKnown),cacheCreated:cpuKnownAdd(e.cacheCreated,n.cacheCreated,n.cacheCreationKnown),cacheReadKnown:e.cacheReadKnown||!!n.cacheReadKnown,cacheCreationKnown:e.cacheCreationKnown||!!n.cacheCreationKnown,contextUsed:Math.max(e.contextUsed,n.contextUsed),contextLimit:Math.max(e.contextLimit,n.contextLimit),elapsed:Math.max(e.elapsed,n.elapsed),updatedAt:Date.now(),callCount:e.callCount+(n.callCount||1),totalEstimated:e.totalEstimated||!!n.totalEstimated,projectId:e.projectId,conversationId:e.conversationId,scopeKey:e.scopeKey}),{id:e.id,input:0,output:0,total:0,cached:0,cacheReadTokens:0,cacheCreationTokens:0,cachedReadTokens:0,cacheCreated:0,cacheReadKnown:!1,cacheCreationKnown:!1,contextUsed:0,contextLimit:0,elapsed:e.elapsed||0,updatedAt:0,callCount:0,totalEstimated:!1,projectId:e.projectId,conversationId:e.conversationId,scopeKey:e.scopeKey});return n}
function cpuRememberUsage(e,t,n){const r=cpu.currentTurn;if(!r)return!1;const a=cpuStampScope({...e,source:n||"",elapsed:t||e.elapsed,updatedAt:Date.now(),observedAt:Date.now()});if(r.scopeKey!==a.scopeKey)return!1;const weak=/^(cc-switch|proxy)$/.test(a.source||"");if(cpu.renderReady&&weak)return!1;if(weak&&r.calls.some(e=>!/^(cc-switch|proxy)$/.test(e.source||"")))return!1;if(!weak)r.calls=r.calls.filter(e=>!/^(cc-switch|proxy)$/.test(e.source||""));const s=r.calls.find(e=>cpuSameUsage(e,a));if(s)return cpuScheduleFinalRender(),!1;r.calls.push({...a,id:++cpu.seq});r.lastUpdatedAt=Date.now();r.elapsed=Math.max(r.elapsed||0,t||0);const l=cpuAggregateTurn();cpu.lastId=l.id;cpu.last=l;cpu.byScope[l.scopeKey]=l;cpu.pending=!1;cpu.renderReady=!1;cpu.recent=[l,...cpu.recent.filter(e=>!cpuSameUsage(e,l))].slice(0,CPU_RECENT_LIMIT);cpu.ledger.push({...a,turnId:r.id});cpu.ledger=cpu.ledger.slice(-CPU_LEDGER_LIMIT);cpuPublish();cpuScheduleFinalRender();return!0}
function cpuRemember(e,t,n){const r=cpuExtractUsages(e);if(!r.length)return!1;r.forEach(e=>cpuRememberUsage(e,t,n));return!0}
function cpuPayload(e,t,n){try{return cpuRemember(e,t,n)}catch(r){cpu.debug.unshift({at:new Date().toISOString(),source:n||"",error:String(r?.message||r)});cpu.debug=cpu.debug.slice(0,CPU_DEBUG_LIMIT);cpuPublish();return!1}}
function cpuContextReading(){const e=window.__codexContextMeter;if(!e)return null;const n=e.last||e.current||e.state||e.captureState||e;if(!n||typeof n!=="object")return null;return cpuNormalizeUsage({contextUsed:n.usedTokens??n.used??n.contextUsed??n.context_used??n.totalTokens??n.total_tokens,contextLimit:n.limit??n.contextLimit??n.context_limit??n.modelContextWindow??n.model_context_window,totalTokens:n.totalTokens??n.total_tokens})}
function cpuMergeContext(){const e=cpuContextReading();if(!e)return;const n=Date.now();if(cpu.last&&n-(cpu.last.updatedAt||0)<=CPU_CONTEXT_MERGE_WINDOW_MS){cpu.last={...cpu.last,contextUsed:e.contextUsed||cpu.last.contextUsed,contextLimit:e.contextLimit||cpu.last.contextLimit};cpu.byScope[cpu.last.scopeKey]=cpu.last;cpuPublish();cpuScheduleFinalRender()}}
function cpuInstallContextMeterObserver(){if(window.__claudePlusTokenUsageContextObserver)return;const poll=()=>{try{const meter=window.__codexContextMeter,captureState=meter&&meter.captureState;if(captureState&&captureState.__claudePlusTokenUsageWrapped!==!0){const originalInspectText=captureState.inspectText,originalInspectValue=captureState.inspectValue;typeof originalInspectText==="function"&&(captureState.inspectText=function captureStateInspectText(...args){try{cpuPayload(args[0],0,"context-capture")}catch(err){}return originalInspectText.apply(this,args)});typeof originalInspectValue==="function"&&(captureState.inspectValue=function captureStateInspectValue(...args){try{cpuPayload(args[0],0,"context-value")}catch(err){}return originalInspectValue.apply(this,args)});captureState.__claudePlusTokenUsageWrapped=!0}cpuMergeContext()}catch(err){}};poll();cpu.contextPollTimer=setInterval(poll,CPU_CONTEXT_POLL_INTERVAL_MS);window.__claudePlusTokenUsageContextObserver=!0}
function cpuScheduleFinalRender(){clearTimeout(cpu.renderTimer);cpu.renderTimer=setTimeout(()=>cpuFinalizeTurnRender(),CPU_FINAL_RENDER_DELAY_MS)}
function cpuFinalizeTurnRender(){if(!cpu.last)return;if(cpuBusy()){cpuScheduleFinalRender();return}cpu.renderReady=!0;cpuRender()}
function cpuApiUrl(e){const t=String(e||"");return!/\/claude-plus\/token-usage\b/i.test(t)&&(/\/(responses|chat\/completions|conversation|thread|api|claude-desktop)\b|codex|claude/i.test(t))}
function cpuReqUrl(e){return typeof e==="string"?e:e?.url?e.url:String(e||"")}
function cpuInstallFetchObserver(){if(typeof window.fetch!=="function"||window.fetch.__claudePlusTokenUsageWrapped)return;const e=window.fetch.__claudePlusTokenUsageOriginal||window.fetch,n=e.bind(window);window.fetch=function(e,r){const a=cpuReqUrl(e),s=performance.now(),l=cpuApiUrl(a)&&!!cpu.currentTurn;l&&(cpu.pending=!0);return n(e,r).then(response=>(l&&response?.clone&&response.clone().text().then(e=>cpuPayload(e,performance.now()-s,a)).catch(()=>{}),response))};window.fetch.__claudePlusTokenUsageOriginal=e;window.fetch.__claudePlusTokenUsageWrapped=!0}
function cpuInstallXhrObserver(){const e=window.XMLHttpRequest;if(!e||e.prototype.__claudePlusTokenUsageWrapped)return;const t=e.prototype.open,n=e.prototype.send;e.prototype.open=function(e,n,...r){this.__claudePlusTokenUsageUrl=n;return t.call(this,e,n,...r)};XMLHttpRequest.prototype.send=function(...e){const t=performance.now(),r=this.__claudePlusTokenUsageUrl,a=cpuApiUrl(r)&&!!cpu.currentTurn;a&&(cpu.pending=!0);this.addEventListener?.("loadend",()=>{if(!a)return;try{cpuPayload(this.responseText||"",performance.now()-t,r)}catch(e){}});return n.apply(this,e)};e.prototype.__claudePlusTokenUsageWrapped=!0}
function cpuInstallWebSocketObserver(){if(typeof window.WebSocket!=="function"||window.__claudePlusTokenUsageWebSocketWrapped)return;const NativeWebSocket=window.__claudePlusTokenUsageNativeWebSocket||window.WebSocket;function T(...e){const t=new NativeWebSocket(...e);t.addEventListener?.("message",e=>{try{typeof e.data==="string"?cpuPayload(e.data,0,"websocket"):e.data instanceof Blob&&e.data.size<=CPU_MAX_CAPTURE_BLOB_BYTES&&e.data.text().then(e=>cpuPayload(e,0,"websocket")).catch(()=>{})}catch(t){}});return t}T.prototype=NativeWebSocket.prototype;window.WebSocket=T;window.__claudePlusTokenUsageNativeWebSocket=NativeWebSocket;window.__claudePlusTokenUsageWebSocketWrapped=!0}
function cpuInstallPostMessageObserver(){if(window.__claudePlusTokenUsageMessageObserver)return;window.addEventListener?.("message",e=>{try{cpuPayload(e.data,0,"post-message")}catch(t){}},!0);window.__claudePlusTokenUsageMessageObserver=!0}
function cpuCacheText(e,n){return e?cpuF(n):"未知"}
function cpuRateText(e,n,t,o){const r=t+n+(o||0);return e&&r?cpuPct(Math.min(n,r),r):"未知"}
function cpuContextText(e){const r=e.contextUsed||e.total||0,a=e.contextLimit||0;if(!a)return "";return "上下文 "+cpuF(r)+"/"+cpuF(a)+" ("+cpuPct(r,a)+")"}
function cpuRenderBadge(e,n){const t=e.input||0,r=e.cachedReadTokens||e.cacheReadTokens||0,a=e.totalEstimated?"(估算)":"",s=cpuContextText(e),l=document.createElement("span");l.className="cpu-line";l.append("本轮调用合计 ");const i=document.createElement("strong");i.textContent=cpuF(e.total)+a;l.append(i," · 输入 "+cpuF(e.input)+" · 输出 "+cpuF(e.output)+" · 缓存写 "+cpuCacheText(e.cacheCreationKnown,e.cacheCreationTokens)+" · 缓存读 "+cpuCacheText(e.cacheReadKnown,r)+" · 缓存命中率 "+cpuRateText(e.cacheReadKnown,r,t,e.cacheCreationTokens||0)+(s?" · "+s:"")+" · 调用 "+cpuF(e.callCount)+" 次 · 耗时 "+((e.elapsed||0)/1000).toFixed(1)+"s · 数据仅供参考");n.replaceChildren(l)}
function cpuRect(e){if(!(e instanceof Element))return null;const n=e.getBoundingClientRect();return n.width||n.height?n:null}
function cpuAction(e){if(!(e instanceof Element))return!1;const n=e.getAttribute("aria-label")||"";return/^(复制|喜欢|不喜欢|从此处开始分叉|Copy|Good response|Bad response|Branch from here)$/i.test(n)}
function cpuLooksLikeRunStatus(e){const n=E(e?.textContent||"");return/运行中|Running|tokens|List all|source code files|正在/i.test(n)&&!/复制|Copy|Good response|Bad response|喜欢|不喜欢/i.test(n)}
function cpuBusy(){return Array.from(document.querySelectorAll("button,[role=button]")).some(e=>{const n=(e.getAttribute("aria-label")||e.textContent||"").trim();return/^(停止|停止生成|Stop|Stop generating)$/i.test(n)})}
function cpuEdit(e){return!!(e&&(e.tagName==="TEXTAREA"||e.tagName==="INPUT"||e.isContentEditable||e.closest?.("textarea,input,[contenteditable='true']")))}
function cpuSend(e){const n=e.target;if(e.type==="submit")return!0;if(e.type==="keydown")return e.key==="Enter"&&!e.shiftKey&&cpuEdit(n);if(e.type==="click"){const e=(n?.getAttribute?.("aria-label")||n?.closest?.("button,[role=button]")?.getAttribute?.("aria-label")||n?.textContent||"").trim();return/^(发送|提交|Send|Submit)$|send|submit/i.test(e)}return!1}
function cpuStart(e){if(cpuSend(e)){cpuBeginTurn();cpuPoll(!0)}}
function cpuContainer(e){let n=null,t=-1;for(let r=e;r&&r!==document.body;r=r.parentElement){if(cpuLooksLikeRunStatus(r))continue;const a=cpuRect(r),s=String(r.className||""),l=r.innerText||r.textContent||"";if(a&&a.width>=220&&a.height>=32&&!r.querySelector("textarea,input,[contenteditable='true']")&&!/thread-scroll-container|main-surface|app-shell|timeline/i.test(s)&&l.trim().length>=2){let e=0;r.querySelector("button[aria-label='复制'],button[aria-label='Copy']")&&(e+=6);r.querySelector("button[aria-label='喜欢'],button[aria-label='不喜欢'],button[aria-label='Good response'],button[aria-label='Bad response']")&&(e+=3);r.querySelector("p,li,pre,code,table")&&(e+=2);/group flex min-w-0 flex-col/.test(s)&&(e+=5);e-=Math.max(0,l.length/2000);if(e>t){n=r;t=e}if(e>=10)break}}return t>0?n:null}
function cpuLatestAssistant(){const e=Array.from(document.querySelectorAll("button")).filter(cpuAction);for(let n=e.length-1;n>=0;n--){const t=cpuContainer(e[n]);if(t)return t}const n=W().filter(e=>e.role==="Assistant").map(e=>e.node).filter(e=>e instanceof Element&&!cpuLooksLikeRunStatus(e));if(n.length)return n[n.length-1];for(const t of ['[data-message-author-role="assistant"]','[data-testid*="assistant"]',"main article","main section","main [class*='assistant' i]"])try{const e=Array.from(document.querySelectorAll(t)).filter(e=>e instanceof Element&&!e.querySelector("textarea,input,[contenteditable='true']")&&!cpuLooksLikeRunStatus(e));if(e.length)return e[e.length-1]}catch(r){}return null}
function cpuMount(e){const n=e.closest('[data-testid="conversation-turn"],[data-message-author-role],article,section')||e,t=n.parentElement;if(!t||t===document.body||t===document.documentElement)return e;return n}
function cpuAssistantFooter(e){const n=cpuMount(e),t=Array.from(n.querySelectorAll("button,[role=button]")).filter(cpuAction).pop();if(!t)return null;for(let e=t;e&&e!==n;e=e.parentElement){const t=e.parentElement;if(!t)break;const r=Array.from(t.querySelectorAll("button,[role=button]")).filter(cpuAction);if(r.length)return t}return t.parentElement||null}
function cpuInsertAfter(e,n){const t=n.parentElement;t&&t.insertBefore(e,n.nextSibling)}
function cpuRender(){document.querySelectorAll("main>.claude-plus-token-usage,body>.claude-plus-token-usage").forEach(e=>e.remove());let e=document.querySelector(".claude-plus-token-usage");if(!window.__claudePlusEnhanceTokenUsageV1){e&&e.remove();return}if(!cpu.last||!cpu.renderReady){e&&e.remove();return}O();const n=cpuLatestAssistant();if(!n||cpuLooksLikeRunStatus(n)){e&&e.remove();return}const t=cpuMount(n),r=t.parentElement;if(!r)return;const a=cpuAssistantFooter(t);const s=a?.parentElement||r;e&&e.dataset.host!==String(cpu.lastId)&&(e.remove(),e=null);e||(e=document.createElement("div"),e.className="claude-plus-token-usage");e.dataset.host=String(cpu.lastId);e.dataset.scopeKey=cpu.last.scopeKey||"";cpuRenderBadge(cpu.last,e);e.parentElement!==s||e.previousElementSibling!==(a||t)?s.insertBefore(e,a?a.nextSibling:t.nextSibling):0;document.querySelectorAll(".claude-plus-token-usage").forEach(n=>{n!==e&&n.remove()})}
async function cpuPoll(e){if(!window.__claudePlusEnhanceTokenUsageV1){cpuRender();return}const n=Date.now(),a=cpu.currentTurn?.startedAt||0;if(!a)return;if(cpu.pollBusy||(!e&&n-cpu.lastPollAt<350))return;cpu.pollBusy=!0;cpu.lastPollAt=n;try{const e=window.claudePlusTokenUsage,s={sinceMs:a},r=e&&typeof e.get==="function"?await e.get(s):null,t=cpuMap(r&&r.usage);if(!t)return;const o=t.id||t.updatedAt||0;if(o&&o===cpu.lastProxyId){cpuScheduleFinalRender();return}cpu.lastProxyId=o||Date.now();cpuRememberUsage(t,t.elapsed,t.source||"proxy")}catch(e){}finally{cpu.pollBusy=!1}}
function cpuInstallObservers(){if(!window.__claudePlusEnhanceTokenUsageV1)return;cpuInstallFetchObserver();cpuInstallXhrObserver();cpuInstallWebSocketObserver();cpuInstallPostMessageObserver();cpuInstallContextMeterObserver();["submit","click","keydown"].forEach(e=>document.addEventListener(e,cpuStart,!0))}
function cpuTick(){if(!window.__claudePlusEnhanceTokenUsageV1){cpuRender();return}cpuStampScope({});const busy=cpuBusy();if(busy){if(!cpu.currentTurn)cpuBeginTurn();cpuPoll(!0);cpu.wasBusy=!0;return}if(cpu.wasBusy&&!busy)cpuScheduleFinalRender();cpu.wasBusy=busy;cpu.currentTurn&&cpuPoll();cpuMergeContext();if(!cpu.polling){cpu.polling=!0;setInterval(()=>cpu.currentTurn&&cpuPoll(!0),1200)}}
cpuPublish();async function s(e){if(e.open==="custom3p"||e.open==="custom3p_connectors"){const n=window["claude.settings"]?.Custom3pSetup?.openSetupWindow||window.claude?.settings?.Custom3pSetup?.openSetupWindow;if(typeof n==="function"){try{e.open==="custom3p_connectors"&&localStorage.setItem("claudePlusCustom3pPane","connectors");await n();return}catch(t){}}return}if(e.open==="skills"){B();return}const n=new URL(e.path,location.origin),t=n.pathname+n.search+n.hash;try{history.pushState(null,"",t);window.dispatchEvent(new PopStateEvent("popstate",{state:history.state}));window.dispatchEvent(new Event("pushstate"));window.dispatchEvent(new Event("locationchange"))}catch(r){location.assign(n.toString())}}
cpuInstallObservers();
new MutationObserver(y).observe(document.documentElement,{childList:!0,subtree:!0});
document.readyState==="loading"?document.addEventListener("DOMContentLoaded",()=>{x();M();P();Y();cpuTick()},{once:!0}):(x(),M(),P(),Y(),cpuTick());
window.addEventListener("resize",()=>{Y()},{passive:!0});
D();
})();"####;
