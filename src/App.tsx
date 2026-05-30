import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Mapping {
  display: string;
  role: string;
  role_kind: string;
  model: string;
}
interface ProviderMappings {
  provider_name: string;
  provider_id: string;
  mappings: Mapping[];
}
interface StatusInfo {
  running: boolean;
  port: number | null;
  cd_applied: boolean;
}

function App() {
  const [status, setStatus] = useState<StatusInfo | null>(null);
  const [pm, setPm] = useState<ProviderMappings | null>(null);
  const [err, setErr] = useState<string>("");
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setErr("");
    try {
      setStatus(await invoke<StatusInfo>("proxy_status"));
    } catch (e) {
      setErr(String(e));
    }
    try {
      setPm(await invoke<ProviderMappings>("get_mappings"));
    } catch (e) {
      setErr(String(e));
      setPm(null);
    }
  }, []);

  useEffect(() => {
    refresh();
    const t = setInterval(refresh, 4000);
    return () => clearInterval(t);
  }, [refresh]);

  const run = async (cmd: string) => {
    setBusy(true);
    setErr("");
    try {
      await invoke(cmd);
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <main className="container">
      <h1>ccs2claude</h1>
      <p className="sub">CC Switch ↔ Claude Desktop 模型名桥接</p>

      <section className="card">
        <div className="statusRow">
          <span className={status?.running ? "dot on" : "dot off"} />
          <span>
            代理 {status?.running ? `运行中 (127.0.0.1:${status.port})` : "已停止"}
          </span>
          <div className="spacer" />
          <button disabled={busy} onClick={() => run(status?.running ? "stop_proxy" : "start_proxy")}>
            {status?.running ? "停止" : "启动"}
          </button>
        </div>
        <div className="statusRow">
          <span className={status?.cd_applied ? "dot on" : "dot off"} />
          <span>Claude Desktop {status?.cd_applied ? "已接入" : "未接入"}</span>
          <div className="spacer" />
          {status?.cd_applied ? (
            <button disabled={busy} onClick={() => run("revert_cd_config")}>
              撤销接入
            </button>
          ) : (
            <button disabled={busy} onClick={() => run("apply_cd_config")}>
              接入 Claude Desktop
            </button>
          )}
        </div>
      </section>

      <section className="card">
        <div className="statusRow">
          <strong>当前服务商:</strong>
          <span>{pm?.provider_name ?? "(读取失败)"}</span>
          <div className="spacer" />
          <button disabled={busy} onClick={refresh}>
            刷新
          </button>
        </div>
        <table className="map">
          <thead>
            <tr>
              <th>模型角色</th>
              <th>菜单显示名</th>
              <th>实际请求模型</th>
            </tr>
          </thead>
          <tbody>
            {pm?.mappings.map((m) => (
              <tr key={m.role}>
                <td>{m.role_kind}</td>
                <td>{m.display}</td>
                <td>{m.model}</td>
              </tr>
            ))}
            {!pm?.mappings.length && (
              <tr>
                <td colSpan={3} className="empty">
                  无映射
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </section>

      {err && <p className="err">{err}</p>}

      <section className="card todo">
        <strong>后续功能(规划中)</strong>
        <div className="todoRow">
          <button disabled title="规划中">一键下载 Claude Desktop</button>
          <button disabled title="规划中">一键汉化</button>
        </div>
      </section>
    </main>
  );
}

export default App;
