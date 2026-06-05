import type { ProviderMappings, StatusInfo } from "../appTypes";
import { RouteStatusCard } from "../components/RouteStatusCard";
import { MiniMapping } from "../components/MiniMapping";
import { routeSummaryText, ccswitchRouteDetailText, claudeRouteDetailText } from "../routeStatus";

export function OverviewPage({
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
  const claudeRouteOn = ccswitchRoute?.claude_route_enabled === true;
  const ccswitchSwitchOn = ccswitchRoute?.proxy_enabled === true;
  const ccswitchSwitchValue = ccswitchSwitchOn
    ? ccswitchRoute?.reachable === false
      ? "端口不可达"
      : "已开启"
    : "未开启";
  const claudeRouteDetail = claudeRouteDetailText(ccswitchRoute);
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
            active={claudeRouteOn}
            label="Claude 路由开关"
            value={claudeRouteOn ? "已开启" : "未开启"}
            detail={claudeRouteDetail}
          />
          <RouteStatusCard
            active={ccswitchSwitchOn && ccswitchRoute?.reachable !== false}
            label="CC Switch 路由总开关"
            value={ccswitchSwitchValue}
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
          <span>模型服务商配置</span>
          <div>
            <strong>{providerConfigured ? "已配置" : "未配置"}</strong>
            {!providerConfigured && <small>{mappingError || "请在 CC Switch 中配置模型服务商"}</small>}
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
