export type RouteStatusAction = {
  label: string;
  onClick: () => void;
  disabled: boolean;
  primary?: boolean;
};

export function RouteStatusCard({
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
  action?: RouteStatusAction;
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
