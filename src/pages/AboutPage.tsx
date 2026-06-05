import { GITHUB_RELEASES_URL, GITHUB_REPOSITORY_URL } from "../appConstants";
import { AboutInfoRow } from "../components/AboutInfoRow";
import { openExternalUrl } from "../tauriClient";

export function AboutPage({
  appVersion,
  claudeDesktopVersion,
  setErr,
}: {
  appVersion: string;
  claudeDesktopVersion: string;
  setErr: (error: string) => void;
}) {
  const openRepository = async () => {
    setErr("");
    try {
      await openExternalUrl(GITHUB_REPOSITORY_URL);
    } catch (e) {
      setErr(String(e));
    }
  };

  const openReleases = async () => {
    setErr("");
    try {
      await openExternalUrl(GITHUB_RELEASES_URL);
    } catch (e) {
      setErr(String(e));
    }
  };

  return (
    <div className="pageGrid aboutPage">
      <section className="panel aboutPanel">
        <div className="aboutInfoTable">
          <AboutInfoRow label="Claude++ 版本" value={appVersion} />
          <AboutInfoRow label="Claude Desktop 版本" value={claudeDesktopVersion} />
          <AboutInfoRow
            label="仓库地址"
            value={GITHUB_REPOSITORY_URL}
            action={<button onClick={() => void openRepository()}>打开仓库</button>}
          />
          <div className="releaseCard">
            <div className="releaseCardHead">
              <strong>GitHub Release 更新</strong>
              <span>当前版本 {appVersion}</span>
            </div>
            <AboutInfoRow label="状态" value="未接入自动检查" />
            <AboutInfoRow
              label="最新版本"
              value="请以 GitHub Release 为准"
              action={<button onClick={() => void openReleases()}>打开 Release</button>}
            />
            <AboutInfoRow label="资源" value="GitHub Release 与本地构建脚本" />
            <textarea
              className="releaseNotes"
              readOnly
              value={[
                `当前应用版本: ${appVersion}`,
                "",
                "1.0.1 更新内容:",
                "- 新增 Win 虚拟机平台自动开启。",
                "- 新增一键开启 Claude Desktop 开发者模式。",
                "- 优化一键汉化/恢复、启动状态检测、路由桥接、UI 布局、部分路径和诊断日志。",
                "- 修复了一些 bug。",
                "",
                "Claude++ 目前未接入自动更新。需要确认新版时，请打开 GitHub Release，或使用本地 release 构建脚本。",
              ].join("\n")}
            />
          </div>
        </div>
      </section>
    </div>
  );
}
