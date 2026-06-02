import { useEffect, useState, useCallback, useRef, type ComponentType, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Activity,
  CheckCircle2,
  CircleAlert,
  Code2,
  Gauge,
  ListRestart,
  FileText,
  Hammer,
  Info,
  Languages,
  Link2,
  Network,
  PackageCheck,
  Plug,
  Moon,
  Power,
  RefreshCw,
  Sun,
  type LucideProps,
} from "lucide-react";
import botLogo from "../src-tauri/icons/icon.png";
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
  claude_version: string | null;
  install_path: string | null;
  resources_path: string | null;
  locale: string | null;
  language_files: string[];
}
interface ClaudeEnhanceFeature {
  id: string;
  category: string;
  label: string;
  description: string;
  enabled: boolean;
  available: boolean;
  note: string;
}
interface ClaudeEnhanceStatus {
  supported: boolean;
  claude_found: boolean;
  installed: boolean;
  backup_available: boolean;
  install_path: string | null;
  resources_path: string | null;
  features: ClaudeEnhanceFeature[];
}
interface LogsPayload {
  path: string;
  text: string;
  lines: number;
}
interface DiagnosticsPayload {
  report: string;
}

type Route = "overview" | "localization" | "enhance" | "about" | "diagnostics";
type Theme = "light" | "dark";
type LocalizationScope = "complete" | "safe";
type Icon = ComponentType<LucideProps>;
type CommandArgs = Record<string, unknown>;

const routes: Array<{ id: Route; label: string; icon: Icon }> = [
  { id: "overview", label: "CCS转接", icon: Link2 },
  { id: "localization", label: "一键汉化", icon: Languages },
  { id: "enhance", label: "页面增强", icon: Hammer },
  { id: "diagnostics", label: "诊断日志", icon: FileText },
  { id: "about", label: "关于工具", icon: Info },
];

const routeMeta: Record<Route, { title: string }> = {
  overview: {
    title: "CCS转接",
  },
  localization: {
    title: "一键汉化",
  },
  enhance: {
    title: "页面增强",
  },
  about: {
    title: "关于工具",
  },
  diagnostics: {
    title: "诊断日志",
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
    throw new Error("浏览器预览无法读取 CC Switch 数据库；请在 Claude++ EXE 中查看真实服务商和模型映射。");
  }
  if (cmd === "claude_zh_status") {
    return {
      supported: true,
      claude_found: true,
      installed: false,
      backup_available: true,
      claude_version: "未检测到",
      install_path: null,
      resources_path: null,
      locale: "en-US",
      language_files: [],
    } as T;
  }
  if (cmd === "claude_enhance_status") {
    return {
      supported: true,
      claude_found: true,
      installed: false,
      backup_available: true,
      install_path: null,
      resources_path: null,
      features: [
        {
          id: "third_party_api",
          category: "菜单栏增强",
          label: "第三方API",
          description: "在 Claude Desktop 左侧菜单栏，增加第三方API快捷入口。",
          enabled: false,
          available: true,
          note: "侧边栏软入口",
        },
        {
          id: "plugins",
          category: "菜单栏增强",
          label: "技能",
          description: "在 Claude Desktop 左侧菜单栏，增加全局/项目 skills 快捷入口。",
          enabled: false,
          available: true,
          note: "侧边栏软入口",
        },
        {
          id: "mcp",
          category: "菜单栏增强",
          label: "MCP",
          description: "在 Claude Desktop 左侧菜单栏，增加 MCP 快捷入口。",
          enabled: false,
          available: true,
          note: "侧边栏软入口",
        },
        {
          id: "conversation_title_i18n",
          category: "对话增强",
          label: "对话列表中文化",
          description: "把 Claude Desktop 对话列表里的英文标题自动翻译为中文显示。",
          enabled: false,
          available: true,
          note: "本地代理翻译",
        },
        {
          id: "markdown",
          category: "对话增强",
          label: "对话导出Markdown",
          description: "在对话列表“...”内，增加导出 Markdown 文件按钮。",
          enabled: false,
          available: true,
          note: "导出已加载内容",
        },
        {
          id: "timeline",
          category: "对话增强",
          label: "对话时间线",
          description: "在对话区域右侧，增加用户发言时间线，方便快速定位上下文。",
          enabled: false,
          available: true,
          note: "已加载问题定位",
        },
      ],
    } as T;
  }
  if (
    cmd === "use_claude_plus_route" ||
    cmd === "use_ccs_route" ||
    cmd === "backup_claude_zh" ||
    cmd === "install_claude_enhance" ||
    cmd === "uninstall_claude_enhance"
  ) {
    return undefined as T;
  }
  if (cmd === "read_latest_logs") {
    return {
      path: "C:\\Users\\Administrator\\.claude-plus-plus\\claude-plus-plus.log",
      text: [
        '{"timestamp_ms":1780394324466,"pid":18896,"event":"manager.proxy_status","detail":{"running":true,"port":15722,"cd_applied":true}}',
        '{"timestamp_ms":1780394329499,"pid":18896,"event":"manager.generate_diagnostics","detail":{}}',
      ].join("\n"),
      lines: 200,
    } as T;
  }
  if (cmd === "generate_diagnostics") {
    return {
      report: JSON.stringify(
        {
          generatedAtMs: Date.now(),
          version: "0.1.0",
          overview: {
            app: "Claude++",
            status: { running: true, port: 15722, cd_applied: true },
          },
          paths: {
            ccSwitchDb: "C:\\Users\\Administrator\\.cc-switch\\cc-switch.db",
            diagnosticLog: "C:\\Users\\Administrator\\.claude-plus-plus\\claude-plus-plus.log",
          },
        },
        null,
        2,
      ),
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
  const [mappingError, setMappingError] = useState("");
  const [zhStatus, setZhStatus] = useState<ClaudeZhStatus | null>(null);
  const [enhanceStatus, setEnhanceStatus] = useState<ClaudeEnhanceStatus | null>(null);
  const [zhScope, setZhScope] = useState<LocalizationScope>("complete");
  const [err, setErr] = useState<string>("");
  const [busy, setBusy] = useState(false);
  const [restartNeeded, setRestartNeeded] = useState(false);
  const [logs, setLogs] = useState<LogsPayload | null>(null);
  const [diagnostics, setDiagnostics] = useState<DiagnosticsPayload | null>(null);
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
      setMappingError("");
    } catch (e) {
      setPm(null);
      setMappingError(String(e));
    }
    try {
      setZhStatus(await callCommand<ClaudeZhStatus>("claude_zh_status"));
    } catch (e) {
      setErr(String(e));
      setZhStatus(null);
    }
    try {
      setEnhanceStatus(await callCommand<ClaudeEnhanceStatus>("claude_enhance_status"));
    } catch (e) {
      setErr(String(e));
      setEnhanceStatus(null);
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

  const toggleTheme = () => {
    setTheme((current) => (current === "dark" ? "light" : "dark"));
  };

  const installClaudeZh = async () => {
    setBusy(true);
    setErr("");
    try {
      await callCommand("install_claude_zh", {
        language: "zh-CN",
        skipAsarPatch: zhScope === "safe",
      });
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const backupClaudeZh = async () => {
    setBusy(true);
    setErr("");
    try {
      await callCommand("backup_claude_zh");
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

  const installClaudeEnhance = async (feature: string) => {
    setBusy(true);
    setErr("");
    try {
      await callCommand("install_claude_enhance", { feature });
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const uninstallClaudeEnhance = async (feature: string) => {
    setBusy(true);
    setErr("");
    try {
      await callCommand("uninstall_claude_enhance", { feature });
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const refreshLogs = useCallback(async () => {
    setErr("");
    try {
      setLogs(await callCommand<LogsPayload>("read_latest_logs", { lines: 200 }));
    } catch (e) {
      setErr(String(e));
      setLogs(null);
    }
  }, []);

  const refreshDiagnostics = useCallback(async () => {
    setErr("");
    try {
      setDiagnostics(
        await callCommand<DiagnosticsPayload>("generate_diagnostics", {
          restartNeeded,
        }),
      );
    } catch (e) {
      setErr(String(e));
      setDiagnostics(null);
    }
  }, [restartNeeded]);

  useEffect(() => {
    if (route !== "diagnostics") return;
    refreshLogs();
    refreshDiagnostics();
  }, [route, refreshLogs, refreshDiagnostics]);

  const copyText = async (text: string) => {
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
    } catch (e) {
      setErr(String(e));
    }
  };

  const meta = routeMeta[route];

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brandMark" aria-hidden="true">
            <img src={botLogo} alt="" />
          </div>
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
          </div>
          <div className="topbarActions" aria-label="页面操作">
            <button
              className="iconButton square"
              onClick={toggleTheme}
              title={theme === "dark" ? "切换亮色主题" : "切换暗色主题"}
              aria-label={theme === "dark" ? "切换亮色主题" : "切换暗色主题"}
            >
              {theme === "dark" ? <Sun size={16} /> : <Moon size={16} />}
            </button>
            <button className="iconButton" disabled={busy} onClick={restartClaudeDesktop} title="重启 Claude Desktop">
              <Power size={16} />
              <span>重启 Claude Desktop</span>
            </button>
            <button className="iconButton square" disabled={busy} onClick={refresh} title="刷新本页" aria-label="刷新本页">
              <RefreshCw size={16} />
            </button>
          </div>
        </header>

        <section className="screen">
          {err && <div className="errorBanner">{err}</div>}

          {route === "overview" && (
            <OverviewPage
              busy={busy}
              status={status}
              pm={pm}
              mappingError={mappingError}
              zhStatus={zhStatus}
              run={run}
              restartClaudeDesktop={restartClaudeDesktop}
            />
          )}

          {route === "localization" && (
            <LocalizationPage
              busy={busy}
              zhStatus={zhStatus}
              zhScope={zhScope}
              setZhScope={setZhScope}
              installClaudeZh={installClaudeZh}
              backupClaudeZh={backupClaudeZh}
              uninstallClaudeZh={uninstallClaudeZh}
            />
          )}

          {route === "enhance" && (
            <EnhancePage
              busy={busy}
              enhanceStatus={enhanceStatus}
              installClaudeEnhance={installClaudeEnhance}
              uninstallClaudeEnhance={uninstallClaudeEnhance}
            />
          )}

          {route === "about" && (
            <AboutPage
              claudeDesktopVersion={zhStatus?.claude_version ?? (zhStatus?.claude_found ? "待补充" : "未检测到")}
            />
          )}

          {route === "diagnostics" && (
            <DiagnosticsPage
              diagnostics={diagnostics}
              logs={logs}
              refreshDiagnostics={refreshDiagnostics}
              refreshLogs={refreshLogs}
              copyText={copyText}
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
  mappingError,
  zhStatus,
  run,
  restartClaudeDesktop,
}: {
  busy: boolean;
  status: StatusInfo | null;
  pm: ProviderMappings | null;
  mappingError: string;
  zhStatus: ClaudeZhStatus | null;
  run: (cmd: string) => Promise<void>;
  restartClaudeDesktop: () => Promise<void>;
}) {
  return (
    <div className="pageGrid overviewPage">
      <div className="mechanismNote">
        <span>
          直连 CC Switch 时，模型其实能跑通，但 Claude Desktop 菜单仍会显示 Haiku、Opus、Sonnet 这些原名，看不到
          mimo、DeepSeek、Kimi 等映射名。
        </span>
        <span>
          Claude++ 会读取 CC Switch 当前映射，把更容易看懂的名字交给 Claude Desktop 显示；真正发送请求时，再转回 CC Switch 能识别的模型角色。
        </span>
        <span>例如菜单显示“Opus - mimo-v2.5-pro”；选中后，Claude++ 会按 Opus 档位转发到实际模型。</span>
        <strong>使用期间请保持 Claude++ 运行；CC Switch 增/改/删模型或切换服务商后，重启 Claude Desktop 生效。</strong>
      </div>

      <section className="panel routePanel">
        <div className="panelHead routePanelHead">
          <div>
            <h2>路由转接步骤</h2>
          </div>
          <span className="routeHint">CC Switch 增/改/删模型或切换服务商时，需重启 Claude Desktop</span>
        </div>
        <div className="routeCardBody">
          <RouteStatusCard
            active={!!zhStatus?.claude_found}
            label="Claude Desktop"
            step={1}
            value={zhStatus?.claude_found ? "已安装" : "未安装"}
          />
          <RouteStatusCard active={!!pm} label="CC Switch 路由" step={2} value={pm ? "已开启" : "已断开"} />
          <RouteActionCard
            state={status?.cd_applied ? "on" : "off"}
            disabled={busy}
            label="Claude++ 路由"
            step={3}
            value={status?.cd_applied ? "开启" : "断开"}
            onClick={() => run("use_claude_plus_route")}
          />
          <RouteActionCard
            state="neutral"
            disabled={busy}
            label="Claude Desktop"
            step={4}
            value="重启"
            onClick={restartClaudeDesktop}
          />
        </div>
      </section>

      <section className="panel mappingPanel">
        <div className="panelHead">
          <div>
            <h2>当前服务商与模型映射</h2>
          </div>
        </div>
        <div className="providerStrip">
          <span>CC Switch 当前生效服务商</span>
          <div>
            <strong>{pm?.provider_name ?? "读取失败"}</strong>
            <small>
              {pm ? `Provider ID: ${pm.provider_id}` : mappingError || "请在 Claude++ EXE 中读取 CC Switch 数据库。"}
            </small>
          </div>
        </div>
        <MiniMapping mappings={pm?.mappings ?? []} emptyText={pm ? "无映射" : "未读取到真实模型映射"} />
      </section>
    </div>
  );
}

function LocalizationPage({
  busy,
  zhStatus,
  zhScope,
  setZhScope,
  installClaudeZh,
  backupClaudeZh,
  uninstallClaudeZh,
}: {
  busy: boolean;
  zhStatus: ClaudeZhStatus | null;
  zhScope: LocalizationScope;
  setZhScope: (value: LocalizationScope) => void;
  installClaudeZh: () => Promise<void>;
  backupClaudeZh: () => Promise<void>;
  uninstallClaudeZh: () => Promise<void>;
}) {
  const statusText = zhStatus?.installed
    ? `已安装简体中文汉化，语言文件 ${zhStatus.language_files.join(", ")}`
    : zhStatus?.claude_found
      ? "未安装简体中文汉化"
      : "未检测到 Claude Desktop";
  const installPercent = zhStatus?.installed ? "已汉化" : zhStatus?.claude_found ? "未汉化" : "无法检测";
  const scopeText =
    zhScope === "complete"
      ? "完整汉化：语言文件、前端文案、菜单与 3P 模型校验补丁"
      : "安全汉化：跳过 app.asar 与 Claude.exe 完整性补丁";
  const disabledByMissingClaude = busy || !zhStatus?.supported || !zhStatus?.claude_found;

  return (
    <div className="localizationFlow">
      <div className="actionNotice localizationActionNotice">
        汉化写入后，需点击上方重启Claude Desktop按钮，让新语言资源立即生效。
      </div>
      <section className="panel localizationChecklist">
        <div className="workflowRows">
          <WorkflowRow
            ok={!!zhStatus?.installed}
            title="检测汉化程度"
            description={statusText}
            badge={installPercent}
            tone={zhStatus?.installed ? "success" : zhStatus?.claude_found ? "warning" : "danger"}
          />
          <WorkflowRow
            ok={!!zhStatus?.backup_available}
            title="检测语言文件备份状态"
            description={
              zhStatus?.backup_available
                ? "已发现可恢复备份，可随时恢复英文。"
                : "未发现可恢复备份；建议先备份当前 Claude Desktop 资源。"
            }
            badge={zhStatus?.backup_available ? "可恢复" : undefined}
            tone={zhStatus?.backup_available ? "success" : "warning"}
            action={
              !zhStatus?.backup_available ? (
                <button disabled={disabledByMissingClaude} onClick={backupClaudeZh}>
                  备份
                </button>
              ) : undefined
            }
          />
          <div className="workflowRow">
            <div className="rowIcon success">
              <CheckCircle2 size={16} />
            </div>
            <div className="workflowCopy">
              <strong>选择汉化范围</strong>
              <span>{scopeText}</span>
            </div>
            <div className="scopeSelect" role="group" aria-label="汉化范围">
              <button
                className={zhScope === "complete" ? "active" : ""}
                disabled={busy}
                onClick={() => setZhScope("complete")}
              >
                完整汉化
              </button>
              <button
                className={zhScope === "safe" ? "active" : ""}
                disabled={busy}
                onClick={() => setZhScope("safe")}
              >
                安全汉化
              </button>
            </div>
          </div>
          <WorkflowRow
            ok={!!zhStatus?.claude_found}
            title="一键汉化"
            description="默认安装简体中文，不再提供语言选择。安装时会自动写入必要备份。"
            action={
              <button className="primary" disabled={disabledByMissingClaude} onClick={installClaudeZh}>
                一键汉化
              </button>
            }
          />
          <WorkflowRow
            ok={!!zhStatus?.backup_available}
            title="恢复英文"
            description="从最近一次备份恢复 Claude Desktop 资源，并把语言设回 en-US。"
            action={
              <button disabled={busy || !zhStatus?.backup_available} onClick={uninstallClaudeZh}>
                恢复英文
              </button>
            }
          />
        </div>
      </section>
    </div>
  );
}

function WorkflowRow({
  ok,
  title,
  description,
  badge,
  tone = "success",
  action,
}: {
  ok: boolean;
  title: string;
  description: string;
  badge?: string;
  tone?: "success" | "warning" | "danger";
  action?: ReactNode;
}) {
  const IconComponent = ok ? CheckCircle2 : CircleAlert;
  return (
    <div className="workflowRow">
      <div className={`rowIcon ${tone}`}>
        <IconComponent size={16} />
      </div>
      <div className="workflowCopy">
        <strong>{title}</strong>
        <span>{description}</span>
      </div>
      {action ?? (badge && <span className={`stateBadge ${tone}`}>{badge}</span>)}
    </div>
  );
}

function EnhancePage({
  busy,
  enhanceStatus,
  installClaudeEnhance,
  uninstallClaudeEnhance,
}: {
  busy: boolean;
  enhanceStatus: ClaudeEnhanceStatus | null;
  installClaudeEnhance: (feature: string) => Promise<void>;
  uninstallClaudeEnhance: (feature: string) => Promise<void>;
}) {
  const disabledByMissingClaude = busy || !enhanceStatus?.supported || !enhanceStatus?.claude_found;
  const features = enhanceStatus?.features ?? previewEnhanceFeatures();

  return (
    <div className="enhanceFlow">
      <div className="actionNotice enhanceActionNotice">
        增强脚本开启后，需点击上方重启Claude Desktop按钮，让页面立即生效。
      </div>
      <section className="panel enhanceCardsPanel">
        <div className="enhanceCards">
          {features.map((feature) => (
            <EnhanceCard
              key={feature.id}
              feature={feature}
              disabled={disabledByMissingClaude || !feature.available}
              onInstall={() => installClaudeEnhance(feature.id)}
              onUninstall={() => uninstallClaudeEnhance(feature.id)}
            />
          ))}
        </div>
      </section>
    </div>
  );
}

function EnhanceCard({
  feature,
  disabled,
  onInstall,
  onUninstall,
}: {
  feature: ClaudeEnhanceFeature;
  disabled: boolean;
  onInstall: () => void;
  onUninstall: () => void;
}) {
  const IconComponent = enhanceIcon(feature.id);
  return (
    <div className={`workflowRow enhanceWorkflowRow ${feature.enabled ? "enabled" : ""}`}>
      <div className={`rowIcon ${feature.enabled ? "success" : "warning"}`}>
        <IconComponent size={17} />
      </div>
      <div className="workflowCopy">
        <strong>
          <span className="enhanceCategory">{feature.category}</span>
          {feature.label}
          {feature.id === "conversation_title_i18n" && (
            <span className="enhanceTokenNotice">会消耗少量 token</span>
          )}
        </strong>
        <span>{feature.description}</span>
      </div>
      <div className="enhanceActions">
        <button className="primary" disabled={disabled || feature.enabled} onClick={onInstall}>
          增强
        </button>
        <button disabled={disabled || !feature.enabled} onClick={onUninstall}>
          取消
        </button>
      </div>
    </div>
  );
}

function previewEnhanceFeatures(): ClaudeEnhanceFeature[] {
  return [
    {
      id: "third_party_api",
      category: "菜单栏增强",
      label: "第三方API",
      description: "在 Claude Desktop 左侧菜单栏，增加第三方API快捷入口。",
      enabled: false,
      available: true,
      note: "侧边栏软入口",
    },
    {
      id: "plugins",
      category: "菜单栏增强",
      label: "技能",
      description: "在 Claude Desktop 左侧菜单栏，增加全局/项目 skills 快捷入口。",
      enabled: false,
      available: true,
      note: "侧边栏软入口",
    },
    {
      id: "mcp",
      category: "菜单栏增强",
      label: "MCP",
      description: "在 Claude Desktop 左侧菜单栏，增加 MCP 快捷入口。",
      enabled: false,
      available: true,
      note: "侧边栏软入口",
    },
    {
      id: "conversation_title_i18n",
      category: "对话增强",
      label: "对话列表中文化",
      description: "把 Claude Desktop 对话列表里的英文标题自动翻译为中文显示。",
      enabled: false,
      available: true,
      note: "本地代理翻译",
    },
    {
      id: "markdown",
      category: "对话增强",
      label: "对话导出Markdown",
      description: "在对话列表“...”内，增加导出 Markdown 文件按钮。",
      enabled: false,
      available: true,
      note: "导出已加载内容",
    },
    {
      id: "timeline",
      category: "对话增强",
      label: "对话时间线",
      description: "在对话区域右侧，增加用户发言时间线，方便快速定位上下文。",
      enabled: false,
      available: true,
      note: "已加载问题定位",
    },
    {
      id: "token_usage",
      category: "对话增强",
      label: "Token使用信息",
      description: "在对话页面显示本次响应的 Token、缓存、上下文占用与耗时信息。",
      enabled: false,
      available: true,
      note: "响应用量统计",
    },
  ];
}

function enhanceIcon(id: string): Icon {
  if (id === "third_party_api") return Code2;
  if (id === "plugins") return PackageCheck;
  if (id === "mcp") return Network;
  if (id === "conversation_title_i18n") return ListRestart;
  if (id === "markdown") return FileText;
  if (id === "timeline") return Activity;
  if (id === "token_usage") return Gauge;
  return Plug;
}

function AboutPage({ claudeDesktopVersion }: { claudeDesktopVersion: string }) {
  return (
    <div className="pageGrid aboutPage">
      <section className="panel aboutPanel">
        <div className="panelHead">
          <div>
            <h2>Claude++</h2>
            <p>本地 Claude Desktop 增强，并优化与 CC Switch 桥接的工具。</p>
            <p>包含 CCS 转接、一键安装、一键汉化、页面增强、诊断日志等能力</p>
          </div>
          <Info size={20} />
        </div>

        <div className="aboutInfoTable">
          <AboutInfoRow label="Claude++ 版本" value="1.0.0" />
          <AboutInfoRow label="Claude Desktop 版本" value={claudeDesktopVersion} />
          <AboutInfoRow
            label="仓库地址"
            value="待补充"
            action={
              <button disabled title="仓库地址待补充">
                前往仓库
              </button>
            }
          />
          <div className="releaseCard">
            <div className="releaseCardHead">
              <strong>GitHub Release 更新</strong>
              <span>当前版本 1.0.0</span>
            </div>
            <AboutInfoRow label="状态" value="待补充" />
            <AboutInfoRow label="最新版本" value="待补充" />
            <AboutInfoRow label="资源" value="待补充" />
            <AboutInfoRow label="进度" value="0%" />
            <textarea className="releaseNotes" readOnly value="Release 信息待补充。" />
            <div className="releaseActions">
              <button disabled>检查更新</button>
              <button disabled>下载并运行安装包</button>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}

function DiagnosticsPage({
  diagnostics,
  logs,
  refreshDiagnostics,
  refreshLogs,
  copyText,
}: {
  diagnostics: DiagnosticsPayload | null;
  logs: LogsPayload | null;
  refreshDiagnostics: () => Promise<void>;
  refreshLogs: () => Promise<void>;
  copyText: (text: string) => Promise<void>;
}) {
  const logLines = splitLogLines(logs?.text ?? "");

  return (
    <div className="pageGrid diagnosticsPage">
      <section className="panel diagnosticsReportPanel">
        <div className="panelHead">
          <div>
            <h2>诊断报告</h2>
            <p>包含路由、模型映射、汉化、增强和本地路径信息。</p>
          </div>
          <Activity size={20} />
        </div>
        <textarea
          className="diagnosticsReport"
          readOnly
          spellCheck={false}
          value={diagnostics?.report ?? "尚未生成诊断报告。"}
        />
        <div className="diagnosticsToolbar">
          <button onClick={() => void refreshDiagnostics()}>重新生成</button>
          <button
            disabled={!diagnostics?.report}
            onClick={() => void copyText(diagnostics?.report ?? "")}
          >
            复制报告
          </button>
        </div>
      </section>

      <section className="panel diagnosticsLogPanel">
        <div className="panelHead">
          <div>
            <h2>最近日志</h2>
            <p>{logs?.path ?? "读取本地 Claude++ 诊断日志。"}</p>
          </div>
          <FileText size={20} />
        </div>
        <div className="logLines">
          {logLines.length ? (
            logLines.map((line, index) => (
              <div className="logLine" key={`${index}-${line.slice(0, 20)}`}>
                <span>{index + 1}</span>
                <code>{line || " "}</code>
              </div>
            ))
          ) : (
            <div className="emptyInline">暂无日志。</div>
          )}
        </div>
        <div className="diagnosticsToolbar">
          <button onClick={() => void refreshLogs()}>刷新</button>
          <button disabled={!logs?.text} onClick={() => void copyText(logs?.text ?? "")}>
            复制
          </button>
        </div>
      </section>
    </div>
  );
}

function splitLogLines(text: string) {
  return text.split(/\r?\n/).filter((line) => line.length > 0);
}

function RouteStatusCard({
  active,
  label,
  step,
  value,
}: {
  active: boolean;
  label: string;
  step: number;
  value: string;
}) {
  return (
    <div className={`routeStatusCard ${active ? "active" : "inactive"}`}>
      <span className="stepBadge">{step}</span>
      <span className={`dot ${active ? "on" : "off"}`} />
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
      </div>
    </div>
  );
}

function RouteActionCard({
  state,
  disabled,
  label,
  step,
  value,
  onClick,
}: {
  state: "on" | "off" | "neutral";
  disabled: boolean;
  label: string;
  step: number;
  value?: string;
  onClick: () => void;
}) {
  return (
    <button className={`routeActionCard ${state}`} disabled={disabled} onClick={onClick}>
      <span className="stepBadge">{step}</span>
      <span>{label}</span>
      {value && <strong>{value}</strong>}
    </button>
  );
}

function MiniMapping({ mappings, emptyText }: { mappings: Mapping[]; emptyText: string }) {
  return (
    <div className="miniTable">
      {!!mappings.length && (
        <div className="miniTableHead">
          <span>CCS模型角色</span>
          <span>Claude模型显示名</span>
          <span>实际请求模型</span>
        </div>
      )}
      {mappings.map((m) => (
        <div key={m.role}>
          <strong>{ccsRoleLabel(m.role_kind)}</strong>
          <span>{m.display}</span>
          <code>{m.model}</code>
        </div>
      ))}
      {!mappings.length && <div className="emptyInline">{emptyText}</div>}
    </div>
  );
}

function ccsRoleLabel(roleKind: string) {
  const normalized = roleKind.toLowerCase();
  if (normalized === "opus") return "Opus";
  if (normalized === "sonnet") return "Sonnet";
  if (normalized === "haiku") return "Haiku";
  return roleKind || "未知";
}

function AboutInfoRow({ label, value, action }: { label: string; value: string; action?: ReactNode }) {
  return (
    <div className="aboutInfoRow">
      <span>{label}</span>
      <strong>{value}</strong>
      {action && <div className="aboutInfoAction">{action}</div>}
    </div>
  );
}

export default App;
