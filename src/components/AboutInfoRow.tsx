import type { ReactNode } from "react";

export function AboutInfoRow({ label, value, action }: { label: string; value: string; action?: ReactNode }) {
  return (
    <div className="aboutInfoRow">
      <span>{label}</span>
      <strong>{value}</strong>
      {action && <div className="aboutInfoAction">{action}</div>}
    </div>
  );
}
