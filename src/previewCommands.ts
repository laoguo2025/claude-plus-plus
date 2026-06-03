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
};

const previewWelcomeStatus: WelcomeStatus = {
  claude_code_installed: false,
  developer_mode_enabled: true,
  cc_switch_installed: true,
};

export function previewEnhanceFeatures(): ClaudeEnhanceFeature[] {
  return enhanceFeatureDefinitions.map((feature) => ({ ...feature, enabled: false }));
}

export function previewCommand<T>(cmd: string): T {
  if (cmd === "app_version") {
    return PREVIEW_APP_VERSION as T;
  }
  if (cmd === "proxy_status") {
    return clone(previewStatus) as T;
  }
  if (cmd === "get_mappings") {
    throw new Error("浏览器预览无法读取 CC Switch 数据库；请在 Claude++ EXE 中查看真实服务商和模型映射。");
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
    return { report: previewDiagnosticsReport() } satisfies DiagnosticsPayload as T;
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
    path: "C:\\Users\\Administrator\\.claude-plus-plus\\claude-plus-plus.log",
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
      generatedAtMs: Date.now(),
      version: PREVIEW_APP_VERSION,
      overview: {
        app: "Claude++",
        status: previewStatus,
      },
      paths: {
        ccSwitchDb: "C:\\Users\\Administrator\\.cc-switch\\cc-switch.db",
        diagnosticLog: "C:\\Users\\Administrator\\.claude-plus-plus\\claude-plus-plus.log",
      },
    },
    null,
    2,
  );
}

function clone<T>(value: T): T {
  return structuredClone(value);
}
