import {
  useEffect,
  useState,
  useCallback,
  useRef,
} from "react";
import { Minus, Moon, Power, RefreshCw, Square, Sun, X } from "lucide-react";
import botLogo from "../src-tauri/icons/icon.png";
import type {
  ClaudeEnhanceStatus,
  ClaudeZhStatus,
  DiagnosticsPayload,
  LocalizationScope,
  LogsPayload,
  ProviderMappings,
  Route,
  StatusInfo,
  Theme,
  WelcomeStatus,
} from "./appTypes";
import { routeMeta, routes } from "./appConstants";
import { PREVIEW_APP_VERSION } from "./previewCommands";
import { callCommand } from "./tauriClient";
import { AboutPage } from "./pages/AboutPage";
import { DiagnosticsPage } from "./pages/DiagnosticsPage";
import { EnhancePage } from "./pages/EnhancePage";
import { LocalizationPage } from "./pages/LocalizationPage";
import { OverviewPage } from "./pages/OverviewPage";
import { WelcomePage } from "./pages/WelcomePage";
import "./App.css";

const ROUTE_STATE_POLL_INTERVAL_MS = 4_000;
const LOG_REFRESH_POLL_INTERVAL_MS = 10_000;

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
  const [diagnosticsBusy, setDiagnosticsBusy] = useState(false);
  const [welcomeStatus, setWelcomeStatus] = useState<WelcomeStatus | null>(null);
  const [appVersion, setAppVersion] = useState(PREVIEW_APP_VERSION);
  const lastMappingFingerprint = useRef<string | null>(null);
  const virtualMachinePlatformEnableRequested = useRef(false);
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
    const t = setInterval(refreshRouteState, ROUTE_STATE_POLL_INTERVAL_MS);
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

  const enableVirtualMachinePlatform = useCallback(async () => {
    await callCommand("enable_virtual_machine_platform");
    await refreshWelcomeStatus();
  }, [refreshWelcomeStatus]);

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
    if (diagnosticsBusy) return;
    setDiagnosticsBusy(true);
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
    } finally {
      setDiagnosticsBusy(false);
    }
  }, [diagnosticsBusy, restartNeeded]);

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
        refreshSteps.push(refreshLogs);
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
    refreshEnhanceStatus,
    refreshLogs,
    refreshRouteState,
    refreshWelcomeStatus,
    route,
  ]);

  useEffect(() => {
    if (route !== "diagnostics") return;
    refreshLogs();
  }, [route, refreshLogs]);

  useEffect(() => {
    if (route !== "diagnostics") return;
    const t = setInterval(refreshLogs, LOG_REFRESH_POLL_INTERVAL_MS);
    return () => clearInterval(t);
  }, [route, refreshLogs]);

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
              enableVirtualMachinePlatform={enableVirtualMachinePlatform}
              virtualMachinePlatformEnableRequested={virtualMachinePlatformEnableRequested}
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
              welcomeStatus={welcomeStatus}
              zhScope={zhScope}
              setZhScope={setZhScope}
              enableClaudeDeveloperMode={enableClaudeDeveloperMode}
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
              diagnosticsBusy={diagnosticsBusy}
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

export default App;
