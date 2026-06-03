export interface Mapping {
  display: string;
  role: string;
  role_kind: string;
  model: string;
}

export interface ProviderMappings {
  provider_name: string;
  provider_id: string;
  mappings: Mapping[];
}

export interface StatusInfo {
  running: boolean;
  port: number | null;
  cd_applied: boolean;
  ccswitch_route: CcSwitchRouteStatus;
}

export interface CcSwitchRouteStatus {
  enabled: boolean;
  configured: boolean | null;
  has_mappings: boolean;
  reachable: boolean;
}

export interface ClaudeZhStatus {
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

export interface ClaudeEnhanceFeature {
  id: string;
  category: string;
  label: string;
  version: string;
  description: string;
  enabled: boolean;
  available: boolean;
  note: string;
}

export interface ClaudeEnhanceStatus {
  supported: boolean;
  claude_found: boolean;
  installed: boolean;
  backup_available: boolean;
  install_path: string | null;
  resources_path: string | null;
  features: ClaudeEnhanceFeature[];
}

export interface LogsPayload {
  path: string;
  text: string;
  lines: number;
}

export interface DiagnosticsPayload {
  report: string;
}

export interface WelcomeStatus {
  claude_code_installed: boolean;
  developer_mode_enabled: boolean;
  cc_switch_installed: boolean;
}

export type Route = "welcome" | "overview" | "localization" | "enhance" | "about" | "diagnostics";
export type Theme = "light" | "dark";
export type LocalizationScope = "complete" | "safe";
export type CommandArgs = Record<string, unknown>;
