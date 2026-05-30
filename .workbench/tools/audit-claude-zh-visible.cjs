const fs = require("fs");
const path = require("path");
const childProcess = require("child_process");

const claudeResources = process.argv[2] || findClaudeResources();
const packDir = process.argv[3] || path.resolve("src-tauri/resources/claude-zh");

const en = JSON.parse(
  fs.readFileSync(path.join(claudeResources, "ion-dist/i18n/en-US.json"), "utf8"),
);
const zh = JSON.parse(fs.readFileSync(path.join(packDir, "frontend-zh-CN.json"), "utf8"));
const overridePath = path.join(packDir, "frontend-visible-overrides-zh-CN.json");
const overrides = fs.existsSync(overridePath)
  ? JSON.parse(fs.readFileSync(overridePath, "utf8"))
  : {};

const cjk = /[\u4e00-\u9fff]/;
const englishWord = /[A-Za-z]{3,}/;
const technicalOnly =
  /^(?:[A-Z][A-Za-z0-9+.#-]*|[a-z]+(?:-[a-z0-9]+)*|\{[^}]+\}|\([^)]*\)|[\d.,/%: +#-]+|[A-Z]{2,}|[\w.-]+@[\w.-]+)$/;
const allowVisibleEnglish = [
  /^Claude\b/,
  /^Anthropic\b/,
  /^Google\b/,
  /^GitHub\b/,
  /^Gmail$/,
  /^Amazon Bedrock$/,
  /^Azure AI Foundry$/,
  /^AWS Bedrock$/,
  /^Microsoft Foundry$/,
  /^Slack$/,
  /^Asana$/,
  /^Linear$/,
  /^Notion$/,
  /^HubSpot$/,
  /^Canva$/,
  /^Python$/,
  /^JavaScript$/,
  /^TypeScript$/,
  /^React$/,
  /^Node\.js$/,
  /^SQL$/,
  /^CSV$/,
  /^JSON$/,
  /^Markdown$/,
  /^Haiku$/,
  /^Sonnet$/,
  /^Opus$/,
  /^Max$/,
  /^Pro$/,
  /^Team$/,
  /^Enterprise$/,
  /^Free$/,
  /^KB$/,
  /^MB$/,
  /^GB$/,
  /^Linux \(x64\)$/,
  /^Windows \(x64\)$/,
  /^Windows \(arm64\)$/,
  /^macOS$/,
  /^Latin-1 \(ISO-8859-1\)$/,
  /^status\.claude\.com$/,
  /^https?:\/\//,
  /^\.claude\.app$/,
  /^website\.com$/,
  /^Ctrl⏎$/,
  /^⌘ Enter\b/,
  /^Enter\b.*Esc\b/,
  /^CI \{done\}\/\{total\}$/,
  /^OTEL export$/,
  /^ACS URL$/,
  /^\{[^}]+\}(?:\s*[/%]\s*\{[^}]+\})?$/,
  /^[-+]?[$€£¥]?\{[^}]+\}$/,
  /^[-+]\s?\{[^}]+\}$/,
  /^#\{[^}]+\}$/,
  /^v\{[^}]+\}$/,
  /^\{[^}]+\}\s+\(\{[^}]+\}\)$/,
  /^\{size\} (?:KB|MB|GB)$/,
  /^\{(?:percent|progress)\}%$/,
  /^\{firstPart\}\{lineBreak\}\{goalPart\}$/,
  /^\{name\},$/,
  /^SSH · \{sshHost\}$/,
  /^\{status\} · \{ref\}$/,
  /^PR #\{prNumber\}$/,
  /^\{count, plural, one \{# \{singular\}\} other \{# \{plural\}\}\}$/,
  /^X \/ Twitter$/,
  /^−\{amount\}$/,
  /^\{filePath\}:\{lineNumber\}$/,
  /^\{pct\}%$/,
  /^\+ JCT$/,
  /^© \d{4} ANTHROPIC PBC$/,
  /^\[Ant-only\] /,
  /^1 = \{[^}]+\}, \{[^}]+\} = \{[^}]+\}$/,
  /^\{[^}]+\} \{[^}]+\}$/,
  /^\([^)]*\)$/,
];

function shouldIgnore(value) {
  const s = String(value).trim();
  if (!s || cjk.test(s) || !englishWord.test(s)) return true;
  if (allowVisibleEnglish.some((re) => re.test(s))) return true;
  if (technicalOnly.test(s) && s.length < 30) return true;
  return false;
}

const rows = [];
for (const key of Object.keys(en)) {
  const value = en[key];
  if (typeof value !== "string") continue;
  const translated = Object.prototype.hasOwnProperty.call(overrides, key)
    ? overrides[key]
    : Object.prototype.hasOwnProperty.call(zh, key)
      ? zh[key]
      : value;
  if (translated === value && !shouldIgnore(value)) rows.push({ key, value });
}

console.log(JSON.stringify({ count: rows.length, rows }, null, 2));

function findClaudeResources() {
  const roots = [
    process.env.ProgramW6432,
    process.env.ProgramFiles,
    "C:/Program Files",
  ].filter(Boolean);
  const candidates = [];
  for (const root of roots) {
    const windowsApps = path.join(root, "WindowsApps");
    if (!fs.existsSync(windowsApps)) continue;
    let names = [];
    try {
      names = fs.readdirSync(windowsApps);
    } catch {
      names = listWindowsAppsWithPowerShell(windowsApps);
    }
    for (const name of names) {
      if (!name.startsWith("Claude_")) continue;
      const resources = path.join(windowsApps, name, "app/resources");
      if (fs.existsSync(path.join(resources, "ion-dist/i18n/en-US.json"))) {
        const stat = fs.statSync(path.join(windowsApps, name));
        candidates.push({ resources, mtime: stat.mtimeMs });
      }
    }
  }
  candidates.sort((a, b) => b.mtime - a.mtime);
  if (candidates[0]) return candidates[0].resources;
  throw new Error("Cannot find Claude Desktop resources. Pass the resources path as argv[2].");
}

function listWindowsAppsWithPowerShell(windowsApps) {
  const quoted = windowsApps.replace(/'/g, "''");
  const command = `Get-ChildItem '${quoted}\\\\Claude_*' -Directory -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | ForEach-Object { $_.Name }`;
  for (const exe of ["pwsh.exe", "powershell.exe"]) {
    try {
      return childProcess
        .execFileSync(exe, ["-NoProfile", "-Command", command], {
          encoding: "utf8",
          windowsHide: true,
        })
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean);
    } catch {
      // Try the next shell.
    }
  }
  return [];
}
