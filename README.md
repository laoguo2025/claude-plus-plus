# Claude++

<p align="center">
  <img src="src-tauri/icons/icon.png" width="96" alt="Claude++ icon" />
</p>

<p align="center">
  <strong>Claude Desktop enhancement toolkit: page enhancements, Chinese localization, CC Switch routing, diagnostics, and cross-platform installers.</strong>
</p>

<p align="center">
  <a href="https://github.com/laoguo2025/claude-plus-plus/releases/latest"><img alt="release" src="https://img.shields.io/github/v/release/laoguo2025/claude-plus-plus?label=release&style=flat-square&labelColor=555&color=007ec6"></a>
  <a href="https://github.com/laoguo2025/claude-plus-plus/actions/workflows/macos-dmg.yml"><img alt="Build macOS DMG" src="https://img.shields.io/github/actions/workflow/status/laoguo2025/claude-plus-plus/macos-dmg.yml?branch=main&label=Build%20macOS%20DMG&style=flat-square&labelColor=555"></a>
  <a href="LICENSE"><img alt="license" src="https://img.shields.io/badge/license-MIT-97ca00?style=flat-square&labelColor=555"></a>
  <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24c8db?style=flat-square&labelColor=555">
  <img alt="Rust backend" src="https://img.shields.io/badge/Rust-backend-b7410e?style=flat-square&labelColor=555">
  <img alt="React frontend" src="https://img.shields.io/badge/React-frontend-61dafb?style=flat-square&labelColor=555">
</p>

[中文](#中文) · [English](#english)

---

## 中文

### 快速下载

前往 [GitHub Releases](https://github.com/laoguo2025/claude-plus-plus/releases/latest) 下载最新版安装包。

| 平台 | 安装包 |
| --- | --- |
| Windows x64 | `Claude++_1.0.0_x64-setup.exe` |
| macOS Apple Silicon | `Claude++_1.0.0_aarch64.dmg` |
| macOS Intel | `Claude++_1.0.0_x64.dmg` |

### 这是什么

Claude++ 是一个面向 Claude Desktop 的本地增强控制台。它把常用的 Claude Desktop 改造能力集中到一个 Tauri 桌面应用里：安装页面增强、查看 token 用量、导出 Markdown、增加 Skills / MCP / 第三方 API 入口、安装简体中文资源、接入 CC Switch 模型路由，并在出问题时生成诊断报告。

它不是云服务，也不接管你的上游账号。Claude++ 运行在本机，按需修改 Claude Desktop 本地资源或配置；路由桥接场景下，它只读 CC Switch 配置并把请求继续转发给 CC Switch。

### 核心功能

- **页面增强优先**：为 Claude Desktop 增加第三方 API、Skills、MCP 快捷入口，并支持对话标题中文化、Markdown 导出、对话时间线和 token 使用信息。
- **一键汉化与恢复**：安装简体中文语言资源，保留备份，并支持恢复英文。
- **CC Switch 转接**：让 Claude Desktop 3P 模型菜单显示 CC Switch 的自定义模型名，同时保持真实请求仍回到 CC Switch 路由。
- **本地代理与可逆接管**：默认监听 `127.0.0.1:15722`，写入独立的 Claude++ 配置条目，不覆盖 CC Switch 原始条目，可随时断开接管。
- **环境准备检查**：检测 Claude Code、Claude Desktop、开发者模式、CC Switch 状态，并提供快捷安装/开启入口。
- **诊断与排查**：读取本地日志，生成包含路由、模型映射、汉化、增强状态的诊断报告。
- **跨平台分发**：提供 Windows 安装包和 macOS Apple Silicon / Intel DMG。

### 使用方式

#### 1. 准备环境

![欢迎使用](docs/images/claude-plus-plus-welcome.png)

欢迎页会检查 Claude Code、Claude Desktop、开发者模式和 CC Switch。缺少组件时，可以直接从卡片入口下载或开启。

#### 2. 接入 CC Switch 路由

![CCS 转接](docs/images/claude-plus-plus-route.png)

在「CCS转接」页查看 CC Switch 路由开关、Claude++ 接管状态和当前模型映射。接管后，Claude Desktop 菜单显示 CC Switch 的自定义模型名；实际请求仍按 CC Switch 的角色和上游配置发送。

#### 3. 安装或恢复中文化

![一键汉化](docs/images/claude-plus-plus-localization.png)

在「一键汉化」页安装简体中文资源，选择完整汉化或安全汉化；如果需要回退，可以从备份恢复英文资源。

#### 4. 管理 Claude Desktop 页面增强

![页面增强](docs/images/claude-plus-plus-enhance.png)

在「页面增强」页安装或卸载增强项。当前支持第三方 API、Skills、MCP 入口，对话列表中文化，Markdown 导出，对话时间线和 token 使用信息。安装后重启 Claude Desktop 即可生效。

#### 5. 查看诊断与版本

「诊断日志」页用于生成诊断报告和复制最近日志；「Github仓库」页用于查看当前 Claude++ / Claude Desktop 版本，并跳转到仓库和 Release。

### 构建

需要环境：

- Rust stable
- Node.js 22 + npm
- Windows：Visual Studio Build Tools C++ 工作负载与 WebView2
- macOS：Xcode Command Line Tools

开发运行：

```bash
npm install
npx tauri dev
```

前端构建检查：

```bash
npm run build
```

Windows 发布构建：

```bat
build-release.bat
```

macOS DMG 构建示例：

```bash
npm ci
npx tauri build --bundles dmg --target aarch64-apple-darwin --no-sign
npx tauri build --bundles dmg --target x86_64-apple-darwin --no-sign
```

### 路线图

- [x] Claude Desktop 页面增强安装与卸载
- [x] Skills / MCP / 第三方 API 菜单入口
- [x] 对话标题中文化、Markdown 导出、对话时间线、token 使用信息
- [x] Claude Desktop 简体中文安装、备份和恢复
- [x] CC Switch 3P 模型名桥接与可逆接管
- [x] 诊断报告、日志查看和复制
- [x] Windows 与 macOS 安装包
- [ ] 自动更新与版本检查
- [ ] 更多 Claude Desktop 页面增强项
- [ ] 更完整的配置导出与恢复

### 技术栈

Claude++ 使用 Tauri 2 构建桌面外壳，React + TypeScript 构建控制台界面，Rust 负责本地命令、文件补丁、诊断、代理服务和 CC Switch SQLite 读取。代理侧使用 axum、reqwest、rusqlite 等库完成 HTTP 转发、模型发现和本地状态读取。

### License

[MIT](LICENSE)

> 本项目与 Anthropic、Claude、CC Switch 官方无关。请仅在你了解本地资源修改和第三方路由配置影响的前提下使用。

---

## English

### Download

Download the latest installers from [GitHub Releases](https://github.com/laoguo2025/claude-plus-plus/releases/latest).

| Platform | Installer |
| --- | --- |
| Windows x64 | `Claude++_1.0.0_x64-setup.exe` |
| macOS Apple Silicon | `Claude++_1.0.0_aarch64.dmg` |
| macOS Intel | `Claude++_1.0.0_x64.dmg` |

### What is this

Claude++ is a local enhancement console for Claude Desktop. It brings the practical desktop tweaks into one Tauri app: page enhancements, token usage display, Markdown export, Skills / MCP / third-party API shortcuts, Simplified Chinese localization, CC Switch model routing, and diagnostics.

It is not a cloud service and it does not take over your upstream accounts. Claude++ runs locally, patches Claude Desktop resources or config only when you ask it to, and in the routing flow it reads CC Switch config read-only before forwarding requests back to CC Switch.

### Features

- **Page enhancements first**: add third-party API, Skills, and MCP shortcuts to Claude Desktop, plus conversation title localization, Markdown export, conversation timeline, and token usage details.
- **Chinese localization**: install Simplified Chinese resources, keep backups, and restore English when needed.
- **CC Switch bridge**: show CC Switch custom model names in the Claude Desktop 3P model picker while keeping the actual request routing unchanged.
- **Local proxy with reversible takeover**: listens on `127.0.0.1:15722`, writes a separate Claude++ config entry, avoids overwriting the CC Switch entry, and can be disconnected at any time.
- **Environment checks**: detect Claude Code, Claude Desktop, developer mode, and CC Switch, with quick install or enable actions.
- **Diagnostics**: read local logs and generate a report covering routing, model mappings, localization, and enhancement status.
- **Cross-platform installers**: Windows installer plus macOS Apple Silicon / Intel DMGs.

### Usage

#### 1. Prepare the environment

![Welcome](docs/images/claude-plus-plus-welcome.png)

The welcome page checks Claude Code, Claude Desktop, developer mode, and CC Switch. Missing pieces can be installed or enabled from the cards.

#### 2. Connect CC Switch routing

![CCS bridge](docs/images/claude-plus-plus-route.png)

The CCS bridge page shows the CC Switch route switch, Claude++ takeover state, and current model mappings. After takeover, Claude Desktop shows CC Switch custom model names while requests still go through the original CC Switch roles and upstream configuration.

#### 3. Install or restore localization

![Localization](docs/images/claude-plus-plus-localization.png)

The localization page installs Simplified Chinese resources, supports complete or safer patch scopes, and restores English from backups when needed.

#### 4. Manage Claude Desktop page enhancements

![Enhancements](docs/images/claude-plus-plus-enhance.png)

The enhancement page installs or removes Claude Desktop tweaks: third-party API, Skills, MCP shortcuts, conversation title localization, Markdown export, conversation timeline, and token usage display. Restart Claude Desktop after installation to apply them.

#### 5. Check diagnostics and version

The diagnostics page generates reports and copies recent logs. The GitHub page shows the current Claude++ / Claude Desktop versions and links to the repository and Releases.

### Build

Prerequisites:

- Rust stable
- Node.js 22 + npm
- Windows: Visual Studio Build Tools with the C++ workload and WebView2
- macOS: Xcode Command Line Tools

Development:

```bash
npm install
npx tauri dev
```

Frontend build check:

```bash
npm run build
```

Windows release build:

```bat
build-release.bat
```

macOS DMG build:

```bash
npm ci
npx tauri build --bundles dmg --target aarch64-apple-darwin --no-sign
npx tauri build --bundles dmg --target x86_64-apple-darwin --no-sign
```

### Roadmap

- [x] Claude Desktop page enhancement install and uninstall
- [x] Skills / MCP / third-party API shortcuts
- [x] Conversation title localization, Markdown export, conversation timeline, token usage display
- [x] Claude Desktop Simplified Chinese install, backup, and restore
- [x] CC Switch 3P model-name bridge with reversible takeover
- [x] Diagnostics report, log viewer, and copy actions
- [x] Windows and macOS installers
- [ ] Auto update and version checks
- [ ] More Claude Desktop page enhancements
- [ ] Fuller config export and restore

### Tech Stack

Claude++ uses Tauri 2 for the desktop shell, React + TypeScript for the console UI, and Rust for local commands, file patching, diagnostics, proxy service, and CC Switch SQLite reads. The proxy side uses axum, reqwest, rusqlite, and related crates for HTTP forwarding, model discovery, and local state inspection.

### License

[MIT](LICENSE)

> This project is not affiliated with Anthropic, Claude, or CC Switch. Use it only when you understand the impact of local resource patches and third-party routing configuration.
