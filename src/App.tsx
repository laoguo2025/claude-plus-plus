import { useEffect, useState, useCallback, useRef, type ComponentType } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Activity,
  BookOpen,
  FileText,
  Hammer,
  Info,
  Languages,
  LayoutDashboard,
  MonitorCog,
  Moon,
  PlugZap,
  RefreshCw,
  RotateCw,
  Sun,
  Table2,
  type LucideProps,
} from "lucide-react";
import "./App.css";

interface Mapping {
  display: string;
  role: string;
  role_kind: string;
  model: string;
}
interface ProviderMappings {
  provider_name: string;
  provider_id: string;
  mappings: Mapping[];
}
interface StatusInfo {
  running: boolean;
  port: number | null;
  cd_applied: boolean;
}
interface ClaudeZhStatus {
  supported: boolean;
  claude_found: boolean;
  installed: boolean;
  backup_available: boolean;
  install_path: string | null;
  resources_path: string | null;
  locale: string | null;
  language_files: string[];
}

type Route = "overview" | "localization" | "enhance" | "about" | "diagnostics";
type Theme = "light" | "dark";
type Icon = ComponentType<LucideProps>;
type CommandArgs = Record<string, unknown>;

const routes: Array<{ id: Route; label: string; icon: Icon }> = [
  { id: "overview", label: "系统概览", icon: LayoutDashboard },
  { id: "localization", label: "一键汉化", icon: Languages },
  { id: "enhance", label: "页面增强", icon: Hammer },
  { id: "about", label: "关于工具", icon: Info },
  { id: "diagnostics", label: "诊断日志", icon: FileText },
];

const routeMeta: Record<Route, { title: string; subtitle: string }> = {
  overview: {
    title: "系统概览",
    subtitle: "代理、Claude Desktop 接入和模型映射状态。",
  },
  localization: {
    title: "一键汉化",
    subtitle: "安装或恢复 Claude Desktop 语言资源。",
  },
  enhance: {
    title: "页面增强",
    subtitle: "预留 Claude Desktop 页面增强能力入口。",
  },
  about: {
    title: "关于工具",
    subtitle: "版本、边界和界面主题。",
  },
  diagnostics: {
    title: "诊断日志",
    subtitle: "当前运行状态、最近错误和排查信息。",
  },
};

const isTauriRuntime = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function callCommand<T>(cmd: string, args?: CommandArgs): Promise<T> {
  if (isTauriRuntime()) return invoke<T>(cmd, args);
  return previewCommand<T>(cmd);
}

function previewCommand<T>(cmd: string): T {
  if (cmd === "proxy_status") {
    return { running: true, port: 15722, cd_applied: true } as T;
  }
  if (cmd === "get_mappings") {
    return {
      provider_name: "CC Switch 当前服务商",
      provider_id: "preview",
      mappings: [
        {
          display: "Opus - mimo-v2.5-pro",
          role: "opus",
          role_kind: "Opus",
          model: "mimo-v2.5-pro",
        },
        {
          display: "Sonnet - mimo-v2.5",
          role: "sonnet",
          role_kind: "Sonnet",
          model: "mimo-v2.5",
        },
      ],
    } as T;
  }
  if (cmd === "claude_zh_status") {
    return {
      supported: true,
      claude_found: true,
      installed: false,
      backup_available: true,
      install_path: null,
      resources_path: null,
      locale: "en-US",
      language_files: [],
    } as T;
  }
  return undefined as T;
}

function loadInitialTheme(): Theme {
  if (typeof window === "undefined") return "light";
  return window.localStorage.getItem("claude-plus-theme") === "dark" ? "dark" : "light";
}

function App() {
  const [route, setRoute] = useState<Route>("overview");
  const [theme, setTheme] = useState<Theme>(() => loadInitialTheme());
  const [status, setStatus] = useState<StatusInfo | null>(null);
  const [pm, setPm] = useState<ProviderMappings | null>(null);
  const [zhStatus, setZhStatus] = useState<ClaudeZhStatus | null>(null);
  const [zhLanguage, setZhLanguage] = useState("zh-CN");
  const [skipAsarPatch, setSkipAsarPatch] = useState(false);
  const [err, setErr] = useState<string>("");
  const [busy, setBusy] = useState(false);
  const [restartNeeded, setRestartNeeded] = useState(false);
  const lastMappingFingerprint = useRef<string | null>(null);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem("claude-plus-theme", theme);
  }, [theme]);

  const refresh = useCallback(async () => {
    setErr("");
    try {
      setStatus(await callCommand<StatusInfo>("proxy_status"));
    } catch (e) {
      setErr(String(e));
    }
    try {
      const nextPm = await callCommand<ProviderMappings>("get_mappings");
      const nextFingerprint = JSON.stringify({
        provider_id: nextPm.provider_id,
        mappings: nextPm.mappings,
      });
      if (
        lastMappingFingerprint.current !== null &&
        lastMappingFingerprint.current !== nextFingerprint
      ) {
        setRestartNeeded(true);
      }
      lastMappingFingerprint.current = nextFingerprint;
      setPm(nextPm);
    } catch (e) {
      setErr(String(e));
      setPm(null);
    }
    try {
      setZhStatus(await callCommand<ClaudeZhStatus>("claude_zh_status"));
    } catch (e) {
      setErr(String(e));
      setZhStatus(null);
    }
  }, []);

  useEffect(() => {
    refresh();
    const t = setInterval(refresh, 4000);
    return () => clearInterval(t);
  }, [refresh]);

  const run = async (cmd: string) => {
    setBusy(true);
    setErr("");
    try {
      await callCommand(cmd);
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const restartClaudeDesktop = async () => {
    setBusy(true);
    setErr("");
    try {
      await callCommand("restart_claude_desktop");
      setRestartNeeded(false);
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const installClaudeZh = async () => {
    setBusy(true);
    setErr("");
    try {
      await callCommand("install_claude_zh", {
        language: zhLanguage,
        skipAsarPatch,
      });
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const uninstallClaudeZh = async () => {
    setBusy(true);
    setErr("");
    try {
      await callCommand("uninstall_claude_zh");
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const meta = routeMeta[route];
  const proxyText = status?.running ? `127.0.0.1:${status.port}` : "未运行";
  const desktopText = status?.cd_applied ? "已接入" : "未接入";
  const zhText = zhStatus?.installed
    ? `已安装 ${zhStatus.language_files.join(", ")}`
    : zhStatus?.claude_found
      ? "未安装"
      : "未检测到 Claude Desktop";

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brandMark">C++</div>
          <div>
            <div className="brandTitle">Claude++</div>
            <div className="brandSub">管理控制台</div>
          </div>
        </div>
        <nav className="nav" aria-label="主导航">
          {routes.map((item) => {
            const IconComponent = item.icon;
            return (
              <button
                key={item.id}
                className={`navItem ${route === item.id ? "active" : ""}`}
                onClick={() => setRoute(item.id)}
              >
                <IconComponent size={17} />
                <span>{item.label}</span>
              </button>
            );
          })}
        </nav>
      </aside>

      <main className="workspace">
        <header className="topbar">
          <div>
            <h1>{meta.title}</h1>
            <p>{meta.subtitle}</p>
          </div>
          <button className="iconButton" disabled={busy} onClick={refresh} title="刷新">
            <RefreshCw size={16} />
            <span>刷新</span>
          </button>
        </header>

        <section className="screen">
          {err && <div className="errorBanner">{err}</div>}

          {route === "overview" && (
            <OverviewPage
              busy={busy}
              status={status}
              pm={pm}
              proxyText={proxyText}
              desktopText={desktopText}
              restartNeeded={restartNeeded}
              run={run}
              restartClaudeDesktop={restartClaudeDesktop}
            />
          )}

          {route === "localization" && (
            <LocalizationPage
              busy={busy}
              zhStatus={zhStatus}
              zhLanguage={zhLanguage}
              skipAsarPatch={skipAsarPatch}
              setZhLanguage={setZhLanguage}
              setSkipAsarPatch={setSkipAsarPatch}
              installClaudeZh={installClaudeZh}
              uninstallClaudeZh={uninstallClaudeZh}
            />
          )}

          {route === "enhance" && <EnhancePage />}

          {route === "about" && (
            <AboutPage
              theme={theme}
              setTheme={setTheme}
              proxyText={proxyText}
              desktopText={desktopText}
              zhText={zhText}
            />
          )}

          {route === "diagnostics" && (
            <DiagnosticsPage
              err={err}
              status={status}
              pm={pm}
              zhStatus={zhStatus}
              restartNeeded={restartNeeded}
            />
          )}
        </section>
      </main>
    </div>
  );
}

function OverviewPage({
  busy,
  status,
  pm,
  proxyText,
  desktopText,
  restartNeeded,
  run,
  restartClaudeDesktop,
}: {
  busy: boolean;
  status: StatusInfo | null;
  pm: ProviderMappings | null;
  proxyText: string;
  desktopText: string;
  restartNeeded: boolean;
  run: (cmd: string) => Promise<void>;
  restartClaudeDesktop: () => Promise<void>;
}) {
  return (
    <div className="pageGrid overviewPage">
      <section className="summaryGrid">
        <StatusTile icon={PlugZap} title="本地代理" value={proxyText} state={status?.running ? "ok" : "idle"} />
        <StatusTile
          icon={MonitorCog}
          title="Claude Desktop"
          value={desktopText}
          state={status?.cd_applied ? "ok" : "idle"}
        />
        <StatusTile
          icon={BookOpen}
          title="当前服务商"
          value={pm?.provider_name ?? "读取失败"}
          state={pm ? "ok" : "warn"}
        />
      </section>

      <section className="panel controlPanel">
        <div className="panelHead">
          <div>
            <h2>接入控制</h2>
            <p>保持代理运行，并把 Claude Desktop 指向 Claude++。</p>
          </div>
          <MonitorCog size={20} />
        </div>
        <div className="controlRows">
          <StatusRow
            label="代理服务"
            value={status?.running ? `运行中，端口 ${status.port}` : "已停止"}
            active={!!status?.running}
            action={
              <button disabled={busy} onClick={() => run(status?.running ? "stop_proxy" : "start_proxy")}>
                {status?.running ? "停止" : "启动"}
              </button>
            }
          />
          <StatusRow
            label="Claude Desktop 配置"
            value={status?.cd_applied ? "已接入 Claude++" : "未接入 Claude++"}
            active={!!status?.cd_applied}
            action={
              <div className="actions">
                {status?.cd_applied && (
                  <button disabled={busy} onClick={restartClaudeDesktop}>
                    <RotateCw size={14} />
                    重启
                  </button>
                )}
                <button
                  disabled={busy}
                  onClick={() => run(status?.cd_applied ? "revert_cd_config" : "apply_cd_config")}
                >
                  {status?.cd_applied ? "撤销接入" : "接入"}
                </button>
              </div>
            }
          />
        </div>
        {status?.cd_applied && (
          <div className={`notice ${restartNeeded ? "warn" : ""}`}>
            {restartNeeded
              ? "已检测到模型或服务商变化；Claude Desktop 需要重启后刷新模型列表。"
              : "Claude Desktop 的模型列表只在启动时发现；切换 CC Switch 后请重启 Claude Desktop。"}
          </div>
        )}
      </section>

      <section className="panel mappingPanel">
        <div className="panelHead">
          <div>
            <h2>模型映射</h2>
            <p>{pm ? `${pm.mappings.length} 个角色映射` : "读取失败"}</p>
          </div>
          <Table2 size={20} />
        </div>
        <MiniMapping mappings={pm?.mappings ?? []} />
      </section>
    </div>
  );
}

function LocalizationPage({
  busy,
  zhStatus,
  zhLanguage,
  skipAsarPatch,
  setZhLanguage,
  setSkipAsarPatch,
  installClaudeZh,
  uninstallClaudeZh,
}: {
  busy: boolean;
  zhStatus: ClaudeZhStatus | null;
  zhLanguage: string;
  skipAsarPatch: boolean;
  setZhLanguage: (value: string) => void;
  setSkipAsarPatch: (value: boolean) => void;
  installClaudeZh: () => Promise<void>;
  uninstallClaudeZh: () => Promise<void>;
}) {
  const statusText = zhStatus?.installed
    ? `已安装 ${zhStatus.language_files.join(", ")}`
    : zhStatus?.claude_found
      ? "已检测到 Claude Desktop，尚未安装汉化"
      : "未检测到 Claude Desktop";

  return (
    <div className="pageGrid localizationPage">
      <section className="panel mainPanel">
        <div className="panelHead">
          <div>
            <h2>Claude Desktop 汉化</h2>
            <p>{statusText}</p>
          </div>
          <Languages size={20} />
        </div>
        <div className="splitForm">
          <label className="field">
            <span>语言</span>
            <select
              disabled={busy}
              value={zhLanguage}
              onChange={(e) => setZhLanguage(e.target.value)}
            >
              <option value="zh-CN">简体中文</option>
              <option value="zh-TW">繁体中文(中国台湾)</option>
              <option value="zh-HK">繁体中文(中国香港)</option>
            </select>
          </label>
          <label className="toggleRow">
            <input
              type="checkbox"
              disabled={busy}
              checked={skipAsarPatch}
              onChange={(e) => setSkipAsarPatch(e.target.checked)}
            />
            <span>
              <strong>安全模式</strong>
              <small>跳过 app.asar 和 Claude.exe 完整性补丁。</small>
            </span>
          </label>
        </div>
        <div className="actions primaryActions">
          <button
            className="primary"
            disabled={busy || !zhStatus?.supported || !zhStatus?.claude_found}
            onClick={installClaudeZh}
          >
            一键汉化
          </button>
          <button disabled={busy || !zhStatus?.backup_available} onClick={uninstallClaudeZh}>
            恢复英文
          </button>
        </div>
      </section>

      <section className="panel statusPanel">
        <KeyValue label="当前语言" value={zhStatus?.locale ?? "未设置"} />
        <KeyValue label="Claude Desktop" value={zhStatus?.claude_found ? "已检测到" : "未检测到"} />
        <KeyValue label="备份状态" value={zhStatus?.backup_available ? "可恢复" : "暂无备份"} />
        <KeyValue label="资源路径" value={zhStatus?.resources_path ?? "未检测到"} />
      </section>
    </div>
  );
}

function EnhancePage() {
  return (
    <section className="panel emptyPage">
      <Hammer size={28} />
      <h2>页面增强</h2>
      <p>该入口先保留为空，后续用于 Claude Desktop 页面增强能力。</p>
      <div className="badge">规划中</div>
    </section>
  );
}

function AboutPage({
  theme,
  setTheme,
  proxyText,
  desktopText,
  zhText,
}: {
  theme: Theme;
  setTheme: (value: Theme) => void;
  proxyText: string;
  desktopText: string;
  zhText: string;
}) {
  return (
    <div className="pageGrid aboutPage">
      <section className="panel mainPanel">
        <div className="panelHead">
          <div>
            <h2>Claude++</h2>
            <p>本地 Claude Desktop 3P 与 CC Switch 桥接管理工具。</p>
          </div>
          <Info size={20} />
        </div>
        <div className="aboutCopy">
          <p>Claude Desktop 指向本机 Claude++ 代理，Claude++ 从 CC Switch 读取当前服务商映射并转发请求。</p>
          <p>一键汉化只修改 Claude Desktop 资源与配置，并保留恢复路径。</p>
        </div>
        <div className="themeRow">
          <div>
            <strong>界面主题</strong>
            <span>切换右侧工作区的浅色或深色显示。</span>
          </div>
          <div className="segmented">
            <button className={theme === "light" ? "active" : ""} onClick={() => setTheme("light")}>
              <Sun size={14} />
              浅色
            </button>
            <button className={theme === "dark" ? "active" : ""} onClick={() => setTheme("dark")}>
              <Moon size={14} />
              深色
            </button>
          </div>
        </div>
      </section>

      <section className="panel statusPanel">
        <KeyValue label="版本" value="0.1.0" />
        <KeyValue label="代理" value={proxyText} />
        <KeyValue label="Claude Desktop" value={desktopText} />
        <KeyValue label="汉化" value={zhText} />
      </section>
    </div>
  );
}

function DiagnosticsPage({
  err,
  status,
  pm,
  zhStatus,
  restartNeeded,
}: {
  err: string;
  status: StatusInfo | null;
  pm: ProviderMappings | null;
  zhStatus: ClaudeZhStatus | null;
  restartNeeded: boolean;
}) {
  const lines = [
    `代理: ${status?.running ? `运行中 127.0.0.1:${status.port}` : "未运行"}`,
    `Claude Desktop: ${status?.cd_applied ? "已接入" : "未接入"}`,
    `服务商: ${pm?.provider_name ?? "读取失败"}`,
    `映射数量: ${pm?.mappings.length ?? 0}`,
    `汉化: ${zhStatus?.installed ? "已安装" : zhStatus?.claude_found ? "未安装" : "未检测到 Claude Desktop"}`,
    `需要重启 Claude Desktop: ${restartNeeded ? "是" : "否"}`,
    `最近错误: ${err || "无"}`,
  ];

  return (
    <div className="pageGrid diagnosticsPage">
      <section className="panel mainPanel">
        <div className="panelHead">
          <div>
            <h2>诊断摘要</h2>
            <p>当前页面基于现有前端状态生成，不读取额外日志文件。</p>
          </div>
          <Activity size={20} />
        </div>
        <div className="logBox">
          {lines.map((line) => (
            <code key={line}>{line}</code>
          ))}
        </div>
      </section>

      <section className="panel statusPanel">
        <KeyValue label="配置来源" value="CC Switch SQLite 当前映射" />
        <KeyValue label="模型刷新" value="Claude Desktop 重启后生效" />
        <KeyValue label="日志能力" value="真实日志读取待接入" />
        <KeyValue label="最近错误" value={err || "无"} />
      </section>
    </div>
  );
}

function StatusTile({
  icon: IconComponent,
  title,
  value,
  state,
}: {
  icon: Icon;
  title: string;
  value: string;
  state: "ok" | "idle" | "warn";
}) {
  return (
    <article className={`statusTile ${state}`}>
      <IconComponent size={18} />
      <div>
        <span>{title}</span>
        <strong>{value}</strong>
      </div>
    </article>
  );
}

function StatusRow({
  label,
  value,
  active,
  action,
}: {
  label: string;
  value: string;
  active: boolean;
  action: React.ReactNode;
}) {
  return (
    <div className="statusRow">
      <span className={`dot ${active ? "on" : "off"}`} />
      <div>
        <strong>{label}</strong>
        <span>{value}</span>
      </div>
      {action}
    </div>
  );
}

function MiniMapping({ mappings }: { mappings: Mapping[] }) {
  const visible = mappings.slice(0, 4);
  return (
    <div className="miniTable">
      {visible.map((m) => (
        <div key={m.role}>
          <span>{m.role_kind}</span>
          <strong>{m.display}</strong>
          <code>{m.model}</code>
        </div>
      ))}
      {!visible.length && <div className="emptyInline">无映射</div>}
    </div>
  );
}

function KeyValue({ label, value }: { label: string; value: string }) {
  return (
    <div className="keyValue">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

export default App;
