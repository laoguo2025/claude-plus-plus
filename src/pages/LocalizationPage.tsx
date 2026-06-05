import { CheckCircle2 } from "lucide-react";
import type { ClaudeZhStatus, LocalizationScope, WelcomeStatus } from "../appTypes";
import { WorkflowRow } from "../components/WorkflowRow";

export function LocalizationPage({
  busy,
  zhStatus,
  welcomeStatus,
  zhScope,
  setZhScope,
  enableClaudeDeveloperMode,
  installClaudeZh,
  backupClaudeZh,
  uninstallClaudeZh,
}: {
  busy: boolean;
  zhStatus: ClaudeZhStatus | null;
  welcomeStatus: WelcomeStatus | null;
  zhScope: LocalizationScope;
  setZhScope: (value: LocalizationScope) => void;
  enableClaudeDeveloperMode: () => Promise<void>;
  installClaudeZh: () => Promise<void>;
  backupClaudeZh: () => Promise<void>;
  uninstallClaudeZh: () => Promise<void>;
}) {
  const statusText = zhStatus?.installed
    ? `已安装简体中文汉化，语言文件 ${zhStatus.language_files.join(", ")}`
    : zhStatus?.claude_found
      ? "未安装简体中文汉化"
      : "未定位到 Claude Desktop 资源目录，可重新安装或在 Claude++ 设置中指定资源路径";
  const installPercent = zhStatus?.installed ? "已汉化" : zhStatus?.claude_found ? "未汉化" : "未定位";
  const scopeText =
    zhScope === "complete"
      ? "完整汉化：语言文件、前端文案、菜单与 3P 模型校验补丁"
      : "安全汉化：跳过 app.asar 与 Claude.exe 完整性补丁";
  const disabledByMissingClaude = busy || !zhStatus?.supported || !zhStatus?.claude_found;
  const developerModeEnabled = welcomeStatus?.developer_mode_enabled === true;
  const developerModeLoading = welcomeStatus === null;

  return (
    <div className="localizationFlow">
      <section className="panel developerModePanel">
        <div className="workflowRows">
          <WorkflowRow
            ok={developerModeEnabled}
            title="开发者模式"
            description={
              developerModeLoading
                ? "正在检测 Claude Desktop 开发者模式状态。"
                : developerModeEnabled
                  ? "已开启 Claude Desktop 开发者模式。"
                  : "开启后可支持开发与汉化相关能力。"
            }
            tone={developerModeEnabled ? "success" : "warning"}
            badge={developerModeEnabled ? "已开启" : undefined}
            action={
              !developerModeLoading && !developerModeEnabled ? (
                <button disabled={busy} onClick={enableClaudeDeveloperMode}>
                  一键开启
                </button>
              ) : undefined
            }
          />
        </div>
      </section>
      <section className="panel localizationChecklist">
        <div className="workflowRows">
          <WorkflowRow
            ok={!!zhStatus?.installed}
            title="检测汉化程度"
            description={statusText}
            badge={installPercent}
            tone={zhStatus?.installed ? "success" : zhStatus?.claude_found ? "warning" : "danger"}
          />
          <WorkflowRow
            ok={!!zhStatus?.backup_available}
            title="检测语言文件备份状态"
            description={
              zhStatus?.backup_available
                ? "已发现可恢复备份，可随时恢复英文。"
                : "未发现可恢复备份；建议先备份当前 Claude Desktop 资源。"
            }
            badge={zhStatus?.backup_available ? "可恢复" : undefined}
            tone={zhStatus?.backup_available ? "success" : "warning"}
            action={
              !zhStatus?.backup_available ? (
                <button disabled={disabledByMissingClaude} onClick={backupClaudeZh}>
                  备份
                </button>
              ) : undefined
            }
          />
          <div className="workflowRow">
            <div className="rowIcon success">
              <CheckCircle2 size={16} />
            </div>
            <div className="workflowCopy">
              <strong>选择汉化范围</strong>
              <span>{scopeText}</span>
            </div>
            <div className="scopeSelect" role="group" aria-label="汉化范围">
              <button
                className={zhScope === "complete" ? "active" : ""}
                disabled={busy}
                onClick={() => setZhScope("complete")}
              >
                完整汉化
              </button>
              <button
                className={zhScope === "safe" ? "active" : ""}
                disabled={busy}
                onClick={() => setZhScope("safe")}
              >
                安全汉化
              </button>
            </div>
          </div>
          <WorkflowRow
            ok={!!zhStatus?.claude_found}
            title="一键汉化"
            description="默认安装简体中文，不再提供语言选择。安装时会自动写入必要备份。"
            action={
              <button className="primary" disabled={disabledByMissingClaude} onClick={installClaudeZh}>
                一键汉化
              </button>
            }
          />
          <WorkflowRow
            ok={!zhStatus?.installed}
            title="恢复英文"
            description="只移除中文语言资源并把语言设回 en-US，不覆盖页面增强脚本。"
            action={
              <button disabled={busy || !zhStatus?.installed} onClick={uninstallClaudeZh}>
                一键恢复
              </button>
            }
          />
        </div>
      </section>
    </div>
  );
}
