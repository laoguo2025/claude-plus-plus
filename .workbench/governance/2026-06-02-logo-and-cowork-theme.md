# 2026-06-02 Logo 与 Cowork 风格配色

## 原因

用户要求把 Claude++ 全部 logo 图标替换为 `C:\Users\Administrator\Desktop\bot2.png`，并参考 `C:\Users\Administrator\Desktop\Claude Cowork 设计文档.md` 优化整体配色。

## 范围

- 使用 `bot2.png` 生成并替换 `src-tauri/icons/` 下现有桌面端 PNG、ICO、ICNS、Windows Store logo。
- 侧栏品牌图标由文字 `C++` 改为同源 bot 图标。
- 配色 token 调整为 Cowork 风格暖白/炭黑层级，保留 Clay 品牌橙作为选中态与强调色。
- 新增轻量语义变量：品牌柔和底、状态底、状态边框、面板阴影。

## 非变化范围

- 不修改功能入口、路由和页面结构；补丁中仅为 NSIS 安装器/卸载器补充图标配置。
- `tauri icon` 生成的移动端图标目录已清理，避免纳入当前 Windows/NSIS 范围。

## 回滚依据

如需恢复旧视觉，可回退本次提交；图标替换不影响路由、汉化、页面增强和诊断日志功能。

## 验证

- `npm run build`：通过。
- 浏览器预览：亮/暗主题 logo 加载正常，暗色层级为 `#0d0d0d / #171716 / #1f1f1e / #2c2c2a`。
- `git diff --check`：通过，仅出现 Windows 换行提示。
- `cargo test --manifest-path src-tauri\Cargo.toml --lib`：通过，36 passed，3 ignored。
- Release 打包：通过，已生成 `src-tauri\target\release\claude-plus-plus.exe` 和 `src-tauri\target\release\bundle\nsis\Claude++_0.1.0_x64-setup.exe`。
- 安装包图标补丁：生成的 `installer.nsi` 中 `INSTALLERICON` 和 `UNINSTALLERICON` 均指向 `src-tauri\icons\icon.ico`，Release 打包通过并重新生成 NSIS 安装包。

## 2026-06-02 配色收敛补丁

用户反馈浅色/深色主题状态色过多，页面视觉混乱。本次仅收敛 `src/App.css`：

- 正常页面主视觉限制为 Cowork 背景层级、文本层级、边框层级和 Clay 品牌橙。
- 蓝色不再用于顶部说明或大面积提示；成功、警告状态不再铺满卡片背景。
- 已启用、警告、token 提醒等状态改为细边框、小标签、左侧细标记等小面积表达。
- CSS 颜色表达从全局 54 个收敛到 32 个；浅色主题 token 从 25 个收敛到 17 个，深色主题 token 从 25 个收敛到 16 个。
- 浏览器预览检查了浅色/深色 `CCS转接` 与 `页面增强`，页面主视觉已收敛为中性色 + Clay 橙。
- `npm run build`：通过。
- Release 打包：第一次因 `src-tauri\target\release\claude-plus-plus.exe` 被运行中进程占用失败；仅停止该 release exe 进程后重跑通过，已重新生成 NSIS 安装包。

## 2026-06-02 透明图标补丁

用户反馈 exe、安装包、窗口左上角、任务栏、托盘、桌面快捷方式和侧栏 logo 仍显示白色背景。排查确认 `bot2.png` 源图本身四角为不透明白色，上一轮生成的 PNG/ICO 也没有透明像素。

- 从 `bot2.png` 生成透明背景版本，使用四角连通白底泛洪移除背景，避免删除图案内部非连通浅色细节。
- 使用透明背景版本重新生成 Windows/NSIS 相关 Tauri 图标资源。
- 移除侧栏 `.brandMark` 自带背景、圆角和阴影，让图片透明区域直接透出页面背景。
- 验证 `icon.png`、`32x32.png`、`128x128.png`、`StoreLogo.png`、`Square44x44Logo.png` 四角 alpha 均为 0；`icon.ico` 的 16/24/32/48/64/256 帧四角 alpha 均为 0。
- 浏览器预览验证侧栏 logo 容器背景为透明、阴影为 none，图片 natural size 为 512x512。
- 发现 `claude-plus-plus.exe` 初次重打包仍嵌入旧白底图标，根因是 Rust build script 未因 `icons/icon.ico` 内容变化重跑资源链接；已在 `src-tauri/build.rs` 增加 `cargo:rerun-if-changed=icons/icon.ico`。
- 清理当前 crate 的 release build 缓存后重打包，并从 `claude-plus-plus.exe` 与 `Claude++_0.1.0_x64-setup.exe` 的 PE 图标资源中抽取验证：16/24/32/48/64/256 全尺寸四角 alpha 均为 0。
- `cargo test --manifest-path src-tauri\Cargo.toml --lib`：通过，36 passed，3 ignored。

## 2026-06-02 概览页留白收敛补丁

用户反馈 `CCS转接` 页顶部说明与步骤卡之间间距过大、模型映射卡底部留白过大。本次仅调整 `src/App.css`：

- `.overviewPage` 从固定三行高度改为按内容高度排列，并用 `align-content: start` 避免最后一块撑满剩余空间。
- `.pageGrid` 间距从 12px 收敛到 10px。
- 路由步骤卡最小高度从 82px 降到 74px。
- 浏览器预览验证 `CCS转接` 三块高度为 103px / 141px / 159px，映射卡不再撑到页面底部。
- 用户要求本轮不打包；仅执行 `npm run build` 验证。

## 2026-06-02 提醒文本样式对齐补丁

用户要求 `一键汉化` 页顶部提醒样式对齐 `CCS转接` 页说明文本，去掉浅橙色容器和灰色背景。本次仅调整 `.actionNotice`：

- 移除边框、圆角、背景和左右/顶部 padding。
- 改为普通文本流样式，保留 Clay 强调色、12.5px 字号和 1.5 行高。
- 浏览器预览验证 `.actionNotice` 背景为透明、边框为 0、左右 padding 为 0。
- 仅执行 `npm run build` 验证，未打包。

## 2026-06-02 页面增强样式对齐补丁

用户要求 `页面增强` 页顶部提醒对齐 `CCS转接` 顶部说明，下方增强项卡片对齐 `一键汉化` 流程卡片。本次仅调整 `src/App.css`：

- `enhanceActionNotice` 继续使用普通文本流样式，无容器背景、边框和左右 padding。
- 移除已启用增强项的橙色左边条和橙色边框，统一使用中性卡片底与中性边框。
- 增强分类标签从橙色胶囊改为中性胶囊，和 token 提醒标签保持同一层级。
- 浏览器预览验证 `页面增强` 顶部提醒背景透明、边框为 0；增强项卡片背景为中性层级、边框为 `border-soft`、无 box-shadow。
- 仅执行 `npm run build` 验证，未打包。
