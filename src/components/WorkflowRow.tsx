import type { ReactNode } from "react";
import { CheckCircle2, CircleAlert } from "lucide-react";

export function WorkflowRow({
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
