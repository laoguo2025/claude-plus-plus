import type { ClaudeEnhanceStatus } from "../appTypes";
import { EnhanceCard } from "../components/EnhanceCard";
import { previewEnhanceFeatures } from "../previewCommands";
import type { EnhanceSection } from "./types";

const QUICK_ACCESS_FEATURE_IDS = new Set(["third_party_api", "plugins", "mcp"]);

export function EnhancePage({
  busy,
  section,
  enhanceStatus,
  installClaudeEnhance,
  uninstallClaudeEnhance,
}: {
  busy: boolean;
  section: EnhanceSection;
  enhanceStatus: ClaudeEnhanceStatus | null;
  installClaudeEnhance: (feature: string) => Promise<void>;
  uninstallClaudeEnhance: (feature: string) => Promise<void>;
}) {
  const disabledByMissingClaude = busy || !enhanceStatus?.supported || !enhanceStatus?.claude_found;
  const features = (enhanceStatus?.features ?? previewEnhanceFeatures()).filter((feature) =>
    section === "quick_access"
      ? QUICK_ACCESS_FEATURE_IDS.has(feature.id)
      : !QUICK_ACCESS_FEATURE_IDS.has(feature.id),
  );

  return (
    <div className="enhanceFlow">
      <div className="actionNotice enhanceActionNotice">
        {enhanceStatus?.claude_found === false
          ? "未定位到 Claude Desktop 资源目录；请重新安装 Claude Desktop，或在 Claude++ 设置中指定资源路径后刷新。"
          : "增强脚本开启后，需点击上方重启Claude Desktop按钮，让页面立即生效。"}
      </div>
      <section className="panel enhanceCardsPanel">
        <div className="enhanceCards">
          {features.map((feature) => (
            <EnhanceCard
              key={feature.id}
              feature={feature}
              disabled={disabledByMissingClaude || !feature.available}
              onInstall={() => installClaudeEnhance(feature.id)}
              onUninstall={() => uninstallClaudeEnhance(feature.id)}
            />
          ))}
        </div>
      </section>
    </div>
  );
}
