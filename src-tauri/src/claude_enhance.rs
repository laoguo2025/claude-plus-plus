#[cfg(target_os = "windows")]
mod imp {
    use crate::claude_desktop;
    use crate::claude_patch_common as patch;
    use crate::constants::DEFAULT_PROXY_PORT;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::{fs, path::Path};

    const SCRIPT_MARKER: &str = "__claudePlusEnhanceNavV2";
    const NAV_API_MARKER: &str = "__claudePlusEnhanceThirdPartyApiV1";
    const NAV_PLUGINS_MARKER: &str = "__claudePlusEnhancePluginsV1";
    const NAV_MCP_MARKER: &str = "__claudePlusEnhanceMcpV1";
    const CONVERSATION_TITLE_I18N_MARKER: &str = "__claudePlusEnhanceConversationTitleI18nV1";
    const MARKDOWN_EXPORT_MARKER: &str = "__claudePlusEnhanceMarkdownExportV1";
    const TIMELINE_MARKER: &str = "__claudePlusEnhanceTimelineV1";
    const TOKEN_USAGE_MARKER: &str = "__claudePlusEnhanceTokenUsageV1";
    const SKILLS_BRIDGE_MARKER: &str = "__claudePlusSkillsBridgeV1";
    const SKILLS_MAIN_BRIDGE_MARKER: &str = "__claudePlusSkillsMainBridgeV1";
    const SKILLS_MAIN_BRIDGE_TARGET: &str = ".vite/build/index.js";
    const SKILLS_PRELOAD_BRIDGE_TARGET: &str = ".vite/build/mainView.js";
    const SKILLS_LIST_CHANNEL: &str = "claude-plus:skills:list";
    const SKILLS_TRASH_CHANNEL: &str = "claude-plus:skills:trash";
    const TITLE_I18N_BRIDGE_MARKER: &str = "__claudePlusTitleI18nBridgeV1";
    const TITLE_I18N_MAIN_BRIDGE_MARKER: &str = "__claudePlusTitleI18nMainBridgeV1";
    const TITLE_I18N_CHANNEL: &str = "claude-plus:title-i18n";
    const TOKEN_USAGE_BRIDGE_MARKER: &str = "__claudePlusTokenUsageBridgeV1";
    const TOKEN_USAGE_MAIN_BRIDGE_MARKER: &str = "__claudePlusTokenUsageMainBridgeV1";
    const TOKEN_USAGE_CHANNEL: &str = "claude-plus:token-usage";
    const BACKUP_DIR_NAME: &str = ".claude-plus-enhance-backups";
    const ENHANCE_FEATURES_JSON: &str = include_str!("../../src/shared/enhance-features.json");

    fn local_gateway_runtime_js() -> String {
        format!(
            r#"function cppPort(){{try{{const e=process.env.CLAUDE_PLUS_PROXY_PORT;if(e&&/^\d+$/.test(String(e).trim())){{const t=Number(String(e).trim());if(t>0&&t<65536)return t}}}}catch{{}}try{{const e=JSON.parse(fs.readFileSync(path.join(process.env.USERPROFILE||process.env.HOME||"",".claude-plus-plus","settings.json"),"utf8")),t=e.proxyPort??e.proxy_port;if(t!=null&&/^\d+$/.test(String(t).trim())){{const n=Number(String(t).trim());if(n>0&&n<65536)return n}}}}catch{{}}return {DEFAULT_PROXY_PORT}}}function cppUrl(e){{return "http://127.0.0.1:"+cppPort()+e}}"#
        )
    }

    fn local_gateway_token_js() -> &'static str {
        r#"function cppToken(){try{return fs.readFileSync(path.join(process.env.USERPROFILE||process.env.HOME||"",".claude-plus-plus","local-gateway-token"),"utf8").trim()}catch{return""}}"#
    }

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

    fn skills_main_bridge_script(locale: EnhanceScriptLocale) -> String {
        let script = r##";(()=>{const MARK="__claudePlusSkillsMainBridgeV1";
if(globalThis[MARK])return;
Object.defineProperty(globalThis,MARK,{value:!0});
try{
const{ipcMain,shell}=require("electron"),fs=require("fs"),path=require("path"),crypto=require("crypto");
__CPP_GATEWAY_RUNTIME__
__CPP_TOKEN_READER__
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
async function gatewayList(){const e=await fetch(cppUrl("/claude-plus/skills"),{cache:"no-store",headers:{"x-claude-plus-gateway-token":cppToken()}});if(!e.ok)throw new Error("Claude++ skills gateway failed: "+e.status);return await e.json()}
async function gatewayTrash(e){const t=await fetch(cppUrl("/claude-plus/skills/"+encodeURIComponent(e)+"/trash"),{method:"POST",headers:{"x-claude-plus-gateway-token":cppToken()}});if(!t.ok)throw new Error("Claude++ skills gateway failed: "+t.status);return await t.json().catch(()=>({ok:true}))}
async function listSkillsFast(){try{return await gatewayList()}catch(e){return listSkills()}}
async function trashSkillFast(e){try{return await gatewayTrash(e)}catch(t){return trashSkill(e)}}
ipcMain.removeHandler("__CPP_SKILLS_LIST__");ipcMain.removeHandler("__CPP_SKILLS_TRASH__");
ipcMain.handle("__CPP_SKILLS_LIST__",()=>listSkillsFast());
ipcMain.handle("__CPP_SKILLS_TRASH__",(e,t)=>trashSkillFast(String(t||"")));
}catch(e){console.error("[Claude++] skills main bridge failed",e)}
})();"##
            .replace("__CPP_GATEWAY_RUNTIME__", &local_gateway_runtime_js())
            .replace("__CPP_TOKEN_READER__", local_gateway_token_js())
            .replace("__CPP_SKILLS_LIST__", SKILLS_LIST_CHANNEL)
            .replace("__CPP_SKILLS_TRASH__", SKILLS_TRASH_CHANNEL);
        match locale {
            EnhanceScriptLocale::ZhCn => script,
            EnhanceScriptLocale::EnUs => english_skills_main_bridge_script(script),
        }
    }

    fn english_skills_main_bridge_script(mut script: String) -> String {
        for (source, target) in [
            (
                r#"return"未提供描述""#,
                r#"return"No description provided""#,
            ),
            (
                r#"if(!r||r==="未提供描述")return"未提供描述。""#,
                r#"if(!r||r==="No description provided")return"No description provided.""#,
            ),
            (
                r#"collectRoot(r,"global","全局",null,e)"#,
                r#"collectRoot(r,"global","Global",null,e)"#,
            ),
            (
                r#"collectRoot(n,"project","项目",r,e)"#,
                r#"collectRoot(n,"project","Project",r,e)"#,
            ),
            (
                r#"throw new Error("未找到该 skill，可能已经被删除或路径已变化")"#,
                r#"throw new Error("Skill not found. It may have been deleted or moved.")"#,
            ),
            (
                r#"throw new Error("目标不是有效 skill 目录")"#,
                r#"throw new Error("Target is not a valid skill directory")"#,
            ),
        ] {
            script = script.replace(source, target);
        }
        script
    }

    fn title_i18n_preload_bridge_script() -> String {
        r##";(()=>{const MARK="__CPP_TITLE_I18N_MARK__";
if(globalThis[MARK])return;
Object.defineProperty(globalThis,MARK,{value:!0});
try{
const{contextBridge,ipcRenderer}=require("electron");
contextBridge.exposeInMainWorld("claudePlusTitleI18n",{translate:e=>ipcRenderer.invoke("__CPP_TITLE_I18N__",String(e||""))});
}catch(e){console.error("[Claude++] title i18n bridge failed",e)}
})();"##
            .replace("__CPP_TITLE_I18N_MARK__", TITLE_I18N_BRIDGE_MARKER)
            .replace("__CPP_TITLE_I18N__", TITLE_I18N_CHANNEL)
    }

    fn title_i18n_main_bridge_script() -> String {
        r##";(()=>{const MARK="__CPP_TITLE_I18N_MAIN_MARK__";
if(globalThis[MARK])return;
Object.defineProperty(globalThis,MARK,{value:!0});
try{
const{ipcMain}=require("electron"),fs=require("fs"),path=require("path");
__CPP_GATEWAY_RUNTIME__
__CPP_TOKEN_READER__
async function translate(e){const t=String(e||"").replace(/\s+/g," ").trim();if(!t)return"";const r=await fetch(cppUrl("/claude-plus/conversation-title-i18n"),{method:"POST",headers:{"Content-Type":"application/json","x-claude-plus-gateway-token":cppToken()},body:JSON.stringify({title:t})});const n=await r.json().catch(() => ({}));return r.ok&&n&&typeof n.title==="string"?n.title:""}
ipcMain.removeHandler("__CPP_TITLE_I18N__");
ipcMain.handle("__CPP_TITLE_I18N__",(e,t)=>translate(t));
}catch(e){console.error("[Claude++] title i18n main bridge failed",e)}
})();"##
            .replace("__CPP_TITLE_I18N_MAIN_MARK__", TITLE_I18N_MAIN_BRIDGE_MARKER)
            .replace("__CPP_TITLE_I18N__", TITLE_I18N_CHANNEL)
            .replace("__CPP_GATEWAY_RUNTIME__", &local_gateway_runtime_js())
            .replace("__CPP_TOKEN_READER__", local_gateway_token_js())
    }

    fn token_usage_preload_bridge_script() -> String {
        r##";(()=>{const MARK="__CPP_TOKEN_USAGE_MARK__";
if(globalThis[MARK])return;
Object.defineProperty(globalThis,MARK,{value:!0});
try{
const{contextBridge,ipcRenderer}=require("electron");
contextBridge.exposeInMainWorld("claudePlusTokenUsage",{get:e=>ipcRenderer.invoke("__CPP_TOKEN_USAGE__",e||{})});
}catch(e){console.error("[Claude++] token usage bridge failed",e)}
})();"##
            .replace("__CPP_TOKEN_USAGE_MARK__", TOKEN_USAGE_BRIDGE_MARKER)
            .replace("__CPP_TOKEN_USAGE__", TOKEN_USAGE_CHANNEL)
    }

    fn token_usage_main_bridge_script() -> String {
        r##";(()=>{const MARK="__CPP_TOKEN_USAGE_MAIN_MARK__";
if(globalThis[MARK])return;
Object.defineProperty(globalThis,MARK,{value:!0});
try{
const{ipcMain}=require("electron"),fs=require("fs"),path=require("path");
__CPP_GATEWAY_RUNTIME__
__CPP_TOKEN_READER__
async function getUsage(e){const t=cppUrl("/claude-plus/token-usage"),n=e&&e.sinceMs?(t+"?sinceMs="+encodeURIComponent(e.sinceMs)):t;const r=await fetch(n,{cache:"no-store",headers:{"x-claude-plus-gateway-token":cppToken()}});return await r.json().catch(()=>({ok:false}))}
ipcMain.removeHandler("__CPP_TOKEN_USAGE__");
ipcMain.handle("__CPP_TOKEN_USAGE__",(e,n)=>getUsage(n));
}catch(e){console.error("[Claude++] token usage main bridge failed",e)}
})();"##
            .replace("__CPP_TOKEN_USAGE_MAIN_MARK__", TOKEN_USAGE_MAIN_BRIDGE_MARKER)
            .replace("__CPP_TOKEN_USAGE__", TOKEN_USAGE_CHANNEL)
            .replace("__CPP_GATEWAY_RUNTIME__", &local_gateway_runtime_js())
            .replace("__CPP_TOKEN_READER__", local_gateway_token_js())
    }

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

    fn current_script_locale() -> EnhanceScriptLocale {
        EnhanceScriptLocale::from_claude_locale(crate::claude_zh::status().locale.as_deref())
    }

    fn inject_script_for_locale(locale: EnhanceScriptLocale) -> String {
        let script = INJECT_SCRIPT_TEMPLATE.to_string();
        match locale {
            EnhanceScriptLocale::ZhCn => script,
            EnhanceScriptLocale::EnUs => english_inject_script(script),
        }
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
            (r#"<button type="button" data-cps-detail>详情</button>"#, r#"<button type="button" data-cps-detail>Details</button>"#),
            (r#"<button type="button" class="cps-danger" data-cps-trash>删除</button>"#, r#"<button type="button" class="cps-danger" data-cps-trash>Delete</button>"#),
            (r#"<strong>原始描述</strong>"#, r#"<strong>Original description</strong>"#),
            (r#"<strong>文件地址</strong>"#, r#"<strong>Skill file</strong>"#),
            (r#"<strong>目录地址</strong>"#, r#"<strong>Directory</strong>"#),
            (r#""未提供描述。""#, r#""No description provided.""#),
            (r#"'<p class="cps-empty">暂无'+r+'。</p>'"#, r#"'<p class="cps-empty">No '+r+'.</p>'"#),
            (
                r#"<p class="cps-error">本地 skills 桥未安装或尚未生效。</p><p class="cps-path">请在 Claude++ 中重新安装“技能”页面增强，并重启 Claude Desktop。</p>"#,
                r#"<p class="cps-error">The local skills bridge is not installed or not active yet.</p><p class="cps-path">Reinstall the Skills enhancement in Claude++ and restart Claude Desktop.</p>"#,
            ),
            (r#"e.textContent=t?"收起":"详情""#, r#"e.textContent=t?"Hide":"Details""#),
            (r#""该 skill""#, r#""this skill""#),
            (
                r#""确认删除 skill “"+o+"”？\n\n该操作会把对应 skill 目录移动到回收站。""#,
                r#""Delete skill \""+o+"\"?\n\nThis moves the skill directory to the Recycle Bin.""#,
            ),
            (r#"s("已移动到回收站")"#, r#"s("Moved to the Recycle Bin")"#),
            (
                r#"<p class="cps-error">读取本地 skills 失败。</p>"#,
                r#"<p class="cps-error">Failed to read local skills.</p>"#,
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
function z(e){return String(e==null?"":e).replace(/[&<>"']/g,e=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[e]))}
function D(){try{if(localStorage.getItem("claudePlusCustom3pPane")!=="connectors")return}catch(e){return}for(let e=0;e<14;e++)setTimeout(()=>{const e=Array.from(document.querySelectorAll("button,a,[role=button],[role=tab],[role=menuitem],nav *,aside *")).find(e=>/连接器与扩展|Connectors|MCP servers/i.test(o(e)));if(e){e.click();try{localStorage.removeItem("claudePlusCustom3pPane")}catch(t){}}},220+e*250)}
function A(){let e=document.getElementById("claude-plus-skills-modal");if(e)return e.remove();e=document.createElement("div");e.id="claude-plus-skills-modal";e.innerHTML='<div class="cps-backdrop"></div><section class="cps-panel" role="dialog" aria-modal="true" aria-label="技能"><header><strong>技能</strong><button type="button" data-cps-close>关闭</button></header><main><p class="cps-loading">正在读取 skills...</p></main></section>';document.body.appendChild(e);const n=document.createElement("style");n.id="claude-plus-skills-style";n.textContent="#claude-plus-skills-modal{position:fixed;inset:0;z-index:2147483647;color:#f4f1ea;font:13px/1.45 system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}#claude-plus-skills-modal .cps-backdrop{position:absolute;inset:0;background:rgba(0,0,0,.52)}#claude-plus-skills-modal .cps-panel{position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:min(886px,calc(100vw - 48px));height:min(713px,calc(100vh - 48px));display:grid;grid-template-rows:auto 1fr;background:#171717;border:1px solid #3d3a35;border-radius:10px;box-shadow:0 22px 80px rgba(0,0,0,.48);overflow:hidden}#claude-plus-skills-modal header{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:18px 20px 12px;border-bottom:1px solid #2f2d2a;background:#1f1e1b}#claude-plus-skills-modal header strong{font-size:18px;font-weight:650}#claude-plus-skills-modal button{border:1px solid #5a544b;background:#2b2925;color:#f4f1ea;border-radius:7px;min-height:30px;padding:0 10px;cursor:pointer}#claude-plus-skills-modal button:hover{border-color:#d97745}#claude-plus-skills-modal button.cps-danger{border-color:#7f2d22;background:#4a1f1a;color:#ffd8cf}#claude-plus-skills-modal button:disabled{opacity:.55;cursor:default}#claude-plus-skills-modal main{overflow:auto;padding:18px 20px 20px;display:flex;flex-direction:column;gap:18px}#claude-plus-skills-modal .cps-section{display:flex;flex-direction:column;gap:10px}#claude-plus-skills-modal .cps-section-title{font-size:14px;font-weight:650;color:#f4f1ea}#claude-plus-skills-modal .cps-container{display:grid;gap:8px;border:1px solid #34312d;border-radius:8px;background:#1f1f1c;padding:10px}#claude-plus-skills-modal .cps-card{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:12px;padding:10px 12px;border:1px solid #34312d;border-radius:8px;background:#262521}#claude-plus-skills-modal .cps-main{display:flex;min-width:0;flex-direction:column;gap:4px}#claude-plus-skills-modal .cps-name{font-size:14px;font-weight:650;color:#f4f1ea}#claude-plus-skills-modal .cps-brief{color:#e7e0d4}#claude-plus-skills-modal .cps-file,#claude-plus-skills-modal .cps-empty,#claude-plus-skills-modal .cps-loading,#claude-plus-skills-modal .cps-error{color:#a9a39a}#claude-plus-skills-modal .cps-file{font-size:12px;word-break:break-all}#claude-plus-skills-modal .cps-actions{display:flex;align-items:flex-start;gap:8px}#claude-plus-skills-modal .cps-detail{grid-column:1/-1;border-top:1px solid #34312d;margin-top:4px;padding-top:10px;color:#d8d0c4;display:grid;gap:8px}#claude-plus-skills-modal .cps-detail[hidden]{display:none}#claude-plus-skills-modal .cps-detail strong{display:block;color:#f4f1ea;font-size:12px;margin-bottom:2px}.cps-toast{position:absolute;right:16px;bottom:14px;background:#2b2925;border:1px solid #5a544b;border-radius:8px;padding:8px 10px;color:#f4f1ea}";document.head.appendChild(n);function t(){e.remove();n.remove()}e.querySelector("[data-cps-close]").addEventListener("click",t);e.querySelector(".cps-backdrop").addEventListener("click",t);return e}
function C(e,n){const t=e.filter(e=>e.scope===n),r=n==="global"?"全局 skills":"项目 skills";return'<section class="cps-section"><div class="cps-section-title">'+r+'</div><div class="cps-container">'+(t.length?t.map(e=>'<article class="cps-card" data-id="'+z(e.id)+'"><div class="cps-main"><div class="cps-name">'+z(e.name)+'</div><div class="cps-brief">'+z(e.summary_zh||e.description||"未提供描述。")+'</div><div class="cps-file">'+z(e.skill_file||e.path)+'</div></div><div class="cps-actions"><button type="button" data-cps-detail>详情</button><button type="button" class="cps-danger" data-cps-trash>删除</button></div><div class="cps-detail" hidden><div><strong>原始描述</strong><div>'+z(e.description||"未提供描述。")+'</div></div><div><strong>文件地址</strong><div class="cps-file">'+z(e.skill_file||e.path)+'</div></div><div><strong>目录地址</strong><div class="cps-file">'+z(e.path)+'</div></div></div></article>').join(""):'<p class="cps-empty">暂无'+r+'。</p>')+"</div></section>"}
async function B(){const e=A(),n=e.querySelector("main"),t=window.claudePlusSkills;if(!t||typeof t.list!=="function"||typeof t.trash!=="function"){n.innerHTML='<p class="cps-error">本地 skills 桥未安装或尚未生效。</p><p class="cps-path">请在 Claude++ 中重新安装“技能”页面增强，并重启 Claude Desktop。</p>';return}try{const r=await t.list(),a=r.skills||[];n.innerHTML=C(a,"global")+C(a,"project");n.querySelectorAll("[data-cps-detail]").forEach(e=>e.addEventListener("click",()=>{const n=e.closest(".cps-card")?.querySelector(".cps-detail");if(!n)return;const t=n.hasAttribute("hidden");t?n.removeAttribute("hidden"):n.setAttribute("hidden","");e.textContent=t?"收起":"详情"}));n.querySelectorAll("[data-cps-trash]").forEach(r=>r.addEventListener("click",async()=>{const a=r.closest(".cps-card"),s=s=>{let n=e.querySelector(".cps-toast");n||(n=document.createElement("div"),n.className="cps-toast",e.appendChild(n));n.textContent=s;setTimeout(()=>n&&n.remove(),2600)},l=a?.dataset.id,o=a?.querySelector(".cps-name")?.textContent||"该 skill";if(!l)return;if(!confirm("确认删除 skill “"+o+"”？\n\n该操作会把对应 skill 目录移动到回收站。"))return;r.disabled=!0;try{await t.trash(l);a.remove();s("已移动到回收站")}catch(e){r.disabled=!1;s(e.message||String(e))}}))}catch(r){n.innerHTML='<p class="cps-error">读取本地 skills 失败。</p><p class="cps-path">'+z(r.message||String(r))+"</p>"}}
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
const CPU_RECENT_LIMIT=20;
const CPU_DEBUG_LIMIT=50;
const CPU_LEDGER_LIMIT=500;
const CPU_CONTEXT_POLL_INTERVAL_MS=1000;
const CPU_TURN_IDLE_TIMEOUT_MS=120000;
const CPU_CONTEXT_MERGE_WINDOW_MS=30000;
const CPU_CROSS_SOURCE_DEDUPE_WINDOW_MS=3000;
const CPU_FINAL_RENDER_DELAY_MS=900;
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
function cpuCollectUsages(e,t=0,n=[]){if(!e||t>8)return n;if(Array.isArray(e)){e.forEach(e=>cpuCollectUsages(e,t+1,n));return n}if(typeof e!=="object")return n;for(const r of["usage","token_usage","tokenUsage","last","lastUsage","last_token_usage","lastTokenUsage"]){const t=cpuNormalizeUsage(e[r]);t&&n.push(t)}const r=cpuNormalizeUsage(e);if(r){n.push(r);return n}for(const r of["response","data","body","message","result","event","params","context_usage","contextUsage","info","completion","delta"])cpuCollectUsages(e[r],t+1,n);return n}
function cpuExtractUsages(e){if(typeof e==="string"){const t=[];try{cpuCollectUsages(JSON.parse(e),0,t)}catch(n){}String(e||"").split(/\r?\n/).map(e=>e.trim()).filter(e=>e.startsWith("data:")).map(e=>e.slice(5).trim()).filter(e=>e&&e!=="[DONE]").forEach(e=>{try{cpuCollectUsages(JSON.parse(e),0,t)}catch(n){}});return t}return cpuCollectUsages(e)}
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
function cpuInstallWebSocketObserver(){if(typeof window.WebSocket!=="function"||window.__claudePlusTokenUsageWebSocketWrapped)return;const NativeWebSocket=window.__claudePlusTokenUsageNativeWebSocket||window.WebSocket;function T(...e){const t=new NativeWebSocket(...e);t.addEventListener?.("message",e=>{try{typeof e.data==="string"?cpuPayload(e.data,0,"websocket"):e.data instanceof Blob&&e.data.size<=512000&&e.data.text().then(e=>cpuPayload(e,0,"websocket")).catch(()=>{})}catch(t){}});return t}T.prototype=NativeWebSocket.prototype;window.WebSocket=T;window.__claudePlusTokenUsageNativeWebSocket=NativeWebSocket;window.__claudePlusTokenUsageWebSocketWrapped=!0}
function cpuInstallPostMessageObserver(){if(window.__claudePlusTokenUsageMessageObserver)return;window.addEventListener?.("message",e=>{try{cpuPayload(e.data,0,"post-message")}catch(t){}},!0);window.__claudePlusTokenUsageMessageObserver=!0}
function cpuCacheText(e,n){return e?cpuF(n):"未知"}
function cpuRateText(e,n,t,o){const r=t+n+(o||0);return e&&r?cpuPct(Math.min(n,r),r):"未知"}
function cpuContextText(e){const r=e.contextUsed||e.total||0,a=e.contextLimit||0;if(!a)return "";return "上下文 "+cpuF(r)+"/"+cpuF(a)+" ("+cpuPct(r,a)+")"}
function cpuHtml(e){const t=e.input||0,n=e.cachedReadTokens||e.cacheReadTokens||0,o=e.totalEstimated?"(估算)":"",c=cpuContextText(e);return'<div class="cpu-line">本轮调用合计 <strong>'+cpuF(e.total)+o+"</strong> · 输入 "+cpuF(e.input)+" · 输出 "+cpuF(e.output)+" · 缓存写 "+cpuCacheText(e.cacheCreationKnown,e.cacheCreationTokens)+" · 缓存读 "+cpuCacheText(e.cacheReadKnown,n)+" · 缓存命中率 "+cpuRateText(e.cacheReadKnown,n,t,e.cacheCreationTokens||0)+(c?" · "+c:"")+" · 调用 "+cpuF(e.callCount)+" 次 · 耗时 "+((e.elapsed||0)/1000).toFixed(1)+"s · 数据仅供参考</div>"}
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
function cpuRender(){document.querySelectorAll("main>.claude-plus-token-usage,body>.claude-plus-token-usage").forEach(e=>e.remove());let e=document.querySelector(".claude-plus-token-usage");if(!window.__claudePlusEnhanceTokenUsageV1){e&&e.remove();return}if(!cpu.last||!cpu.renderReady){e&&e.remove();return}O();const n=cpuLatestAssistant();if(!n||cpuLooksLikeRunStatus(n)){e&&e.remove();return}const t=cpuMount(n),r=t.parentElement;if(!r)return;const a=cpuAssistantFooter(t);const s=a?.parentElement||r;e&&e.dataset.host!==String(cpu.lastId)&&(e.remove(),e=null);e||(e=document.createElement("div"),e.className="claude-plus-token-usage");e.dataset.host=String(cpu.lastId);e.dataset.scopeKey=cpu.last.scopeKey||"";e.innerHTML=cpuHtml(cpu.last);e.parentElement!==s||e.previousElementSibling!==(a||t)?s.insertBefore(e,a?a.nextSibling:t.nextSibling):0;document.querySelectorAll(".claude-plus-token-usage").forEach(n=>{n!==e&&n.remove()})}
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
        let paths = patch::resolve_claude_paths().ok();
        let resources_path = paths.as_ref().map(|p| p.resources.clone());
        let enabled = resources_path
            .as_ref()
            .map(|path| {
                migrate_feature_versions(path);
                feature_states(path)
            })
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
        feature_definitions()
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

    fn feature_definitions() -> Vec<EnhanceFeatureDefinition> {
        serde_json::from_str(ENHANCE_FEATURES_JSON)
            .expect("embedded enhance feature definitions should be valid")
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

    fn migrate_feature_versions(resources_path: &Path) {
        if let Err(e) = migrate_feature_versions_inner(resources_path) {
            tracing::warn!("Claude Desktop enhance migration skipped: {e}");
        }
    }

    fn migrate_feature_versions_inner(resources_path: &Path) -> Result<(), String> {
        let states = feature_states(resources_path);
        let needs_upgrade: Vec<_> = states
            .into_iter()
            .filter(FeatureState::needs_upgrade)
            .map(|state| state.feature)
            .collect();
        if needs_upgrade.is_empty() {
            return Ok(());
        }

        claude_desktop::stop_claude_processes()?;
        patch::enable_write_access(resources_path, false);
        apply_feature_version_upgrades(resources_path, &needs_upgrade)
    }

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
                fs::write(&path, next).map_err(|e| format!("写入 Claude 页面增强入口失败: {e}"))?;
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
            fs::write(&path, next).map_err(|e| format!("写入 Claude 页面增强入口失败: {e}"))?;
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
        feature_definitions()
            .into_iter()
            .find_map(|definition| (definition.id == id).then_some(definition.version))
            .unwrap_or_else(|| "v0.2".to_string())
    }

    fn update_skills_bridge(
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

    fn update_title_i18n_bridge(
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

    fn update_token_usage_bridge(
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

    struct BridgePatch<'a> {
        file_path: &'a str,
        script: &'a str,
        remover: fn(&str) -> String,
    }

    fn patch_bridge_files(
        resources_path: &Path,
        patches: &[BridgePatch<'_>],
        backup: &mut patch::BackupContext,
        enabled: bool,
    ) -> Result<(), String> {
        let asar_path = resources_path.join("app.asar");
        let mut data = fs::read(&asar_path).map_err(|e| format!("读取 app.asar 失败: {e}"))?;
        let parsed = patch::read_asar_header(&data, &asar_path)?;
        let mut header: Value = serde_json::from_str(&parsed.header_string)
            .map_err(|e| format!("解析 app.asar 头失败: {e}"))?;
        let mut pending = Vec::new();

        for patch_spec in patches {
            let entry = patch::get_required_asar_entry(&header, patch_spec.file_path)?;
            let offset = patch::entry_value_to_usize(entry.get("offset"), "offset")?;
            let old_size = patch::entry_value_to_usize(entry.get("size"), "size")?;
            let content_offset = 8 + parsed.header_size + offset;
            let content_end = content_offset + old_size;
            if content_end > data.len() {
                return Err("app.asar 目标文件边界无效".to_string());
            }
            let content = &data[content_offset..content_end];
            if let Some(patched_content) =
                patch_bridge_content(content, patch_spec.script, enabled, patch_spec.remover)?
            {
                pending.push(PendingAsarPatch {
                    file_path: patch_spec.file_path,
                    offset,
                    old_size,
                    content_offset,
                    content_end,
                    patched_content,
                });
            }
        }

        if pending.is_empty() {
            return Ok(());
        }

        pending.sort_by(|a, b| b.offset.cmp(&a.offset));
        backup.backup_resource(&asar_path)?;
        for pending_patch in pending {
            let entry = patch::get_asar_entry_mut(&mut header, pending_patch.file_path)?;
            entry["size"] = Value::Number((pending_patch.patched_content.len() as u64).into());
            entry["integrity"] = patch::asar_file_integrity(&pending_patch.patched_content);
            patch::shift_asar_offsets_after(
                &mut header,
                pending_patch.offset,
                pending_patch.patched_content.len() as i64 - pending_patch.old_size as i64,
            )?;
            data.splice(
                pending_patch.content_offset..pending_patch.content_end,
                pending_patch.patched_content.iter().copied(),
            );
        }

        let header_string =
            serde_json::to_string(&header).map_err(|e| format!("生成 app.asar 头失败: {e}"))?;
        let encoded_header = patch::encode_asar_header(&header_string, Some(parsed.header_size))?;
        let content_start = 8 + parsed.header_size;
        let mut next_data = Vec::with_capacity(encoded_header.len() + data.len() - content_start);
        next_data.extend_from_slice(&encoded_header);
        next_data.extend_from_slice(&data[content_start..]);
        data = next_data;
        fs::write(&asar_path, data).map_err(|e| format!("写入 app.asar 失败: {e}"))?;
        patch::sync_claude_exe_asar_integrity(resources_path, Some(&header_string), Some(backup))?;
        Ok(())
    }

    struct PendingAsarPatch<'a> {
        file_path: &'a str,
        offset: usize,
        old_size: usize,
        content_offset: usize,
        content_end: usize,
        patched_content: Vec<u8>,
    }

    #[cfg(test)]
    fn patch_bridge_files_for_test(
        contents: Vec<(&str, &str)>,
        enabled: bool,
        remover: fn(&str) -> String,
        script: &str,
    ) -> Result<Vec<String>, String> {
        contents
            .into_iter()
            .map(|(_, content)| {
                patch_bridge_content(content.as_bytes(), script, enabled, remover).map(|patched| {
                    String::from_utf8(patched.unwrap_or_else(|| content.as_bytes().to_vec()))
                        .expect("patched bridge should be utf8")
                })
            })
            .collect()
    }

    fn patch_bridge_content(
        content: &[u8],
        script: &str,
        enabled: bool,
        remover: fn(&str) -> String,
    ) -> Result<Option<Vec<u8>>, String> {
        let text =
            std::str::from_utf8(content).map_err(|e| format!("preload 入口不是 UTF-8: {e}"))?;
        let mut next = remover(text);
        if enabled {
            next.insert_str(0, script);
        }
        if next == text {
            Ok(None)
        } else {
            Ok(Some(next.into_bytes()))
        }
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

    fn remove_title_i18n_bridge(text: &str) -> String {
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

    fn remove_token_usage_bridge(text: &str) -> String {
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

    fn title_i18n_bridge_installed(resources_path: &Path) -> bool {
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

    fn token_usage_bridge_installed(resources_path: &Path) -> bool {
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

    #[cfg(test)]
    mod tests {
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

        fn feature_state(
            states: &[super::FeatureState],
            feature: EnhanceFeatureId,
        ) -> super::FeatureState {
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

            super::refresh_enabled_features_for_locale(
                &resources,
                super::EnhanceScriptLocale::EnUs,
            )
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
                    "plugins" => "v0.3",
                    "conversation_title_i18n" | "token_usage" => "v0.4",
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
            assert!(super::title_i18n_main_bridge_script()
                .contains("/claude-plus/conversation-title-i18n"));
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
            assert!(!INJECT_SCRIPT.contains(r#"id:"mcp",marker:"__claudePlusEnhanceMcpV1",label:"MCP",path:"/customize/connectors""#));
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
            assert!(INJECT_SCRIPT
                .contains(r#"querySelectorAll('button,[role="menuitem"],[cmdk-item]')"#));
            assert!(!INJECT_SCRIPT
                .contains(r#"querySelectorAll('button,[role="menuitem"],[cmdk-item],div')"#));
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
            assert!(INJECT_SCRIPT.contains("n=e.cachedReadTokens||e.cacheReadTokens||0"));
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
            assert!(INJECT_SCRIPT
                .contains("[\"submit\",\"click\",\"keydown\"].forEach(e=>document.addEventListener(e,cpuStart,!0))"));
            assert!(INJECT_SCRIPT.contains("cpu.pollBusy||(!e&&n-cpu.lastPollAt<350)"));
            assert!(!INJECT_SCRIPT.contains("setInterval(()=>cpuPoll(!0),1200)"));
            assert!(!INJECT_SCRIPT.contains("http://127.0.0.1:"));
            assert!(!INJECT_SCRIPT.contains("__CPP_TOKEN_USAGE_URL__"));
            assert!(!INJECT_SCRIPT.contains("function cpuHost"));
            assert!(!INJECT_SCRIPT.contains("n.appendChild(e)"));
            assert!(!INJECT_SCRIPT.contains("Token 使用信息：等待下一次"));
            assert!(!INJECT_SCRIPT
                .contains("document.querySelector(\"textarea,[contenteditable='true']\")"));

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
            assert!(INJECT_SCRIPT
                .contains("cpuRateText(e.cacheReadKnown,n,t,e.cacheCreationTokens||0)"));
            assert!(INJECT_SCRIPT.contains("function cpuContextText"));
            assert!(INJECT_SCRIPT.contains("if(!a)return \"\""));
            assert!(INJECT_SCRIPT.contains("return \"上下文 \"+cpuF(r)+\"/\"+cpuF(a)"));
            assert!(INJECT_SCRIPT.contains("c?\" · \"+c:\"\""));
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
                serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
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

            let language = include_str!("../resources/nsis/SimpChinese.nsh");
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
            assert!(script.contains("return trashSkill(e)"));
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
            let paths = crate::claude_patch_common::resolve_claude_paths()
                .expect("resolve Claude Desktop paths");
            let text =
                super::read_index_bundle(&paths.resources).expect("read installed index bundle");
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
    }

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

    fn read_asar_file(resources_path: &Path, file_path: &str) -> Result<Vec<u8>, String> {
        let asar_path = resources_path.join("app.asar");
        let data = fs::read(&asar_path).map_err(|e| format!("读取 app.asar 失败: {e}"))?;
        let parsed = patch::read_asar_header(&data, &asar_path)?;
        let header: Value = serde_json::from_str(&parsed.header_string)
            .map_err(|e| format!("解析 app.asar 头失败: {e}"))?;
        let entry = patch::get_required_asar_entry(&header, file_path)?;
        let offset = patch::entry_value_to_usize(entry.get("offset"), "offset")?;
        let size = patch::entry_value_to_usize(entry.get("size"), "size")?;
        let content_offset = 8 + parsed.header_size + offset;
        let content_end = content_offset + size;
        if content_end > data.len() {
            return Err("app.asar 目标文件边界无效".to_string());
        }
        Ok(data[content_offset..content_end].to_vec())
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
