interface CcSwitchRouteStatus {
  enabled: boolean;
  configured: boolean | null;
  has_mappings: boolean;
  reachable: boolean;
}

interface StatusInfo {
  running: boolean;
  port: number | null;
  cd_applied: boolean;
  ccswitch_route: CcSwitchRouteStatus;
}

export function routeSummaryText(status: StatusInfo | null): string {
  if (!status?.cd_applied) {
    return "Claude Desktop 当前未接入 Claude++";
  }
  if (!status.running) {
    return "Claude Desktop 已配置接入 Claude++，但本地代理未运行";
  }
  return "Claude Desktop 当前接入 Claude++ 本地代理，代理运行中";
}

export function ccswitchRouteDetailText(
  route: CcSwitchRouteStatus | null | undefined,
): string | undefined {
  if (!route || route.configured === null) {
    return "无法读取 CC Switch 路由配置";
  }
  if (!route.configured) {
    return "请在 CCS 开启路由";
  }
  if (route.reachable === false) {
    return "路由已开启，端口暂不可达或仍在启动中";
  }
  return undefined;
}
