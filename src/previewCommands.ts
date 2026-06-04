import enhanceFeatureDefinitions from "./shared/enhance-features.json";
import type {
  ClaudeEnhanceFeature,
  ClaudeEnhanceStatus,
  ClaudeZhStatus,
  DiagnosticsPayload,
  LogsPayload,
  StatusInfo,
  WelcomeStatus,
} from "./appTypes";

export const PREVIEW_APP_VERSION = __APP_VERSION__;

const previewStatus: StatusInfo = {
  running: true,
  port: __DEFAULT_PROXY_PORT__,
  cd_applied: true,
  ccswitch_route: {
    claude_route_enabled: true,
    proxy_enabled: true,
    enabled: true,
    configured: true,
    has_mappings: true,
    reachable: true,
  },
};

const previewZhStatus: ClaudeZhStatus = {
  supported: true,
  claude_found: true,
  installed: false,
  backup_available: true,
  claude_version: "未检测到",
  install_path: null,
  resources_path: null,
  locale: "en-US",
  language_files: [],
  resource_metadata: {
    language: "zh-CN",
    source_repository: "javaht/claude-desktop-zh-cn",
    source_commit: "8505555ef344df5a26a0a17c9d6fac2a7c235d93",
    source_release: "1.2.0",
    synchronized_at: "2026-06-05",
    resource_scope: ["frontend", "frontend-hardcoded", "desktop", "statsig"],
    merge_policy:
      "Import upstream zh-CN entries only when Claude++ is missing the key or still falls back to English; keep Claude++ visible overrides and existing hardcoded translations authoritative.",
  },
};

const previewWelcomeStatus: WelcomeStatus = {
  claude_code_installed: false,
  claude_desktop_found: true,
  developer_mode_enabled: true,
  cc_switch_installed: true,
};

export function previewEnhanceFeatures(): ClaudeEnhanceFeature[] {
  return enhanceFeatureDefinitions.map((feature) => ({
    ...feature,
    enabled: false,
  }));
}

export function previewCommand<T>(cmd: string): T {
  if (cmd === "app_version") {
    return PREVIEW_APP_VERSION as T;
  }
  if (cmd === "proxy_status") {
    return clone(previewStatus) as T;
  }
  if (cmd === "get_mappings") {
    throw new Error(
      "浏览器预览无法读取 CC Switch 数据库；请在 Claude++ EXE 中查看真实服务商和模型映射。",
    );
  }
  if (cmd === "claude_zh_status") {
    return clone(previewZhStatus) as T;
  }
  if (cmd === "welcome_status") {
    return clone(previewWelcomeStatus) as T;
  }
  if (cmd === "claude_enhance_status") {
    return previewEnhanceStatus() as T;
  }
  if (isPreviewNoopCommand(cmd)) {
    applyPreviewCommandState(cmd);
    return undefined as T;
  }
  if (cmd === "read_latest_logs") {
    return previewLogs() as T;
  }
  if (cmd === "generate_diagnostics") {
    return {
      report: previewDiagnosticsReport(),
    } satisfies DiagnosticsPayload as T;
  }
  throw new Error(`浏览器预览不支持命令: ${cmd}`);
}

function previewEnhanceStatus(): ClaudeEnhanceStatus {
  return {
    supported: previewZhStatus.supported,
    claude_found: previewZhStatus.claude_found,
    installed: false,
    backup_available: true,
    install_path: null,
    resources_path: null,
    features: previewEnhanceFeatures(),
  };
}

function isPreviewNoopCommand(cmd: string): boolean {
  return [
    "use_claude_plus_route",
    "use_ccs_route",
    "restart_claude_desktop",
    "backup_claude_zh",
    "install_claude_zh",
    "uninstall_claude_zh",
    "enable_claude_developer_mode",
    "install_claude_code",
    "install_claude_enhance",
    "uninstall_claude_enhance",
  ].includes(cmd);
}

function applyPreviewCommandState(cmd: string) {
  if (cmd === "enable_claude_developer_mode") {
    previewWelcomeStatus.developer_mode_enabled = true;
  }
  if (cmd === "install_claude_code") {
    previewWelcomeStatus.claude_code_installed = true;
  }
  if (cmd === "install_claude_zh") {
    previewZhStatus.installed = true;
    previewZhStatus.locale = "zh-CN";
  }
  if (cmd === "uninstall_claude_zh") {
    previewZhStatus.installed = false;
    previewZhStatus.locale = "en-US";
  }
}

function previewLogs(): LogsPayload {
  return {
    path: "%USERPROFILE%\\.claude-plus-plus\\claude-plus-plus.log",
    text: [
      JSON.stringify({
        timestamp_ms: Date.now(),
        pid: 18896,
        event: "manager.proxy_status",
        detail: previewStatus,
      }),
      '{"timestamp_ms":1780394329499,"pid":18896,"event":"manager.generate_diagnostics","detail":{}}',
    ].join("\n"),
    lines: 200,
  };
}

function previewDiagnosticsReport(): string {
  return JSON.stringify(
    {
      schemaVersion: 2,
      generatedAtMs: Date.now(),
      version: PREVIEW_APP_VERSION,
      summary: {
        health: "warning",
        errorCount: 0,
        warningCount: 1,
        findingCount: 2,
        topFinding: {
          severity: "warning",
          code: "preview_only",
          title: "浏览器预览无法读取本机运行状态",
          impact: "真实问题需要在 Claude++ EXE 中生成诊断报告。",
          fixHint: "在桌面应用的诊断日志页点击重新生成并复制报告。",
          evidence: {},
        },
      },
      findings: [
        {
          severity: "warning",
          code: "preview_only",
          title: "浏览器预览无法读取本机运行状态",
          impact: "真实问题需要在 Claude++ EXE 中生成诊断报告。",
          fixHint: "在桌面应用的诊断日志页点击重新生成并复制报告。",
          evidence: {},
        },
        {
          severity: "info",
          code: "diagnostics_redacted",
          title: "报告只输出脱敏后的 key/token 状态",
          impact: "可用于定位配置是否存在，同时避免复制密钥原文。",
          fixHint: "不要手工补充 API key、token 或完整 Authorization 头。",
          evidence: { apiKeyPresent: true, apiKeyLength: 64 },
        },
      ],
      overview: {
        app: "Claude++",
        status: previewStatus,
        mappings: {
          status: "ok",
          provider_name: "Preview Provider",
          provider_id: "preview",
          mappings: [
            {
              display: "Claude Sonnet Preview",
              role: "sonnet",
              role_kind: "sonnet",
              model: "sonnet-preview",
            },
          ],
        },
        claude_zh: previewZhStatus,
        claude_enhance: previewEnhanceStatus(),
        developer_mode: {
          enabled: previewWelcomeStatus.developer_mode_enabled,
          candidates: [],
        },
      },
      checks: {
        settings: {
          effectiveProxyPort: __DEFAULT_PROXY_PORT__,
          effectiveProxyPortSource: "default",
        },
        gateway: {
          expectedPort: __DEFAULT_PROXY_PORT__,
          statusRunning: previewStatus.running,
          statusPort: previewStatus.port,
          tcpAcceptsExpectedPort: previewStatus.running,
          localGatewayToken: { exists: true, validFormat: true, length: 64 },
        },
        ccSwitch: {
          database: {
            exists: true,
            path: "%USERPROFILE%\\.cc-switch\\cc-switch.db",
          },
          proxyConfig: {
            status: "ok",
            proxyEnabled: true,
            listenAddress: "127.0.0.1",
            listenPort: 15721,
            reachable: true,
          },
          gatewayProfile: {
            status: "ok",
            baseUrl: {
              rawPresent: true,
              parseOk: true,
              scheme: "http",
              host: "127.0.0.1",
              port: 15721,
              path: "/v1",
            },
            apiKeyPresent: true,
            apiKeyLength: 64,
          },
          mappings: {
            status: "ok",
            count: 1,
            providerName: "Preview Provider",
            providerId: "preview",
          },
        },
        configLibrary: {
          isApplied: previewStatus.cd_applied,
          candidates: [
            {
              path: "%APPDATA%\\Claude\\configLibrary",
              exists: true,
              appliedId: "00000000-0000-4000-8000-000000157220",
              claudePlusEntry: {
                exists: true,
                baseUrl: {
                  rawPresent: true,
                  parseOk: true,
                  scheme: "http",
                  host: "127.0.0.1",
                  port: __DEFAULT_PROXY_PORT__,
                  path: "/claude-desktop",
                },
                apiKeyPresent: true,
                apiKeyLength: 64,
                portMatchesExpected: true,
                pathIsClaudeDesktop: true,
              },
            },
          ],
        },
        claudeDesktop: {
          supported: true,
          found: previewZhStatus.claude_found,
          running: false,
          files: { appAsar: { status: "ok", readable: true } },
        },
        logs: {
          path: "%USERPROFILE%\\.claude-plus-plus\\claude-plus-plus.log",
          exists: true,
          tailLineCount: 2,
          eventCounts: {
            "manager.proxy_status": 1,
            "manager.generate_diagnostics": 1,
          },
          recentErrors: [],
        },
      },
      paths: {
        appStateDir: "%USERPROFILE%\\.claude-plus-plus",
        ccSwitchDb: "%USERPROFILE%\\.cc-switch\\cc-switch.db",
        settings: "%USERPROFILE%\\.claude-plus-plus\\settings.json",
        diagnosticLog: "%USERPROFILE%\\.claude-plus-plus\\claude-plus-plus.log",
        localGatewayToken:
          "%USERPROFILE%\\.claude-plus-plus\\local-gateway-token",
      },
    },
    null,
    2,
  );
}

function clone<T>(value: T): T {
  return structuredClone(value);
}
