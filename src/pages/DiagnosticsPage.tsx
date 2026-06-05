import { Activity, FileText } from "lucide-react";
import type { DiagnosticsPayload, LogsPayload } from "../appTypes";

export function DiagnosticsPage({
  diagnostics,
  diagnosticsBusy,
  logs,
  refreshDiagnostics,
  refreshLogs,
  copyText,
}: {
  diagnostics: DiagnosticsPayload | null;
  diagnosticsBusy: boolean;
  logs: LogsPayload | null;
  refreshDiagnostics: () => Promise<void>;
  refreshLogs: () => Promise<void>;
  copyText: (text: string) => Promise<void>;
}) {
  const logLines = splitLogLines(logs?.text ?? "");

  return (
    <div className="pageGrid diagnosticsPage">
      <section className="panel diagnosticsReportPanel">
        <div className="panelHead">
          <div>
            <h2>诊断报告</h2>
            <p>只读检查路由、模型映射、汉化、增强和关键本地路径，敏感信息会自动脱敏。</p>
          </div>
          <Activity size={20} />
        </div>
        <textarea
          className="diagnosticsReport"
          readOnly
          spellCheck={false}
          placeholder="点击生成报告后显示诊断内容。"
          value={diagnostics?.report ?? ""}
        />
        <div className="diagnosticsToolbar">
          <button disabled={diagnosticsBusy} onClick={() => void refreshDiagnostics()}>
            {diagnosticsBusy ? "生成中..." : diagnostics?.report ? "重新生成" : "生成报告"}
          </button>
          <button disabled={!diagnostics?.report} onClick={() => void copyText(diagnostics?.report ?? "")}>
            复制报告
          </button>
        </div>
      </section>

      <section className="panel diagnosticsLogPanel">
        <div className="panelHead">
          <div>
            <h2>最近日志</h2>
            <p>{logs?.path ?? "读取本地 Claude++ 诊断日志。"}</p>
          </div>
          <FileText size={20} />
        </div>
        <div className="logLines">
          {logLines.length ? (
            logLines.map((line, index) => (
              <div className="logLine" key={`log-${index}`}>
                <span>{index + 1}</span>
                <code>{line || " "}</code>
              </div>
            ))
          ) : (
            <div className="emptyInline">暂无日志。</div>
          )}
        </div>
        <div className="diagnosticsToolbar">
          <button onClick={() => void refreshLogs()}>刷新</button>
          <button disabled={!logs?.text} onClick={() => void copyText(logs?.text ?? "")}>
            复制
          </button>
        </div>
      </section>
    </div>
  );
}

function splitLogLines(text: string) {
  return text.split(/\r?\n/).filter((line) => line.length > 0);
}
