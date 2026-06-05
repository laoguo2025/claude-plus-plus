import { Activity, Code2, Gauge, ListRestart, FileText, Network, PackageCheck, Plug } from "lucide-react";
import type { ClaudeEnhanceFeature } from "../appTypes";
import type { Icon } from "../appConstants";

export function EnhanceCard({
  feature,
  disabled,
  onInstall,
  onUninstall,
}: {
  feature: ClaudeEnhanceFeature;
  disabled: boolean;
  onInstall: () => void;
  onUninstall: () => void;
}) {
  const IconComponent = enhanceIcon(feature.id);
  return (
    <div className={`workflowRow enhanceWorkflowRow ${feature.enabled ? "enabled" : ""}`}>
      <div className={`rowIcon ${feature.enabled ? "success" : "warning"}`}>
        <IconComponent size={17} />
      </div>
      <div className="workflowCopy">
        <strong>
          {feature.label}
          <span className="enhanceVersion">{feature.version}</span>
          {feature.id === "conversation_title_i18n" && (
            <span className="enhanceTokenNotice">会消耗少量 token</span>
          )}
        </strong>
        <span>{feature.description}</span>
      </div>
      <div className="enhanceActions">
        <button className="primary" disabled={disabled || feature.enabled} onClick={onInstall}>
          安装
        </button>
        <button disabled={disabled || !feature.enabled} onClick={onUninstall}>
          卸载
        </button>
      </div>
    </div>
  );
}

function enhanceIcon(id: string): Icon {
  if (id === "third_party_api") return Code2;
  if (id === "plugins") return PackageCheck;
  if (id === "mcp") return Network;
  if (id === "conversation_title_i18n") return ListRestart;
  if (id === "markdown") return FileText;
  if (id === "timeline") return Activity;
  if (id === "token_usage") return Gauge;
  return Plug;
}
