import { useEffect, type MutableRefObject } from "react";
import type { WelcomeStatus } from "../appTypes";
import { ALIPAY_QR_PATH, CC_SWITCH_DOWNLOAD_URL, CLAUDE_DESKTOP_DOWNLOAD_URL, QQ_GROUP_QR_PATH } from "../appConstants";
import { QrCard } from "../components/QrCard";
import { RouteStatusCard } from "../components/RouteStatusCard";
import { openExternalUrl } from "../tauriClient";
import botLogo from "../../src-tauri/icons/icon.png";

export function WelcomePage({
  busy,
  welcomeStatus,
  setErr,
  enableVirtualMachinePlatform,
  virtualMachinePlatformEnableRequested,
  installClaudeCode,
}: {
  busy: boolean;
  welcomeStatus: WelcomeStatus | null;
  setErr: (error: string) => void;
  enableVirtualMachinePlatform: () => Promise<void>;
  virtualMachinePlatformEnableRequested: MutableRefObject<boolean>;
  installClaudeCode: () => Promise<void>;
}) {
  const loading = welcomeStatus === null;
  const virtualMachinePlatformSupported = welcomeStatus?.virtual_machine_platform_supported === true;
  const virtualMachinePlatformEnabled = welcomeStatus?.virtual_machine_platform_enabled === true;
  const virtualMachinePlatformPending =
    !loading &&
    virtualMachinePlatformSupported &&
    !virtualMachinePlatformEnabled &&
    virtualMachinePlatformEnableRequested.current;

  useEffect(() => {
    if (
      loading ||
      !virtualMachinePlatformSupported ||
      virtualMachinePlatformEnabled ||
      virtualMachinePlatformEnableRequested.current
    ) {
      return;
    }
    virtualMachinePlatformEnableRequested.current = true;
    enableVirtualMachinePlatform().catch((e) => {
      setErr(String(e));
    });
  }, [
    enableVirtualMachinePlatform,
    loading,
    setErr,
    virtualMachinePlatformEnableRequested,
    virtualMachinePlatformEnabled,
    virtualMachinePlatformSupported,
  ]);

  const downloadClaudeDesktop = async () => {
    setErr("");
    try {
      await openExternalUrl(CLAUDE_DESKTOP_DOWNLOAD_URL);
    } catch (e) {
      setErr(String(e));
    }
  };

  const downloadCcSwitch = async () => {
    setErr("");
    try {
      await openExternalUrl(CC_SWITCH_DOWNLOAD_URL);
    } catch (e) {
      setErr(String(e));
    }
  };

  return (
    <div className="welcomePage">
      <section className="welcomeHero">
        <div className="welcomeIntro">
          <img className="welcomeLogo" src={botLogo} alt="Claude++" />
          <div className="welcomeCopy">
            <h2>Claude++</h2>
            <p>Claude++，是一款 Claude Desktop 的本地增强工具，</p>
            <p>提供 CCS 转接优化、一键汉化、第三方API接入、对话增强等能力。</p>
          </div>
        </div>
        <div className="welcomeQrGroup">
          <QrCard
            kind="qq"
            src={QQ_GROUP_QR_PATH}
            alt="QQ交流群二维码"
            text="QQ群：582589880，欢迎交流反馈，提出建议。"
          />
          <QrCard
            kind="alipay"
            src={ALIPAY_QR_PATH}
            alt="个人支付宝收款码"
            text="如果 Claude++ 帮到了你，可用支付宝支持一下。"
          />
        </div>
      </section>

      <p className="welcomeActionHint">
        如果下方几项显示未安装/未开启，可直接点击卡片进行下载/开启。
        <br />
        下载会跳转百度网盘连接，无需魔法登录github。
      </p>

      <section className="welcomeStatusGrid" aria-label="环境状态检测">
        <RouteStatusCard
          loading={loading}
          active={virtualMachinePlatformEnabled}
          label="Win虚拟机平台"
          value={
            loading
              ? "检测中"
              : !virtualMachinePlatformSupported
                ? "不支持"
                : virtualMachinePlatformEnabled
                  ? "已开启"
                  : "未开启"
          }
          detail={
            loading
              ? undefined
              : !virtualMachinePlatformSupported
                ? "仅 Windows 需要检测"
                : virtualMachinePlatformEnabled
                  ? undefined
                  : virtualMachinePlatformPending
                    ? "已发起开启，请重启电脑"
                    : "将自动开启 WSL 与虚拟机平台"
          }
        />
        <RouteStatusCard
          loading={loading}
          active={!!welcomeStatus?.claude_code_installed}
          label="Claude Code"
          value={loading ? "检测中" : welcomeStatus?.claude_code_installed ? "已安装" : "未安装"}
          detail={loading ? undefined : welcomeStatus?.claude_code_installed ? undefined : "点击后一键命令行安装"}
          action={
            loading || welcomeStatus?.claude_code_installed
              ? undefined
              : {
                  label: "一键安装",
                  onClick: () => void installClaudeCode(),
                  disabled: busy,
                  primary: true,
                }
          }
        />
        <RouteStatusCard
          loading={loading}
          active={!!welcomeStatus?.claude_desktop_found}
          label="Claude Desktop"
          value={loading ? "检测中" : welcomeStatus?.claude_desktop_found ? "已安装" : "未安装"}
          detail={loading ? undefined : welcomeStatus?.claude_desktop_found ? undefined : "点击后从网盘下载"}
          action={
            loading || welcomeStatus?.claude_desktop_found
              ? undefined
              : {
                  label: "下载",
                  onClick: () => void downloadClaudeDesktop(),
                  disabled: busy,
                  primary: true,
                }
          }
        />
        <RouteStatusCard
          loading={loading}
          active={!!welcomeStatus?.cc_switch_installed}
          label="CC Switch"
          value={loading ? "检测中" : welcomeStatus?.cc_switch_installed ? "已安装" : "未安装"}
          detail={loading ? undefined : welcomeStatus?.cc_switch_installed ? undefined : "点击后从网盘下载"}
          action={
            loading || welcomeStatus?.cc_switch_installed
              ? undefined
              : {
                  label: "下载",
                  onClick: () => void downloadCcSwitch(),
                  disabled: busy,
                  primary: true,
                }
          }
        />
      </section>
    </div>
  );
}
