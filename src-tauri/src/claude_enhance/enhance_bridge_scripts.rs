use crate::constants::DEFAULT_PROXY_PORT;

use super::{
    enhance_injected::EnhanceScriptLocale, SKILLS_LIST_CHANNEL, SKILLS_TRASH_CHANNEL,
    TITLE_I18N_BRIDGE_MARKER, TITLE_I18N_CHANNEL, TITLE_I18N_MAIN_BRIDGE_MARKER,
    TOKEN_USAGE_BRIDGE_MARKER, TOKEN_USAGE_CHANNEL, TOKEN_USAGE_MAIN_BRIDGE_MARKER,
};

fn local_gateway_runtime_js() -> String {
    format!(
        r#"function cppPort(){{try{{const e=process.env.CLAUDE_PLUS_PROXY_PORT;if(e&&/^\d+$/.test(String(e).trim())){{const t=Number(String(e).trim());if(t>0&&t<65536)return t}}}}catch{{}}try{{const e=JSON.parse(fs.readFileSync(path.join(process.env.USERPROFILE||process.env.HOME||"",".claude-plus-plus","settings.json"),"utf8")),t=e.proxyPort??e.proxy_port;if(t!=null&&/^\d+$/.test(String(t).trim())){{const n=Number(String(t).trim());if(n>0&&n<65536)return n}}}}catch{{}}return {DEFAULT_PROXY_PORT}}}function cppUrl(e){{return "http://127.0.0.1:"+cppPort()+e}}"#
    )
}

fn local_gateway_token_js() -> &'static str {
    r#"function cppToken(){try{return fs.readFileSync(path.join(process.env.USERPROFILE||process.env.HOME||"",".claude-plus-plus","local-gateway-token"),"utf8").trim()}catch{return""}}"#
}

pub(crate) fn skills_bridge_script() -> String {
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

pub(crate) fn skills_main_bridge_script(locale: EnhanceScriptLocale) -> String {
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
async function trashSkillFast(e){return await gatewayTrash(e)}
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

pub(crate) fn title_i18n_preload_bridge_script() -> String {
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

pub(crate) fn title_i18n_main_bridge_script() -> String {
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

pub(crate) fn token_usage_preload_bridge_script() -> String {
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

pub(crate) fn token_usage_main_bridge_script() -> String {
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
