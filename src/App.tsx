import { useEffect, useState, useCallback, useRef, type ComponentType } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  BookOpen,
  Languages,
  LayoutDashboard,
  MonitorCog,
  PlugZap,
  RefreshCw,
  RotateCw,
  ShieldCheck,
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

type Route = "overview" | "models" | "desktop" | "localization";
type Icon = ComponentType<LucideProps>;
type CommandArgs = Record<string, unknown>;

const routes: Array<{ id: Route; label: string; icon: Icon }> = [
  { id: "overview", label: "概览", icon: LayoutDashboard },
  { id: "models", label: "模型映射", icon: Table2 },
  { id: "desktop", label: "Claude Desktop", icon: MonitorCog },
  { id: "localization", label: "一键汉化", icon: Languages },
];

const routeMeta: Record<Route, { title: string; subtitle: string }> = {
  overview: {
    title: "Claude++ 概览",
    subtitle: "检查代理、Claude Desktop 接入、模型映射和汉化状态。",
  },
  models: {
    title: "模型映射",
    subtitle: "从 CC Switch 读取当前 Claude Desktop 服务商，并展示角色到实际模型的转发关系。",
  },
  desktop: {
    title: "Claude Desktop 接入",
    subtitle: "管理 Claude Desktop 的配置接入、撤销和重启刷新。",
  },
  localization: {
    title: "Claude Desktop 一键汉化",
    subtitle: "安装或恢复 Claude Desktop 语言资源，保留备份和安全模式入口。",
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

function App() {
  const [route, setRoute] = useState<Route>("overview");
  const [status, setStatus] = useState<StatusInfo | null>(null);
  const [pm, setPm] = useState<ProviderMappings | null>(null);
  const [zhStatus, setZhStatus] = useState<ClaudeZhStatus | null>(null);
  const [zhLanguage, setZhLanguage] = useState("zh-CN");
  const [skipAsarPatch, setSkipAsarPatch] = useState(false);
  const [err, setErr] = useState<string>("");
  const [busy, setBusy] = useState(false);
  const [restartNeeded, setRestartNeeded] = useState(false);
  const lastMappingFingerprint = useRef<string | null>(null);

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
      : "未检测到";

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brandMark">C++</div>
          <div>
            <div className="brandTitle">Claude++</div>
            <div className="brandSub">CC Switch 桥接管理</div>
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
            <>
              <section className="heroPanel">
                <div>
                  <span className="eyebrow">Claude Desktop Gateway</span>
                  <h2>让 Claude Desktop 使用 CC Switch 当前模型配置</h2>
                  <p>
                    Claude++ 保持本地代理运行，读取 CC Switch 的 Claude Desktop 映射，
                    并为 Claude Desktop 提供模型发现与请求转发。
                  </p>
                </div>
                <div className="heroActions">
                  <button
                    className="primary"
                    disabled={busy}
                    onClick={() => run(status?.running ? "stop_proxy" : "start_proxy")}
                  >
                    {status?.running ? "停止代理" : "启动代理"}
                  </button>
                  <button
                    disabled={busy}
                    onClick={() =>
                      status?.cd_applied ? run("revert_cd_config") : run("apply_cd_config")
                    }
                  >
                    {status?.cd_applied ? "撤销接入" : "接入 Claude Desktop"}
                  </button>
                </div>
              </section>

              <section className="summaryGrid">
                <StatusTile
                  icon={PlugZap}
                  title="代理"
                  value={proxyText}
                  state={status?.running ? "ok" : "idle"}
                />
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
                <StatusTile
                  icon={Languages}
                  title="汉化"
                  value={zhText}
                  state={zhStatus?.installed ? "ok" : zhStatus?.claude_found ? "idle" : "warn"}
                />
              </section>

              <div className="grid two">
                <DesktopPanel
                  busy={busy}
                  status={status}
                  restartNeeded={restartNeeded}
                  run={run}
                  restartClaudeDesktop={restartClaudeDesktop}
                />
                <LocalizationPanel
                  compact
                  busy={busy}
                  zhStatus={zhStatus}
                  zhLanguage={zhLanguage}
                  skipAsarPatch={skipAsarPatch}
                  setZhLanguage={setZhLanguage}
                  setSkipAsarPatch={setSkipAsarPatch}
                  installClaudeZh={installClaudeZh}
                  uninstallClaudeZh={uninstallClaudeZh}
                />
              </div>
            </>
          )}

          {route === "models" && (
            <section className="panel">
              <div className="panelHead">
                <div>
                  <h2>当前服务商</h2>
                  <p>{pm?.provider_name ?? "读取失败"}</p>
                </div>
                <button className="iconButton" disabled={busy} onClick={refresh}>
                  <RefreshCw size={16} />
                  <span>重新读取</span>
                </button>
              </div>
              <ModelTable mappings={pm?.mappings ?? []} />
            </section>
          )}

          {route === "desktop" && (
            <DesktopPanel
              large
              busy={busy}
              status={status}
              restartNeeded={restartNeeded}
              run={run}
              restartClaudeDesktop={restartClaudeDesktop}
            />
          )}

          {route === "localization" && (
            <LocalizationPanel
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
        </section>
      </main>
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

function DesktopPanel({
  busy,
  status,
  restartNeeded,
  run,
  restartClaudeDesktop,
  large = false,
}: {
  busy: boolean;
  status: StatusInfo | null;
  restartNeeded: boolean;
  run: (cmd: string) => Promise<void>;
  restartClaudeDesktop: () => Promise<void>;
  large?: boolean;
}) {
  return (
    <section className={`panel ${large ? "largePanel" : ""}`}>
      <div className="panelHead">
        <div>
          <h2>Claude Desktop 接入</h2>
          <p>代理运行后，Claude Desktop 会通过 Claude++ 读取模型列表。</p>
        </div>
        <ShieldCheck size={20} />
      </div>

      <div className="settingList">
        <StatusRow
          label="本地代理"
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
            ? "已检测到 CC Switch 模型或服务商变化。Claude++ 已同步，Claude Desktop 需要重启后刷新模型列表。"
            : "Claude Desktop 的模型列表只在启动时发现；CC Switch 切换后请重启 Claude Desktop。"}
        </div>
      )}
    </section>
  );
}

function LocalizationPanel({
  busy,
  zhStatus,
  zhLanguage,
  skipAsarPatch,
  setZhLanguage,
  setSkipAsarPatch,
  installClaudeZh,
  uninstallClaudeZh,
  compact = false,
}: {
  busy: boolean;
  zhStatus: ClaudeZhStatus | null;
  zhLanguage: string;
  skipAsarPatch: boolean;
  setZhLanguage: (value: string) => void;
  setSkipAsarPatch: (value: boolean) => void;
  installClaudeZh: () => Promise<void>;
  uninstallClaudeZh: () => Promise<void>;
  compact?: boolean;
}) {
  const statusText = zhStatus?.installed
    ? `已安装 ${zhStatus.language_files.join(", ")}`
    : zhStatus?.claude_found
      ? "已检测到 Claude Desktop，尚未安装汉化"
      : "未检测到 Claude Desktop";

  return (
    <section className={`panel localizationPanel ${compact ? "compactPanel" : "largePanel"}`}>
      <div className="panelHead">
        <div>
          <h2>一键汉化</h2>
          <p>{statusText}</p>
        </div>
        <Languages size={20} />
      </div>

      <div className="formGrid">
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

      <div className="notice">
        当前语言: {zhStatus?.locale ?? "未设置"}。汉化会关闭 Claude Desktop，写入语言资源并重启。
      </div>
    </section>
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

function ModelTable({ mappings }: { mappings: Mapping[] }) {
  return (
    <div className="tableWrap">
      <table className="map">
        <thead>
          <tr>
            <th>模型角色</th>
            <th>菜单显示名</th>
            <th>实际请求模型</th>
          </tr>
        </thead>
        <tbody>
          {mappings.map((m) => (
            <tr key={m.role}>
              <td>{m.role_kind}</td>
              <td>{m.display}</td>
              <td>{m.model}</td>
            </tr>
          ))}
          {!mappings.length && (
            <tr>
              <td colSpan={3} className="empty">
                无映射
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

export default App;
