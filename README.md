# Claude++

<p align="center">
  <img src="src-tauri/icons/icon.png" width="96" alt="Claude++ icon" />
</p>

<p align="center">
  <strong>让 Claude Desktop 3P 模型选择器显示 CC Switch 自定义模型名的本地桌面工具。</strong>
</p>

<p align="center">
  <a href="https://github.com/laoguo2025/claude-plus-plus/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/laoguo2025/claude-plus-plus?label=release"></a>
  <a href="https://github.com/laoguo2025/claude-plus-plus/actions/workflows/macos-dmg.yml"><img alt="macOS DMG workflow" src="https://github.com/laoguo2025/claude-plus-plus/actions/workflows/macos-dmg.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="MIT License" src="https://img.shields.io/badge/license-MIT-green.svg"></a>
  <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24C8DB.svg">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-backend-B7410E.svg">
  <img alt="React" src="https://img.shields.io/badge/React-frontend-61DAFB.svg">
</p>

[中文](#中文) · [English](#english)

---

## 中文

### 快速下载

最新安装包请到 [GitHub Releases](https://github.com/laoguo2025/claude-plus-plus/releases/latest) 下载。

| 平台 | 安装包 |
| --- | --- |
| Windows x64 | `Claude++_1.0.0_x64-setup.exe` |
| macOS Apple Silicon | `Claude++_*_aarch64.dmg` / `Claude++-*-arm64.dmg` |
| macOS Intel | `Claude++_*_x64.dmg` / `Claude++-*-x64.dmg` |

> macOS DMG 由 GitHub Actions 构建并上传到 Release；如遇 macOS 安全提示，请在系统设置中允许来自该开发者的应用运行。

### 这是什么

[CC Switch](https://github.com/farion1231/cc-switch) 可以把 Claude Desktop / Code / Codex 的请求路由到不同后端服务商，并支持给模型设置自定义显示名（`labelOverride`）。但 Claude Desktop 3P 的配置 schema 会静默丢弃 `labelOverride` 字段，导致模型选择器仍显示内置官方名称。

`Claude++` 在 Claude Desktop 和 CC Switch 之间增加一个本地代理：Claude Desktop 通过 `/v1/models` 发现自定义模型名，请求发送时再由 Claude++ 把自定义名还原成 CC Switch 能识别的真实角色 ID。

```text
Claude Desktop -> Claude++ (127.0.0.1:15722) -> CC Switch (127.0.0.1:15721) -> 上游 LLM
                         |
                         +-- 只读读取 CC Switch SQLite 配置，实时同步模型映射
```

### 核心功能

- **模型名桥接**：读取 CC Switch 当前 claude-desktop 服务商映射，让 Claude Desktop 显示自定义模型名。
- **独立配置接入**：写入单独的 `Claude++` configLibrary 条目，不覆盖 CC Switch 原始条目，可一键接管或回退。
- **本地代理转发**：默认监听 `127.0.0.1:15722`，请求透传到 CC Switch，流式响应逐块转发。
- **一键汉化**：为 Claude Desktop 安装或恢复简体中文资源，并保留备份回退路径。
- **页面增强**：支持标题汉化、token 用量卡片、Markdown 导出、会话时间线、Skills 弹窗等 Claude Desktop 页面增强。
- **诊断日志**：生成本机状态、代理状态、路由状态和增强状态的诊断报告，便于排查。
- **跨平台打包**：Windows NSIS 安装包、macOS Apple Silicon DMG、macOS Intel DMG。

### 使用方式

1. 安装并启动 [CC Switch](https://github.com/farion1231/cc-switch)，选择 `claude-desktop` 类型服务商。
2. 安装并运行 Claude++，确认本地代理处于运行状态。
3. 在 Claude++ 中点击「接管」或「接入 Claude Desktop」。
4. 重启 Claude Desktop，打开模型选择器检查自定义模型名。
5. 需要恢复时，在 Claude++ 中点击「回退」切回 CC Switch 原始配置。

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

前端检查：

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

### Releases

仓库包含两类发布相关工作流：

- `Build macOS DMG`：手动或 tag push 触发，用于验证 macOS arm64/x64 DMG artifact。
- `Release Installers`：GitHub Release 发布时触发，构建并上传 Windows NSIS 与 macOS DMG 到对应 Release。

### 路线图

- [x] Claude Desktop 3P 模型发现桥接
- [x] 独立 Claude++ configLibrary 接入与回退
- [x] CC Switch 路由状态检测
- [x] Claude Desktop 简体中文安装与恢复
- [x] Claude Desktop 页面增强
- [x] 诊断日志与报告
- [x] Windows NSIS 安装包
- [x] macOS arm64/x64 DMG 自动构建
- [ ] 自动更新与版本检查

### 技术栈

Tauri 2 + React + TypeScript / Rust + axum + rusqlite + reqwest。

### License

[MIT](LICENSE)

> 本项目与 Anthropic、Claude、CC Switch 官方无关，仅为方便个人使用而开发。

---

## English

### Download

Download the latest installers from [GitHub Releases](https://github.com/laoguo2025/claude-plus-plus/releases/latest).

| Platform | Installer |
| --- | --- |
| Windows x64 | `Claude++_1.0.0_x64-setup.exe` |
| macOS Apple Silicon | `Claude++_*_aarch64.dmg` / `Claude++-*-arm64.dmg` |
| macOS Intel | `Claude++_*_x64.dmg` / `Claude++-*-x64.dmg` |

### What is this

[CC Switch](https://github.com/farion1231/cc-switch) routes Claude Desktop / Code / Codex requests to different backend providers and supports custom model display names (`labelOverride`). Claude Desktop 3P silently drops that field from its config schema, so the model picker still shows the built-in names.

`Claude++` fixes this with a local proxy between Claude Desktop and CC Switch. Claude Desktop discovers custom names through `/v1/models`; Claude++ rewrites those names back to the real CC Switch role IDs when forwarding requests.

### Features

- Custom model-name discovery for Claude Desktop 3P.
- Reversible `Claude++` configLibrary entry without overwriting the CC Switch entry.
- Local proxy on `127.0.0.1:15722` with streaming pass-through.
- Simplified Chinese localization install and restore for Claude Desktop.
- Page enhancements including title i18n, token usage, Markdown export, conversation timeline, and Skills popup.
- Diagnostics report and local logs.
- Windows NSIS installer and macOS arm64/x64 DMG packaging.

### Build

```bash
npm install
npx tauri dev
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

### License

[MIT](LICENSE)

> This project is not affiliated with Anthropic, Claude, or CC Switch. It is a personal convenience tool.
