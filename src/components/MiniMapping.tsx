import type { Mapping } from "../appTypes";

export function MiniMapping({ mappings, emptyText }: { mappings: Mapping[]; emptyText: string }) {
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
