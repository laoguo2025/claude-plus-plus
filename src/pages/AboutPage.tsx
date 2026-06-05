import { useEffect, useMemo, useState } from "react";
import type { GithubReleaseAsset, GithubReleaseInfo } from "../appTypes";
import { GITHUB_RELEASES_URL, GITHUB_REPOSITORY_URL } from "../appConstants";
import { AboutInfoRow } from "../components/AboutInfoRow";
import { callCommand, openExternalUrl } from "../tauriClient";

export function AboutPage({
  appVersion,
  claudeDesktopVersion,
  setErr,
}: {
  appVersion: string;
  claudeDesktopVersion: string;
  setErr: (error: string) => void;
}) {
  const [release, setRelease] = useState<GithubReleaseInfo | null>(null);
  const [releaseStatus, setReleaseStatus] = useState(
    "正在检查 GitHub Release...",
  );
  const [releaseLoading, setReleaseLoading] = useState(false);

  const checkLatestRelease = async () => {
    setReleaseLoading(true);
    setReleaseStatus("正在检查 GitHub Release...");
    try {
      const latest = await callCommand<GithubReleaseInfo>(
        "latest_github_release",
      );
      setRelease(latest);
      setReleaseStatus(
        compareVersions(appVersion, latest.version) < 0
          ? "发现新版本"
          : "当前已是最新版",
      );
    } catch (e) {
      setRelease(null);
      setReleaseStatus(`检查失败: ${String(e)}`);
    } finally {
      setReleaseLoading(false);
    }
  };

  useEffect(() => {
    void checkLatestRelease();
  }, [appVersion]);

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

  const openAsset = async (asset: GithubReleaseAsset) => {
    setErr("");
    try {
      await openExternalUrl(asset.url);
    } catch (e) {
      setErr(String(e));
    }
  };

  const releaseNotes = useMemo(
    () =>
      release
        ? release.body || `${release.name}\n\n此 Release 未填写更新说明。`
        : [
            `当前应用版本: ${appVersion}`,
            "",
            "Claude++ 正在读取 GitHub Release 最新内容。如果检查失败，请打开 GitHub Release，或使用本地 release 构建脚本。",
          ].join("\n"),
    [appVersion, release],
  );

  return (
    <div className="pageGrid aboutPage">
      <section className="panel aboutPanel">
        <div className="aboutInfoTable">
          <AboutInfoRow label="Claude++ 版本" value={appVersion} />
          <AboutInfoRow
            label="Claude Desktop 版本"
            value={claudeDesktopVersion}
          />
          <AboutInfoRow
            label="仓库地址"
            value={GITHUB_REPOSITORY_URL}
            action={
              <button onClick={() => void openRepository()}>打开仓库</button>
            }
          />
          <div className="releaseCard">
            <div className="releaseCardHead">
              <strong>GitHub Release 更新</strong>
              <span>当前版本 {appVersion}</span>
            </div>
            <AboutInfoRow
              label="状态"
              value={releaseStatus}
              action={
                <button
                  disabled={releaseLoading}
                  onClick={() => void checkLatestRelease()}
                >
                  {releaseLoading ? "检查中" : "刷新"}
                </button>
              }
            />
            <AboutInfoRow
              label="最新版本"
              value={
                release
                  ? `${release.name} (${release.tag_name})`
                  : "请以 GitHub Release 为准"
              }
              action={
                <button onClick={() => void openReleases()}>
                  打开 Release
                </button>
              }
            />
            <AboutInfoRow
              label="发布时间"
              value={formatPublishedAt(release?.published_at)}
            />
            <div className="aboutInfoRow releaseAssetsRow">
              <span>资源</span>
              <strong>
                {release
                  ? `${release.assets.length} 个安装包`
                  : "GitHub Release 与本地构建脚本"}
              </strong>
              {release && (
                <div className="releaseAssetActions">
                  {release.assets.map((asset) => (
                    <button
                      key={asset.name}
                      onClick={() => void openAsset(asset)}
                    >
                      {assetLabel(asset)}
                    </button>
                  ))}
                </div>
              )}
            </div>
            <textarea className="releaseNotes" readOnly value={releaseNotes} />
          </div>
        </div>
      </section>
    </div>
  );
}

function assetLabel(asset: GithubReleaseAsset) {
  switch (asset.kind) {
    case "windows":
      return "Windows";
    case "macosArm64":
      return "macOS Apple";
    case "macosX64":
      return "macOS Intel";
    default:
      return asset.name;
  }
}

function formatPublishedAt(value: string | null | undefined) {
  if (!value) return "未知";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

function compareVersions(current: string, latest: string) {
  const left = current.split(".").map((part) => Number.parseInt(part, 10) || 0);
  const right = latest.split(".").map((part) => Number.parseInt(part, 10) || 0);
  const length = Math.max(left.length, right.length);
  for (let i = 0; i < length; i += 1) {
    const diff = (left[i] || 0) - (right[i] || 0);
    if (diff !== 0) return diff;
  }
  return 0;
}
