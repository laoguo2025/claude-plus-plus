import {
  FileText,
  Hammer,
  House,
  Info,
  Languages,
  Link2,
  SquareMousePointer,
  type LucideProps,
} from "lucide-react";
import type { ComponentType } from "react";
import type { Route } from "./appTypes";

export type Icon = ComponentType<LucideProps>;

export const QQ_GROUP_QR_PATH = "/qq-group-qr.png";
export const ALIPAY_QR_PATH = "/alipay-qr.png";
export const CLAUDE_DESKTOP_DOWNLOAD_URL = "https://pan.baidu.com/s/1vESMbIYVKFRVFBgA_5ap7Q?pwd=4pfe";
export const CC_SWITCH_DOWNLOAD_URL = "https://pan.baidu.com/s/1iJsHuCsLcSh9kvhp75PKxQ?pwd=jyw9";
export const GITHUB_REPOSITORY_URL = "https://github.com/laoguo2025/claude-plus-plus";
export const GITHUB_RELEASES_URL = `${GITHUB_REPOSITORY_URL}/releases/latest`;

export const routes: Array<{ id: Route; label: string; icon: Icon }> = [
  { id: "welcome", label: "欢迎使用", icon: House },
  { id: "overview", label: "CCS转接", icon: Link2 },
  { id: "localization", label: "一键汉化", icon: Languages },
  { id: "quick_access", label: "快捷入口", icon: SquareMousePointer },
  { id: "enhance", label: "页面增强", icon: Hammer },
  { id: "diagnostics", label: "诊断日志", icon: FileText },
  { id: "about", label: "Github仓库", icon: Info },
];

export const routeMeta: Record<Route, { title: string }> = {
  welcome: {
    title: "欢迎使用",
  },
  overview: {
    title: "CCS转接",
  },
  localization: {
    title: "一键汉化",
  },
  quick_access: {
    title: "快捷入口",
  },
  enhance: {
    title: "页面增强",
  },
  about: {
    title: "Github仓库",
  },
  diagnostics: {
    title: "诊断日志",
  },
};
