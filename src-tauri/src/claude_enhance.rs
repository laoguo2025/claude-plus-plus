#[cfg(target_os = "windows")]
mod imp {
    use crate::claude_desktop;
    use serde::Serialize;
    use serde_json::{Map, Value};
    use sha2::{Digest, Sha256};
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
    const CONVERSATION_TITLE_I18N_MARKER: &str = "__claudePlusEnhanceConversationTitleI18nV1";
    const SKILLS_BRIDGE_MARKER: &str = "__claudePlusSkillsBridgeV1";
    const SKILLS_MAIN_BRIDGE_MARKER: &str = "__claudePlusSkillsMainBridgeV1";
    const SKILLS_MAIN_BRIDGE_TARGET: &str = ".vite/build/index.js";
    const SKILLS_PRELOAD_BRIDGE_TARGET: &str = ".vite/build/mainView.js";
    const SKILLS_LIST_CHANNEL: &str = "claude-plus:skills:list";
    const SKILLS_TRASH_CHANNEL: &str = "claude-plus:skills:trash";
    const ASAR_INTEGRITY_BLOCK_SIZE: usize = 4 * 1024 * 1024;
    fn skills_bridge_script() -> String {
        r##";(()=>{const MARK="__claudePlusSkillsBridgeV1";
if(globalThis[MARK])return;
Object.defineProperty(globalThis,MARK,{value:!0});
try{
const{contextBridge,ipcRenderer}=require("electron");
contextBridge.exposeInMainWorld("claudePlusSkills",{list:()=>ipcRenderer.invoke("__CPP_SKILLS_LIST__"),trash:e=>ipcRenderer.invoke("__CPP_SKILLS_TRASH__",String(e||""))});
}catch(e){console.error("[Claude++] skills bridge failed",e)}
})();"##
            .replace("__CPP_SKILLS_LIST__", SKILLS_LIST_CHANNEL)
            .replace("__CPP_SKILLS_TRASH__", SKILLS_TRASH_CHANNEL)
    }

    fn skills_main_bridge_script() -> String {
        r##";(()=>{const MARK="__claudePlusSkillsMainBridgeV1";
if(globalThis[MARK])return;
Object.defineProperty(globalThis,MARK,{value:!0});
try{
const{ipcMain,shell}=require("electron"),fs=require("fs"),path=require("path"),crypto=require("crypto");
const home=process.env.USERPROFILE||process.env.HOME||"",claudeHome=path.join(home,".claude");
function norm(e){try{return path.resolve(e)}catch{return String(e||"")}}
function id(e){return crypto.createHash("sha256").update(norm(e).toLowerCase()).digest("hex").slice(0,32)}
function readText(e){try{return fs.readFileSync(e,"utf8")}catch{return""}}
function parseFrontmatter(e){const t={};let r=e.split(/\r?\n/);if((r.shift()||"").trim()!=="---")return t;let n=null;for(const e of r){const r=e.trimEnd();if(r.trim()==="---")break;if(/^\s/.test(e)&&n){t[n]=(t[n]?t[n]+" ":"")+r.trim();continue}const s=r.indexOf(":");if(s>0){n=r.slice(0,s).trim();t[n]=r.slice(s+1).trim().replace(/^['"]|['"]$/g,"")}}return t}
function firstSentence(e){for(const t of e.split(/\r?\n/).map(e=>e.trim()))if(t&&t!=="---"&&!t.startsWith("#")&&!t.includes(":")&&!t.startsWith("```"))return Array.from(t).slice(0,120).join("");return"未提供描述"}
function summary(e,t){const r=(t||"").replace(/\s+/g," ").trim();if(!r||r==="未提供描述")return"未提供描述。";return Array.from(r).slice(0,140).join("")}
function readSkill(e,t,r,n){const s=path.join(n,"SKILL.md");if(!fs.existsSync(s))return null;const a=readText(s),o=parseFrontmatter(a),l=o.name&&o.name.trim()?o.name.trim():path.basename(n),i=o.description&&o.description.trim()?o.description.trim():firstSentence(a),c=norm(n),d=norm(s);return{id:id(c),name:l,scope:e,source_label:t,project_path:r,path:c,skill_file:d,description:i,summary_zh:summary(l,i)}}
function collectRoot(e,t,r,n,s){try{if(!fs.existsSync(e))return;for(const a of fs.readdirSync(e,{withFileTypes:!0}))if(a.isDirectory()){const o=readSkill(t,r,n,path.join(e,a.name));o&&s.push(o)}}catch{}}
function fromClaudeJson(){try{const e=JSON.parse(readText(path.join(home,".claude.json"))),t=e&&e.projects&&typeof e.projects==="object"?Object.keys(e.projects):[];return t.map(norm)}catch{return[]}}
function walk(e,t){try{for(const r of fs.readdirSync(e,{withFileTypes:!0})){const n=path.join(e,r.name);if(r.isDirectory())walk(n,t);else if(r.isFile()&&n.endsWith(".jsonl"))for(const e of readText(n).split(/\r?\n/))if(e.includes('"cwd"'))try{const r=JSON.parse(e);typeof r.cwd==="string"&&t.add(norm(r.cwd))}catch{}}}catch{}}
function decodeProjectName(e){const t=e.split("--").filter(Boolean),r=(t[0]||"").replace(/:$/,"");if(r.length!==1)return null;const n=[r+":\\"];for(const e of t.slice(1))e.startsWith("claude-worktrees-")?(n.push(".claude"),n.push("worktrees"),n.push(e.slice("claude-worktrees-".length))):n.push(e==="claude"?".claude":e);return norm(path.join(...n))}
function projectPaths(){const e=new Set;for(const t of fromClaudeJson())try{fs.existsSync(t)&&fs.statSync(t).isDirectory()&&e.add(norm(t))}catch{};const t=path.join(claudeHome,"projects");walk(t,e);try{for(const r of fs.readdirSync(t,{withFileTypes:!0}))if(r.isDirectory()){const t=decodeProjectName(r.name);t&&fs.existsSync(t)&&e.add(norm(t))}}catch{}return Array.from(e).sort()}
function listSkills(){const e=[],t=[];const r=path.join(claudeHome,"skills");fs.existsSync(r)&&(t.push(norm(r)),collectRoot(r,"global","全局",null,e));const n=projectPaths();for(const r of n){const n=path.join(r,".claude","skills");fs.existsSync(n)&&(t.push(norm(n)),collectRoot(n,"project","项目",r,e))}e.sort((e,t)=>e.scope.localeCompare(t.scope)||String(e.project_path||"").localeCompare(String(t.project_path||""))||e.name.localeCompare(t.name));return{skills:e,roots:t,project_count:n.length}}
async function trashSkill(e){const t=listSkills().skills.find(t=>t.id===e);if(!t)throw new Error("未找到该 skill，可能已经被删除或路径已变化");const r=norm(t.path);if(!fs.existsSync(r)||!fs.statSync(r).isDirectory()||!fs.existsSync(path.join(r,"SKILL.md")))throw new Error("目标不是有效 skill 目录");await shell.trashItem(r);return{ok:!0}}
ipcMain.removeHandler("__CPP_SKILLS_LIST__");ipcMain.removeHandler("__CPP_SKILLS_TRASH__");
ipcMain.handle("__CPP_SKILLS_LIST__",()=>listSkills());
ipcMain.handle("__CPP_SKILLS_TRASH__",(e,t)=>trashSkill(String(t||"")));
}catch(e){console.error("[Claude++] skills main bridge failed",e)}
})();"##
            .replace("__CPP_SKILLS_LIST__", SKILLS_LIST_CHANNEL)
            .replace("__CPP_SKILLS_TRASH__", SKILLS_TRASH_CHANNEL)
    }
    const INJECT_SCRIPT: &str = r##";(()=>{const m="__claudePlusEnhanceNavV2";
if(window[m])return;
Object.defineProperty(window,m,{value:!0});
const v="3.8",n=[
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
function I(e){return/[A-Za-z]/.test(e)&&!/[\u4e00-\u9fff]/.test(e)&&e.length>=4&&e.length<=90&&!/^(Claude|Claude\\+\\+|New chat|Recents|Scheduled tasks|Projects|Chats|Search chats|Search projects)$/i.test(e)}
function J(e){if(!e||e.closest("svg,[aria-hidden='true'],button[aria-label*='more' i],button[aria-label*='更多']"))return null;const n=[];let t;const r=document.createTreeWalker(e,NodeFilter.SHOW_TEXT,{acceptNode:e=>{const n=e.parentElement;if(!n||n.closest("svg,[aria-hidden='true']"))return NodeFilter.FILTER_REJECT;const t=E(e.nodeValue);return I(t)?NodeFilter.FILTER_ACCEPT:NodeFilter.FILTER_REJECT}});for(;t=r.nextNode();)n.push(t);return n.sort((e,n)=>E(n.nodeValue).length-E(e.nodeValue).length)[0]||null}
function K(e){const n=e.getAttribute("href")||e.getAttribute("data-href")||e.getAttribute("data-to")||"",t=e.getAttribute("aria-label")||"",r=E(e.textContent),a=new RegExp("(^|/)chat(s)?(/|\\\\?|#|$)|conversation","i");return a.test(n)||/open .*chat|open .*conversation|select .*chat|rename chat|打开会话|选择.*会话/i.test(t)||(/^[A-Za-z0-9][\\s\\S]{3,90}$/.test(r)&&e.closest("aside,nav,[role=navigation]")&&t.includes(r))}
async function L(e,n){const t=E(n.nodeValue);if(!I(t)||e.getAttribute("data-claude-plus-original-title")===t)return;if(H.has(t)){const r=H.get(t);r&&(n.nodeValue=r,e.setAttribute("data-claude-plus-original-title",t),e.setAttribute("data-claude-plus-title-i18n",r));return}e.setAttribute("data-claude-plus-original-title",t);try{const r=await fetch("http://127.0.0.1:15722/claude-plus/conversation-title-i18n",{method:"POST",headers:{"Content-Type":"text/plain"},body:JSON.stringify({title:t})}),a=await r.json();const s=E(a&&a.title);if(r.ok&&s&&s!==t&&/[\u4e00-\u9fff]/.test(s)){H.set(t,s);n.nodeValue=s;e.setAttribute("data-claude-plus-title-i18n",s)}else H.set(t,"")}catch(r){H.set(t,"")}}
function M(){if(!window.__claudePlusEnhanceConversationTitleI18nV1)return;document.querySelectorAll("aside a,nav a,aside button,nav button,aside div,nav div,aside li,nav li,aside [role=link],nav [role=link],aside [role=button],nav [role=button],aside [role=listitem],nav [role=listitem]").forEach(e=>{if(!K(e))return;const n=J(e);n&&L(e,n)})}
function y(){b||q||(q=setTimeout(()=>{q=0,x();M()},250))}
function z(e){return String(e==null?"":e).replace(/[&<>"']/g,e=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[e]))}
function D(){try{if(localStorage.getItem("claudePlusCustom3pPane")!=="connectors")return}catch(e){return}for(let e=0;e<14;e++)setTimeout(()=>{const e=Array.from(document.querySelectorAll("button,a,[role=button],[role=tab],[role=menuitem],nav *,aside *")).find(e=>/连接器与扩展|Connectors|MCP servers/i.test(o(e)));if(e){e.click();try{localStorage.removeItem("claudePlusCustom3pPane")}catch(t){}}},220+e*250)}
function A(){let e=document.getElementById("claude-plus-skills-modal");if(e)return e.remove();e=document.createElement("div");e.id="claude-plus-skills-modal";e.innerHTML='<div class="cps-backdrop"></div><section class="cps-panel" role="dialog" aria-modal="true" aria-label="技能"><header><strong>技能</strong><button type="button" data-cps-close>关闭</button></header><main><p class="cps-loading">正在读取 skills...</p></main></section>';document.body.appendChild(e);const n=document.createElement("style");n.id="claude-plus-skills-style";n.textContent="#claude-plus-skills-modal{position:fixed;inset:0;z-index:2147483647;color:#f4f1ea;font:13px/1.45 system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}#claude-plus-skills-modal .cps-backdrop{position:absolute;inset:0;background:rgba(0,0,0,.52)}#claude-plus-skills-modal .cps-panel{position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:min(886px,calc(100vw - 48px));height:min(713px,calc(100vh - 48px));display:grid;grid-template-rows:auto 1fr;background:#171717;border:1px solid #3d3a35;border-radius:10px;box-shadow:0 22px 80px rgba(0,0,0,.48);overflow:hidden}#claude-plus-skills-modal header{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:18px 20px 12px;border-bottom:1px solid #2f2d2a;background:#1f1e1b}#claude-plus-skills-modal header strong{font-size:18px;font-weight:650}#claude-plus-skills-modal button{border:1px solid #5a544b;background:#2b2925;color:#f4f1ea;border-radius:7px;min-height:30px;padding:0 10px;cursor:pointer}#claude-plus-skills-modal button:hover{border-color:#d97745}#claude-plus-skills-modal button.cps-danger{border-color:#7f2d22;background:#4a1f1a;color:#ffd8cf}#claude-plus-skills-modal button:disabled{opacity:.55;cursor:default}#claude-plus-skills-modal main{overflow:auto;padding:18px 20px 20px;display:flex;flex-direction:column;gap:18px}#claude-plus-skills-modal .cps-section{display:flex;flex-direction:column;gap:10px}#claude-plus-skills-modal .cps-section-title{font-size:14px;font-weight:650;color:#f4f1ea}#claude-plus-skills-modal .cps-container{display:grid;gap:8px;border:1px solid #34312d;border-radius:8px;background:#1f1f1c;padding:10px}#claude-plus-skills-modal .cps-card{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:12px;padding:10px 12px;border:1px solid #34312d;border-radius:8px;background:#262521}#claude-plus-skills-modal .cps-main{display:flex;min-width:0;flex-direction:column;gap:4px}#claude-plus-skills-modal .cps-name{font-size:14px;font-weight:650;color:#f4f1ea}#claude-plus-skills-modal .cps-brief{color:#e7e0d4}#claude-plus-skills-modal .cps-file,#claude-plus-skills-modal .cps-empty,#claude-plus-skills-modal .cps-loading,#claude-plus-skills-modal .cps-error{color:#a9a39a}#claude-plus-skills-modal .cps-file{font-size:12px;word-break:break-all}#claude-plus-skills-modal .cps-actions{display:flex;align-items:flex-start;gap:8px}#claude-plus-skills-modal .cps-detail{grid-column:1/-1;border-top:1px solid #34312d;margin-top:4px;padding-top:10px;color:#d8d0c4;display:grid;gap:8px}#claude-plus-skills-modal .cps-detail[hidden]{display:none}#claude-plus-skills-modal .cps-detail strong{display:block;color:#f4f1ea;font-size:12px;margin-bottom:2px}.cps-toast{position:absolute;right:16px;bottom:14px;background:#2b2925;border:1px solid #5a544b;border-radius:8px;padding:8px 10px;color:#f4f1ea}";document.head.appendChild(n);function t(){e.remove();n.remove()}e.querySelector("[data-cps-close]").addEventListener("click",t);e.querySelector(".cps-backdrop").addEventListener("click",t);return e}
function C(e,n){const t=e.filter(e=>e.scope===n),r=n==="global"?"全局 skills":"项目 skills";return'<section class="cps-section"><div class="cps-section-title">'+r+'</div><div class="cps-container">'+(t.length?t.map(e=>'<article class="cps-card" data-id="'+z(e.id)+'"><div class="cps-main"><div class="cps-name">'+z(e.name)+'</div><div class="cps-brief">'+z(e.summary_zh||e.description||"未提供描述。")+'</div><div class="cps-file">'+z(e.skill_file||e.path)+'</div></div><div class="cps-actions"><button type="button" data-cps-detail>详情</button><button type="button" class="cps-danger" data-cps-trash>删除</button></div><div class="cps-detail" hidden><div><strong>原始描述</strong><div>'+z(e.description||"未提供描述。")+'</div></div><div><strong>文件地址</strong><div class="cps-file">'+z(e.skill_file||e.path)+'</div></div><div><strong>目录地址</strong><div class="cps-file">'+z(e.path)+'</div></div></div></article>').join(""):'<p class="cps-empty">暂无'+r+'。</p>')+"</div></section>"}
async function B(){const e=A(),n=e.querySelector("main"),t=window.claudePlusSkills;if(!t||typeof t.list!=="function"||typeof t.trash!=="function"){n.innerHTML='<p class="cps-error">本地 skills 桥未安装或尚未生效。</p><p class="cps-path">请在 Claude++ 中重新安装“技能”页面增强，并重启 Claude Desktop。</p>';return}try{const r=await t.list(),a=r.skills||[];n.innerHTML=C(a,"global")+C(a,"project");n.querySelectorAll("[data-cps-detail]").forEach(e=>e.addEventListener("click",()=>{const n=e.closest(".cps-card")?.querySelector(".cps-detail");if(!n)return;const t=n.hasAttribute("hidden");t?n.removeAttribute("hidden"):n.setAttribute("hidden","");e.textContent=t?"收起":"详情"}));n.querySelectorAll("[data-cps-trash]").forEach(r=>r.addEventListener("click",async()=>{const a=r.closest(".cps-card"),s=s=>{let n=e.querySelector(".cps-toast");n||(n=document.createElement("div"),n.className="cps-toast",e.appendChild(n));n.textContent=s;setTimeout(()=>n&&n.remove(),2600)},l=a?.dataset.id,o=a?.querySelector(".cps-name")?.textContent||"该 skill";if(!l)return;if(!confirm("确认删除 skill “"+o+"”？\n\n该操作会把对应 skill 目录移动到回收站。"))return;r.disabled=!0;try{await t.trash(l);a.remove();s("已移动到回收站")}catch(e){r.disabled=!1;s(e.message||String(e))}}))}catch(r){n.innerHTML='<p class="cps-error">读取本地 skills 失败。</p><p class="cps-path">'+z(r.message||String(r))+"</p>"}}
async function s(e){if(e.open==="custom3p"||e.open==="custom3p_connectors"){const n=window["claude.settings"]?.Custom3pSetup?.openSetupWindow||window.claude?.settings?.Custom3pSetup?.openSetupWindow;if(typeof n==="function"){try{e.open==="custom3p_connectors"&&localStorage.setItem("claudePlusCustom3pPane","connectors");await n();return}catch(t){}}return}if(e.open==="skills"){B();return}const n=new URL(e.path,location.origin),t=n.pathname+n.search+n.hash;try{history.pushState(null,"",t);window.dispatchEvent(new PopStateEvent("popstate",{state:history.state}));window.dispatchEvent(new Event("pushstate"));window.dispatchEvent(new Event("locationchange"))}catch(r){location.assign(n.toString())}}
new MutationObserver(y).observe(document.documentElement,{childList:!0,subtree:!0});
document.readyState==="loading"?document.addEventListener("DOMContentLoaded",()=>{x();M()},{once:!0}):(x(),M());
D();
})();"##;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum EnhanceFeatureId {
        ThirdPartyApi,
        Plugins,
        Mcp,
        ConversationTitleI18n,
        Markdown,
        Timeline,
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
                _ => None,
            }
        }

        fn marker(self) -> &'static str {
            match self {
                Self::ThirdPartyApi => NAV_API_MARKER,
                Self::Plugins => NAV_PLUGINS_MARKER,
                Self::Mcp => NAV_MCP_MARKER,
                Self::ConversationTitleI18n => CONVERSATION_TITLE_I18N_MARKER,
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

        fn backup_app_file(&mut self, path: &Path) -> Result<(), String> {
            if !path.exists() || self.backed_up.contains(path) {
                return Ok(());
            }

            let app_path = app_path_from_resources(&self.resources_path);
            let relative = relative_to(path, &app_path)?;
            let target = self.ensure_set()?.join("_app").join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("创建增强备份父目录失败: {e}"))?;
            }
            fs::copy(path, &target).map_err(|e| format!("备份 Claude 程序文件失败: {e}"))?;
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
        if matches!(feature, EnhanceFeatureId::Plugins) {
            update_skills_bridge(&paths.resources, &mut backup, true)?;
        }
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
        if matches!(feature, EnhanceFeatureId::Plugins) {
            update_skills_bridge(&paths.resources, &mut backup, false)?;
        }
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
                description: "在 Claude Desktop 左侧菜单中打开本地 skills 弹窗。",
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
                id: "conversation_title_i18n",
                category: "对话增强",
                label: "对话列表中文化",
                description: "把 Claude Desktop 对话列表里的英文标题自动翻译为中文显示。",
                enabled: is_enabled(enabled, EnhanceFeatureId::ConversationTitleI18n),
                available: true,
                note: "本地代理翻译",
            },
            EnhanceFeature {
                id: "markdown",
                category: "对话增强",
                label: "对话导出Markdown",
                description: "在对话页面增加 Markdown 导出入口，把当前对话保存为 Markdown 文件。",
                enabled: is_enabled(enabled, EnhanceFeatureId::Markdown),
                available: true,
                note: "待接入导出逻辑",
            },
            EnhanceFeature {
                id: "timeline",
                category: "对话增强",
                label: "对话时间线",
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
        let mut states = feature_states_from_text(&text);
        if let Some((_, enabled)) = states
            .iter_mut()
            .find(|(feature, _)| *feature == EnhanceFeatureId::Plugins)
        {
            *enabled = *enabled && skills_bridge_installed(resources_path);
        }
        states
    }

    fn feature_states_from_text(text: &str) -> Vec<(EnhanceFeatureId, bool)> {
        [
            EnhanceFeatureId::ThirdPartyApi,
            EnhanceFeatureId::Plugins,
            EnhanceFeatureId::Mcp,
            EnhanceFeatureId::ConversationTitleI18n,
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

    fn update_skills_bridge(
        resources_path: &Path,
        backup: &mut BackupContext,
        enabled: bool,
    ) -> Result<(), String> {
        let main_script = skills_main_bridge_script();
        let preload_script = skills_bridge_script();
        patch_skills_bridge_file(
            resources_path,
            SKILLS_MAIN_BRIDGE_TARGET,
            &main_script,
            backup,
            enabled,
        )?;
        patch_skills_bridge_file(
            resources_path,
            SKILLS_PRELOAD_BRIDGE_TARGET,
            &preload_script,
            backup,
            enabled,
        )
    }

    fn patch_skills_bridge_file(
        resources_path: &Path,
        file_path: &str,
        script: &str,
        backup: &mut BackupContext,
        enabled: bool,
    ) -> Result<(), String> {
        patch_asar_file(resources_path, file_path, backup, |content| {
            let text =
                std::str::from_utf8(content).map_err(|e| format!("preload 入口不是 UTF-8: {e}"))?;
            let mut next = remove_skills_bridge(text);
            if enabled {
                next.insert_str(0, script);
            }
            if next == text {
                Ok(None)
            } else {
                Ok(Some(next.into_bytes()))
            }
        })
    }

    fn remove_skills_bridge(text: &str) -> String {
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

    fn skills_bridge_installed(resources_path: &Path) -> bool {
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

    #[cfg(test)]
    mod tests {
        use super::{
            feature_payload, feature_states_from_text, remove_skills_bridge, EnhanceFeatureId,
            CONVERSATION_TITLE_I18N_MARKER, INJECT_SCRIPT, NAV_API_MARKER, NAV_MCP_MARKER,
            NAV_PLUGINS_MARKER, SKILLS_LIST_CHANNEL, SKILLS_MAIN_BRIDGE_TARGET,
            SKILLS_PRELOAD_BRIDGE_TARGET, SKILLS_TRASH_CHANNEL,
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
            assert!(!state(&states, EnhanceFeatureId::ConversationTitleI18n));
            assert!(!text.contains(&feature_payload(NAV_PLUGINS_MARKER)));
            assert!(!text.contains(&feature_payload(CONVERSATION_TITLE_I18N_MARKER)));
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
            assert!(INJECT_SCRIPT.contains("/claude-plus/conversation-title-i18n"));
            assert!(INJECT_SCRIPT.contains("data-claude-plus-original-title"));
            assert!(INJECT_SCRIPT.contains("data-claude-plus-title-i18n"));
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
        }

        #[test]
        fn skills_popup_uses_preload_bridge_not_local_service() {
            assert!(INJECT_SCRIPT.contains("window.claudePlusSkills"));
            assert!(INJECT_SCRIPT.contains("width:min(886px,calc(100vw - 48px))"));
            assert!(INJECT_SCRIPT.contains("height:min(713px,calc(100vh - 48px))"));
            assert!(!INJECT_SCRIPT.contains("127.0.0.1:15722/claude-plus/skills"));
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
            assert!(!INJECT_SCRIPT.contains(r#"id:"mcp",marker:"__claudePlusEnhanceMcpV1",label:"MCP",path:"/customize/connectors""#));
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
            let script = super::skills_main_bridge_script();
            assert!(script.contains("ipcMain.handle"));
            assert!(script.contains("require(\"fs\")"));
            assert!(script.contains("shell.trashItem"));
            assert!(script.contains("listSkills"));
            assert!(script.contains(SKILLS_LIST_CHANNEL));
            assert!(script.contains(SKILLS_TRASH_CHANNEL));
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
                super::skills_main_bridge_script()
            );
            let cleaned = remove_skills_bridge(&text);

            assert_eq!(
                cleaned,
                "const ready=true;//# sourceMappingURL=mainView.js.map"
            );
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

    fn read_asar_file(resources_path: &Path, file_path: &str) -> Result<Vec<u8>, String> {
        let asar_path = resources_path.join("app.asar");
        let data = fs::read(&asar_path).map_err(|e| format!("读取 app.asar 失败: {e}"))?;
        let parsed = read_asar_header(&data, &asar_path)?;
        let header: Value = serde_json::from_str(&parsed.header_string)
            .map_err(|e| format!("解析 app.asar 头失败: {e}"))?;
        let entry = get_asar_entry(&header, file_path)?;
        let offset = entry_value_to_usize(entry.get("offset"), "offset")?;
        let size = entry_value_to_usize(entry.get("size"), "size")?;
        let content_offset = 8 + parsed.header_size + offset;
        let content_end = content_offset + size;
        if content_end > data.len() {
            return Err("app.asar 目标文件边界无效".to_string());
        }
        Ok(data[content_offset..content_end].to_vec())
    }

    fn patch_asar_file<F>(
        resources_path: &Path,
        file_path: &str,
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
        let entry = get_asar_entry_mut(&mut header, file_path)?;
        let offset = entry_value_to_usize(entry.get("offset"), "offset")?;
        let old_size = entry_value_to_usize(entry.get("size"), "size")?;
        let content_offset = 8 + parsed.header_size + offset;
        let content_end = content_offset + old_size;
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

        backup.backup_resource(&asar_path)?;
        data.splice(content_offset..content_end, patched_content.iter().copied());
        entry["size"] = Value::Number((patched_content.len() as u64).into());
        entry["integrity"] = asar_file_integrity(&patched_content);
        shift_asar_offsets_after(
            &mut header,
            offset,
            patched_content.len() as i64 - old_size as i64,
        )?;
        let header_string =
            serde_json::to_string(&header).map_err(|e| format!("生成 app.asar 头失败: {e}"))?;
        let encoded_header = encode_asar_header(&header_string);
        let content_start = 8 + parsed.header_size;
        let mut next_data = Vec::with_capacity(encoded_header.len() + data.len() - content_start);
        next_data.extend_from_slice(&encoded_header);
        next_data.extend_from_slice(&data[content_start..]);
        data = next_data;
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

    fn encode_asar_header(header_string: &str) -> Vec<u8> {
        let header_bytes = header_string.as_bytes();
        let header_payload_size = align4(4 + header_bytes.len());
        let header_size = 4 + header_payload_size;
        let mut header_pickle = vec![0u8; header_size];
        header_pickle[0..4].copy_from_slice(&(header_payload_size as u32).to_le_bytes());
        header_pickle[4..8].copy_from_slice(&(header_bytes.len() as i32).to_le_bytes());
        header_pickle[8..8 + header_bytes.len()].copy_from_slice(header_bytes);

        let mut encoded = vec![0u8; 8 + header_size];
        encoded[0..4].copy_from_slice(&4u32.to_le_bytes());
        encoded[4..8].copy_from_slice(&(header_size as u32).to_le_bytes());
        encoded[8..].copy_from_slice(&header_pickle);
        encoded
    }

    fn get_asar_entry<'a>(header: &'a Value, file_path: &str) -> Result<&'a Value, String> {
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
        for key in ["size", "offset", "integrity"] {
            if node.get(key).is_none() {
                return Err(format!("app.asar 目标缺少字段: {key}"));
            }
        }
        Ok(node)
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

    fn shift_asar_offsets_after(
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
