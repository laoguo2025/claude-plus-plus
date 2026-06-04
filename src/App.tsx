import { useEffect, useState, useCallback, useRef, type ComponentType, type ReactNode } from "react";
import {
  Activity,
  CheckCircle2,
  CircleAlert,
  Code2,
  Gauge,
  ListRestart,
  FileText,
  Minus,
  Network,
  PackageCheck,
  Plug,
  Moon,
  Power,
  RefreshCw,
  Square,
  Sun,
  X,
  type LucideProps,
} from "lucide-react";
import botLogo from "../src-tauri/icons/icon.png";
import type {
  ClaudeEnhanceFeature,
  ClaudeEnhanceStatus,
  ClaudeZhStatus,
  DiagnosticsPayload,
  LocalizationScope,
  LogsPayload,
  Mapping,
  ProviderMappings,
  Route,
  StatusInfo,
  Theme,
  WelcomeStatus,
} from "./appTypes";
import {
  ALIPAY_QR_PATH,
  CC_SWITCH_DOWNLOAD_URL,
  CLAUDE_DESKTOP_DOWNLOAD_URL,
  GITHUB_RELEASES_URL,
  GITHUB_REPOSITORY_URL,
  QQ_GROUP_QR_PATH,
  routeMeta,
  routes,
} from "./appConstants";
import { previewEnhanceFeatures, PREVIEW_APP_VERSION } from "./previewCommands";
import { routeSummaryText, ccswitchRouteDetailText } from "./routeStatus";
import { callCommand, openExternalUrl } from "./tauriClient";
import "./App.css";

type Icon = ComponentType<LucideProps>;
type EnhanceSection = "quick_access" | "enhance";

const QUICK_ACCESS_FEATURE_IDS = new Set(["third_party_api", "plugins", "mcp"]);

function loadInitialTheme(): Theme {
  return window.localStorage.getItem("claude-plus-theme") === "dark" ? "dark" : "light";
}

function shouldPollRouteState(welcomeStatus: WelcomeStatus | null): boolean {
  return welcomeStatus?.cc_switch_installed === true;
}

async function runWindowAction(action: "minimize" | "toggleMaximize" | "close"): Promise<void> {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const currentWindow = getCurrentWindow();
  if (action === "minimize") {
    await currentWindow.minimize();
    return;
  }
  if (action === "toggleMaximize") {
    await currentWindow.toggleMaximize();
    return;
  }
  await currentWindow.close();
}

function App() {
  const [route, setRoute] = useState<Route>("welcome");
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
  const [welcomeStatus, setWelcomeStatus] = useState<WelcomeStatus | null>(null);
  const [appVersion, setAppVersion] = useState(PREVIEW_APP_VERSION);
  const lastMappingFingerprint = useRef<string | null>(null);
  const routeStatePollingEnabled = shouldPollRouteState(welcomeStatus);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem("claude-plus-theme", theme);
  }, [theme]);

  const refreshAppVersion = useCallback(async () => {
    try {
      setAppVersion(await callCommand<string>("app_version"));
    } catch (e) {
      setErr(String(e));
    }
  }, []);

  const refreshRouteState = useCallback(async () => {
    await refreshAppVersion();
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
  }, [refreshAppVersion]);

  const detectClaudeDesktopOnce = useCallback(async () => {
    try {
      setZhStatus(await callCommand<ClaudeZhStatus>("claude_zh_status"));
    } catch (e) {
      setErr(String(e));
      setZhStatus(null);
    }
  }, []);

  const refreshWelcomeStatus = useCallback(async () => {
    try {
      setWelcomeStatus(await callCommand<WelcomeStatus>("welcome_status"));
    } catch (e) {
      setErr(String(e));
      setWelcomeStatus(null);
    }
  }, []);

  const refreshEnhanceStatus = useCallback(async () => {
    try {
      setEnhanceStatus(await callCommand<ClaudeEnhanceStatus>("claude_enhance_status"));
    } catch (e) {
      setErr(String(e));
      setEnhanceStatus(null);
    }
  }, []);

  useEffect(() => {
    refreshWelcomeStatus();
    refreshAppVersion();
  }, [refreshAppVersion, refreshWelcomeStatus]);

  useEffect(() => {
    if (route === "overview") {
      refreshRouteState();
    }
    if (route === "localization" || route === "about") {
      detectClaudeDesktopOnce();
    }
    if (route === "quick_access" || route === "enhance") {
      refreshEnhanceStatus();
    }
  }, [detectClaudeDesktopOnce, refreshEnhanceStatus, refreshRouteState, route]);

  useEffect(() => {
    if (route !== "overview") return;
    if (!routeStatePollingEnabled) return;
    const t = setInterval(refreshRouteState, 4000);
    return () => clearInterval(t);
  }, [refreshRouteState, route, routeStatePollingEnabled]);

  const runBusy = useCallback(async (action: () => Promise<void>) => {
    setBusy(true);
    setErr("");
    try {
      await action();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }, []);

  const run = (cmd: string) =>
    runBusy(async () => {
      await callCommand(cmd);
      await refreshRouteState();
    });

  const restartClaudeDesktop = () =>
    runBusy(async () => {
      await callCommand("restart_claude_desktop");
      setRestartNeeded(false);
      await refreshRouteState();
    });

  const enableClaudeDeveloperMode = () =>
    runBusy(async () => {
      await callCommand("enable_claude_developer_mode");
      await refreshWelcomeStatus();
    });

  const installClaudeCode = () =>
    runBusy(async () => {
      await callCommand("install_claude_code");
      await refreshWelcomeStatus();
    });

  const toggleTheme = () => {
    setTheme((current) => (current === "dark" ? "light" : "dark"));
  };

  const installClaudeZh = () =>
    runBusy(async () => {
      await callCommand("install_claude_zh", {
        language: "zh-CN",
        skipAsarPatch: zhScope === "safe",
      });
      await detectClaudeDesktopOnce();
      await refreshRouteState();
    });

  const backupClaudeZh = () =>
    runBusy(async () => {
      await callCommand("backup_claude_zh");
      await detectClaudeDesktopOnce();
    });

  const uninstallClaudeZh = () =>
    runBusy(async () => {
      await callCommand("uninstall_claude_zh");
      await detectClaudeDesktopOnce();
    });

  const installClaudeEnhance = (feature: string) =>
    runBusy(async () => {
      await callCommand("install_claude_enhance", { feature });
      await refreshEnhanceStatus();
    });

  const uninstallClaudeEnhance = (feature: string) =>
    runBusy(async () => {
      await callCommand("uninstall_claude_enhance", { feature });
      await refreshEnhanceStatus();
    });

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

  const refreshAll = useCallback(async () => {
    setBusy(true);
    setErr("");
    try {
      const refreshSteps = [refreshAppVersion, refreshWelcomeStatus];
      if (route === "localization" || route === "about") {
        refreshSteps.push(detectClaudeDesktopOnce);
      }
      if (route === "overview") {
        refreshSteps.push(refreshRouteState);
      }
      if (route === "quick_access" || route === "enhance") {
        refreshSteps.push(refreshEnhanceStatus);
      }
      if (route === "diagnostics") {
        refreshSteps.push(refreshLogs, refreshDiagnostics);
      }
      const errors: string[] = [];
      for (const refreshStep of refreshSteps) {
        try {
          await refreshStep();
        } catch (e) {
          errors.push(String(e));
        }
      }
      if (errors.length > 0) {
        setErr(errors.join("\n"));
      }
    } finally {
      setBusy(false);
    }
  }, [
    detectClaudeDesktopOnce,
    refreshAppVersion,
    refreshDiagnostics,
    refreshEnhanceStatus,
    refreshLogs,
    refreshRouteState,
    refreshWelcomeStatus,
    route,
  ]);

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
        <header className="topbar" data-tauri-drag-region>
          <div className="topbarTitle" data-tauri-drag-region>
            <h1>{meta.title}</h1>
          </div>
          <div className="topbarActions" aria-label="页面操作">
            <button
              className="iconButton restartButton"
              disabled={busy}
              onClick={restartClaudeDesktop}
              title="重启 Claude Desktop"
            >
              <Power size={16} />
              <span>重启 Claude Desktop</span>
            </button>
            <button className="iconButton square" disabled={busy} onClick={refreshAll} title="全局刷新" aria-label="全局刷新">
              <RefreshCw size={16} />
            </button>
            <button
              className="iconButton square"
              onClick={toggleTheme}
              title={theme === "dark" ? "切换亮色主题" : "切换暗色主题"}
              aria-label={theme === "dark" ? "切换亮色主题" : "切换暗色主题"}
            >
              {theme === "dark" ? <Sun size={16} /> : <Moon size={16} />}
            </button>
            <button
              className="iconButton square windowButton"
              onClick={() => void runWindowAction("minimize")}
              title="最小化"
              aria-label="最小化"
            >
              <Minus size={16} />
            </button>
            <button
              className="iconButton square windowButton"
              onClick={() => void runWindowAction("toggleMaximize")}
              title="最大化"
              aria-label="最大化"
            >
              <Square size={14} />
            </button>
            <button
              className="iconButton square windowButton closeWindowButton"
              onClick={() => void runWindowAction("close")}
              title="关闭"
              aria-label="关闭"
            >
              <X size={16} />
            </button>
          </div>
        </header>

        <section className="screen">
          {err && <div className="errorBanner">{err}</div>}

          {route === "welcome" && (
            <WelcomePage
              busy={busy}
              welcomeStatus={welcomeStatus}
              setErr={setErr}
              enableClaudeDeveloperMode={enableClaudeDeveloperMode}
              installClaudeCode={installClaudeCode}
            />
          )}

          {route === "overview" && (
            <OverviewPage
              busy={busy}
              status={status}
              pm={pm}
              mappingError={mappingError}
              restartNeeded={restartNeeded}
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

          {route === "quick_access" && (
            <EnhancePage
              busy={busy}
              section="quick_access"
              enhanceStatus={enhanceStatus}
              installClaudeEnhance={installClaudeEnhance}
              uninstallClaudeEnhance={uninstallClaudeEnhance}
            />
          )}

          {route === "enhance" && (
            <EnhancePage
              busy={busy}
              section="enhance"
              enhanceStatus={enhanceStatus}
              installClaudeEnhance={installClaudeEnhance}
              uninstallClaudeEnhance={uninstallClaudeEnhance}
            />
          )}

          {route === "about" && (
            <AboutPage
              appVersion={appVersion}
              claudeDesktopVersion={zhStatus?.claude_version ?? (zhStatus?.claude_found ? "版本未知" : "未定位到资源目录")}
              setErr={setErr}
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
  restartNeeded,
  run,
  restartClaudeDesktop,
}: {
  busy: boolean;
  status: StatusInfo | null;
  pm: ProviderMappings | null;
  mappingError: string;
  restartNeeded: boolean;
  run: (cmd: string) => Promise<void>;
  restartClaudeDesktop: () => Promise<void>;
}) {
  const ccswitchRoute = status?.ccswitch_route;
  const routeSummary = routeSummaryText(status);
  const providerConfigured = !!pm;
  const ccswitchSwitchOn = ccswitchRoute?.configured === true;
  const ccswitchSwitchDetail = ccswitchRouteDetailText(ccswitchRoute);
  const claudePlusTakenOver = !!status?.cd_applied && !!status?.running;

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
        <span>Claude Desktop 菜单会按 CC Switch 的“菜单显示名”原样展示；同名模型由用户自己的命名决定。</span>
        <strong>使用期间请保持 Claude++ 运行；CC Switch 增/改/删模型或切换服务商后，重启 Claude Desktop 生效。</strong>
      </div>

      <section className="panel routePanel">
        <div className="panelHead routePanelHead">
          <div>
            <h2>路由转接状态</h2>
          </div>
          <span className="routeHint">{routeSummary}</span>
        </div>
        <div className="routeCardBody">
          <RouteStatusCard
            active={ccswitchSwitchOn}
            label="CC Switch 路由开关"
            value={ccswitchSwitchOn ? "已开启" : "未开启"}
            detail={ccswitchSwitchDetail}
          />
          <RouteStatusCard
            active={claudePlusTakenOver}
            label="Claude++ 接管"
            value={claudePlusTakenOver ? "已接管" : "未接管"}
            detail={claudePlusTakenOver ? undefined : "点击接管,让 Claude++ 生效"}
            action={{
              label: status?.cd_applied ? "断开接管" : "接管",
              onClick: () => run(status?.cd_applied ? "use_ccs_route" : "use_claude_plus_route"),
              disabled: busy,
              primary: !claudePlusTakenOver,
            }}
          />
          <RouteStatusCard
            active={providerConfigured}
            label="模型服务商配置"
            value={providerConfigured ? "已配置" : "未配置"}
            detail={providerConfigured ? undefined : mappingError || "请在 CC Switch 中配置模型服务商"}
          />
        </div>
        {restartNeeded && (
          <div className="routeRestartNotice">
            <span>模型映射或路由配置已变化,需要重启 Claude Desktop 后菜单才会刷新。</span>
            <button disabled={busy} onClick={restartClaudeDesktop}>
              重启 Claude Desktop
            </button>
          </div>
        )}
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
      : "未定位到 Claude Desktop 资源目录，可重新安装或在 Claude++ 设置中指定资源路径";
  const installPercent = zhStatus?.installed ? "已汉化" : zhStatus?.claude_found ? "未汉化" : "未定位";
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
            ok={!zhStatus?.installed}
            title="恢复英文"
            description="只移除中文语言资源并把语言设回 en-US，不覆盖页面增强脚本。"
            action={
              <button disabled={busy || !zhStatus?.installed} onClick={uninstallClaudeZh}>
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
  section,
  enhanceStatus,
  installClaudeEnhance,
  uninstallClaudeEnhance,
}: {
  busy: boolean;
  section: EnhanceSection;
  enhanceStatus: ClaudeEnhanceStatus | null;
  installClaudeEnhance: (feature: string) => Promise<void>;
  uninstallClaudeEnhance: (feature: string) => Promise<void>;
}) {
  const disabledByMissingClaude = busy || !enhanceStatus?.supported || !enhanceStatus?.claude_found;
  const features = (enhanceStatus?.features ?? previewEnhanceFeatures()).filter((feature) =>
    section === "quick_access"
      ? QUICK_ACCESS_FEATURE_IDS.has(feature.id)
      : !QUICK_ACCESS_FEATURE_IDS.has(feature.id),
  );

  return (
    <div className="enhanceFlow">
      <div className="actionNotice enhanceActionNotice">
        {enhanceStatus?.claude_found === false
          ? "未定位到 Claude Desktop 资源目录；请重新安装 Claude Desktop，或在 Claude++ 设置中指定资源路径后刷新。"
          : "增强脚本开启后，需点击上方重启Claude Desktop按钮，让页面立即生效。"}
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

function WelcomePage({
  busy,
  welcomeStatus,
  setErr,
  enableClaudeDeveloperMode,
  installClaudeCode,
}: {
  busy: boolean;
  welcomeStatus: WelcomeStatus | null;
  setErr: (error: string) => void;
  enableClaudeDeveloperMode: () => Promise<void>;
  installClaudeCode: () => Promise<void>;
}) {
  const loading = welcomeStatus === null;
  const downloadClaudeDesktop = async () => {
    setErr("");
    try {
      await openExternalUrl(CLAUDE_DESKTOP_DOWNLOAD_URL);
    } catch (e) {
      setErr(String(e));
    }
  };

  const downloadCcSwitch = async () => {
    setErr("");
    try {
      await openExternalUrl(CC_SWITCH_DOWNLOAD_URL);
    } catch (e) {
      setErr(String(e));
    }
  };

  return (
    <div className="welcomePage">
      <section className="welcomeHero">
        <div className="welcomeIntro">
          <img className="welcomeLogo" src={botLogo} alt="Claude++" />
          <div className="welcomeCopy">
            <h2>Claude++</h2>
            <p>Claude++，是一款 Claude Desktop 的本地增强工具，</p>
            <p>提供 CCS 转接优化、一键汉化、第三方API接入、对话增强等能力。</p>
          </div>
        </div>
        <div className="welcomeQrGroup">
          <QrCard
            kind="qq"
            src={QQ_GROUP_QR_PATH}
            alt="QQ交流群二维码"
            text="QQ群：582589880，欢迎交流反馈，提出建议。"
          />
          <QrCard
            kind="alipay"
            src={ALIPAY_QR_PATH}
            alt="个人支付宝收款码"
            text="如果 Claude++ 帮到了你，可用支付宝支持一下。"
          />
        </div>
      </section>

      <p className="welcomeActionHint">
        如果下方几项显示未安装/未开启，可直接点击卡片进行下载/开启。
        <br />
        下载会跳转百度网盘连接，无需魔法登录github。
      </p>

      <section className="welcomeStatusGrid" aria-label="环境状态检测">
        <RouteStatusCard
          loading={loading}
          active={!!welcomeStatus?.claude_code_installed}
          label="Claude Code"
          value={loading ? "检测中" : welcomeStatus?.claude_code_installed ? "已安装" : "未安装"}
          detail={loading ? undefined : welcomeStatus?.claude_code_installed ? undefined : "点击后一键命令行安装"}
          action={
            loading || welcomeStatus?.claude_code_installed
              ? undefined
              : {
                  label: "一键安装",
                  onClick: () => void installClaudeCode(),
                  disabled: busy,
                  primary: true,
                }
          }
        />
        <RouteStatusCard
          loading={loading}
          active={!!welcomeStatus?.claude_desktop_found}
          label="Claude Desktop"
          value={loading ? "检测中" : welcomeStatus?.claude_desktop_found ? "已定位" : "未定位"}
          detail={loading ? undefined : welcomeStatus?.claude_desktop_found ? undefined : "点击后从网盘下载"}
          action={
            loading || welcomeStatus?.claude_desktop_found
              ? undefined
              : {
                  label: "下载",
                  onClick: () => void downloadClaudeDesktop(),
                  disabled: busy,
                  primary: true,
                }
          }
        />
        <RouteStatusCard
          loading={loading}
          active={!!welcomeStatus?.developer_mode_enabled}
          label="开发者模式"
          value={loading ? "检测中" : welcomeStatus?.developer_mode_enabled ? "已开启" : "未开启"}
          detail={loading ? undefined : welcomeStatus?.developer_mode_enabled ? undefined : "点击后一键开启"}
          action={
            loading || welcomeStatus?.developer_mode_enabled
              ? undefined
              : {
                  label: "一键开启",
                  onClick: () => void enableClaudeDeveloperMode(),
                  disabled: busy,
                  primary: true,
                }
          }
        />
        <RouteStatusCard
          loading={loading}
          active={!!welcomeStatus?.cc_switch_installed}
          label="CC Switch"
          value={loading ? "检测中" : welcomeStatus?.cc_switch_installed ? "已安装" : "未安装"}
          detail={loading ? undefined : welcomeStatus?.cc_switch_installed ? undefined : "点击后从网盘下载"}
          action={
            loading || welcomeStatus?.cc_switch_installed
              ? undefined
              : {
                  label: "下载",
                  onClick: () => void downloadCcSwitch(),
                  disabled: busy,
                  primary: true,
                }
          }
        />
      </section>
    </div>
  );
}

function QrCard({
  kind,
  src,
  alt,
  text,
}: {
  kind: "qq" | "alipay";
  src: string;
  alt: string;
  text: string;
}) {
  return (
    <div className={`qrCard ${kind}`}>
      <img src={src} alt={alt} />
      <p>{text}</p>
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
          {feature.label}
          <span className="enhanceVersion">{feature.version}</span>
          {feature.id === "conversation_title_i18n" && (
            <span className="enhanceTokenNotice">会消耗少量 token</span>
          )}
        </strong>
        <span>{feature.description}</span>
      </div>
      <div className="enhanceActions">
        <button className="primary" disabled={disabled || feature.enabled} onClick={onInstall}>
          安装
        </button>
        <button disabled={disabled || !feature.enabled} onClick={onUninstall}>
          卸载
        </button>
      </div>
    </div>
  );
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

function AboutPage({
  appVersion,
  claudeDesktopVersion,
  setErr,
}: {
  appVersion: string;
  claudeDesktopVersion: string;
  setErr: (error: string) => void;
}) {
  const openRepository = async () => {
    setErr("");
    try {
      await openExternalUrl(GITHUB_REPOSITORY_URL);
    } catch (e) {
      setErr(String(e));
    }
  };

  const openReleases = async () => {
    setErr("");
    try {
      await openExternalUrl(GITHUB_RELEASES_URL);
    } catch (e) {
      setErr(String(e));
    }
  };

  return (
    <div className="pageGrid aboutPage">
      <section className="panel aboutPanel">
        <div className="aboutInfoTable">
          <AboutInfoRow label="Claude++ 版本" value={appVersion} />
          <AboutInfoRow label="Claude Desktop 版本" value={claudeDesktopVersion} />
          <AboutInfoRow
            label="仓库地址"
            value={GITHUB_REPOSITORY_URL}
            action={<button onClick={() => void openRepository()}>打开仓库</button>}
          />
          <div className="releaseCard">
            <div className="releaseCardHead">
              <strong>GitHub Release 更新</strong>
              <span>当前版本 {appVersion}</span>
            </div>
            <AboutInfoRow label="状态" value="未接入自动检查" />
            <AboutInfoRow
              label="最新版本"
              value="请以 GitHub Release 为准"
              action={<button onClick={() => void openReleases()}>打开 Release</button>}
            />
            <AboutInfoRow label="资源" value="GitHub Release 与本地构建脚本" />
            <textarea
              className="releaseNotes"
              readOnly
              value={`当前应用版本: ${appVersion}\n\nClaude++ 目前未接入自动更新。需要确认新版时，请打开 GitHub Release，或使用本地 release 构建脚本。`}
            />
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
            <p>包含自动判定、路由、模型映射、汉化、增强和本地路径信息。</p>
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
  loading = false,
  label,
  value,
  detail,
  action,
}: {
  active: boolean;
  loading?: boolean;
  label: string;
  value: string;
  detail?: string;
  action?: {
    label: string;
    onClick: () => void;
    disabled: boolean;
    primary?: boolean;
  };
}) {
  return (
    <div className={`routeStatusCard ${loading ? "loading" : active ? "active" : "inactive"}`}>
      <span className={`dot ${loading ? "loading" : active ? "on" : "off"}`} />
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
        {detail && <small>{detail}</small>}
        {action && (
          <button
            className={action.primary ? "primary routeCardButton" : "routeCardButton"}
            disabled={action.disabled}
            onClick={action.onClick}
          >
            {action.label}
          </button>
        )}
      </div>
    </div>
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
