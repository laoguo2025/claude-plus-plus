# ccs2claude

[中文](#中文) · [English](#english)

一个本地中间件，让 **Claude Desktop (3P / 第三方推理模式)** 的模型选择器显示 **CC Switch** 中自定义的模型名称，同时保持真实路由不变。

A local middleware that makes the **Claude Desktop (3P / third-party inference)** model picker show the custom model names configured in **CC Switch**, while keeping the real routing intact.

---

## 中文

### 这是什么

[CC Switch](https://github.com/farion1231/cc-switch) 可以把 Claude Desktop / Code / Codex 的请求路由到不同的后端服务商。它支持给模型设置自定义显示名（`labelOverride`），但 **Claude Desktop 3P 的配置 schema 会静默丢弃 `labelOverride` 字段**，导致选择器里显示的还是内置的官方名称，而不是你设置的自定义名。

`ccs2claude` 通过一个本地代理解决这个问题：它在 CC Switch 和 Claude Desktop 之间充当中间层，利用 Claude Desktop 的「模型发现模式」(`/v1/models`) 来呈现自定义名称，并在请求转发时把自定义名还原成真实的角色 ID。

### 工作原理

```
Claude Desktop ──► ccs2claude (:15722) ──► CC Switch (:15721) ──► 上游 LLM
                      │
                      └─ 读取 CC Switch 的 SQLite 配置（只读），实时同步映射
```

1. **读取映射**：以只读方式读取 CC Switch 的数据库（`~/.cc-switch/cc-switch.db`），取出当前 claude-desktop 服务商的模型路由（显示名 ↔ 角色 ID）。增 / 改 / 删 / 切换服务商都会自动同步，无需任何硬编码。
2. **模型列表**：`GET /v1/models` 用自定义显示名生成模型列表（`id` 与 `display_name` 都是自定义名，且不带 `supports1m`，避免出现 1M 重复变体）。Claude Desktop 在「发现模式」下会直接展示这些名称。
3. **请求改写**：转发 `/v1/messages` 等请求时，把 body 里的 `model`（自定义显示名）改写回真实的角色 ID，再转发给 CC Switch；SSE 流式响应逐块透传。
4. **配置接入**：在 Claude Desktop 的配置库里新建一个独立的 `ccs2claude` 条目（指向 `:15722`，不写 `inferenceModels` 以强制走发现模式），不修改 CC Switch 自己写的条目，两者共存、可一键切换。

### 为什么不直接改源码 / 数据库

- 不改 CC Switch 的数据库：切换服务商时 CC Switch 会重写配置，硬编码会被覆盖。
- 不改 Claude Desktop：升级会被覆盖，且无法自动同步。
- 中间件方案：实时读取、零硬编码、自动同步、随时可撤销。

### 构建与运行

需要环境：

- [Rust](https://www.rust-lang.org/)（稳定版）
- [Node.js](https://nodejs.org/) + npm
- Windows：Visual Studio Build Tools（含 C++ 工作负载）与 WebView2
- 已安装并配置好的 [CC Switch](https://github.com/farion1231/cc-switch)，且当前服务商为 claude-desktop 类型

开发运行：

```bash
npm install
npx tauri dev
```

构建（Windows 下推荐用附带的 `build.bat`，它会自动加载 MSVC 环境，避免 MSYS 的 `link.exe` 冲突）：

```bash
# 直接构建
build.bat build
# 或发布版
build.bat build --release
```

### 使用

1. 先启动 CC Switch，并选好 claude-desktop 类型的服务商。
2. 运行 ccs2claude，代理会自动监听 `127.0.0.1:15722`。
3. 在界面里点击「接入 Claude Desktop」。
4. 重启 Claude Desktop，打开模型选择器——此时应显示你在 CC Switch 里设置的自定义模型名。
5. 想恢复时点「撤销接入」，切回 CC Switch 原本的配置条目。

### 路线图

- [x] 模型配置桥接（v1）
- [ ] 一键下载 Claude Desktop
- [ ] 一键汉化

### 技术栈

Tauri 2 + React + TypeScript（前端）/ Rust + axum + rusqlite + reqwest（后端代理）。

### License

[MIT](LICENSE)

> 本项目与 Anthropic、Claude、CC Switch 官方无关，仅为方便个人使用而开发。

---

## English

### What is this

[CC Switch](https://github.com/farion1231/cc-switch) routes Claude Desktop / Code / Codex requests to different backend providers. It lets you set a custom display name per model (`labelOverride`), but **Claude Desktop 3P silently drops the `labelOverride` field** from its config schema, so the picker keeps showing the built-in official names instead of your custom ones.

`ccs2claude` fixes this with a local proxy that sits between CC Switch and Claude Desktop. It leverages Claude Desktop's model **discovery mode** (`/v1/models`) to present the custom names, then rewrites them back to the real role IDs when forwarding requests.

### How it works

```
Claude Desktop ──► ccs2claude (:15722) ──► CC Switch (:15721) ──► upstream LLM
                      │
                      └─ reads CC Switch's SQLite config (read-only), live-synced
```

1. **Read mappings** — reads CC Switch's database read-only (`~/.cc-switch/cc-switch.db`) to get the current claude-desktop provider's model routes (display name ↔ role ID). Add / edit / delete / provider-switch all sync automatically, with zero hardcoding.
2. **Model list** — `GET /v1/models` returns models built from the custom display names (`id` and `display_name` both set to the custom name, no `supports1m` to avoid 1M duplicate variants). In discovery mode Claude Desktop shows these names directly.
3. **Request rewrite** — when forwarding `/v1/messages` and friends, the `model` field (custom display name) is rewritten back to the real role ID before reaching CC Switch; SSE streaming responses are passed through chunk by chunk.
4. **Config apply** — creates a separate `ccs2claude` entry in Claude Desktop's config library (pointing at `:15722`, with no `inferenceModels` to force discovery mode) without touching CC Switch's own entry. Both coexist and can be toggled with one click.

### Why not patch the source / database

- Don't edit CC Switch's DB: it rewrites the config on provider switch, so hardcoded changes get overwritten.
- Don't patch Claude Desktop: updates would overwrite it, and it can't auto-sync.
- Middleware: live reads, zero hardcoding, auto-sync, reversible at any time.

### Build & run

Prerequisites:

- [Rust](https://www.rust-lang.org/) (stable)
- [Node.js](https://nodejs.org/) + npm
- Windows: Visual Studio Build Tools (with the C++ workload) and WebView2
- [CC Switch](https://github.com/farion1231/cc-switch) installed and configured, with the current provider of type claude-desktop

Dev:

```bash
npm install
npx tauri dev
```

Build (on Windows, the bundled `build.bat` auto-loads the MSVC environment to avoid the MSYS `link.exe` conflict):

```bash
build.bat build
# or release
build.bat build --release
```

### Usage

1. Start CC Switch and select a claude-desktop type provider.
2. Run ccs2claude; the proxy auto-listens on `127.0.0.1:15722`.
3. Click "Apply to Claude Desktop" in the UI.
4. Restart Claude Desktop and open the model picker — it should now show the custom model names you set in CC Switch.
5. Click "Revert" to switch back to CC Switch's original config entry.

### Roadmap

- [x] Model config bridging (v1)
- [ ] One-click Claude Desktop download
- [ ] One-click Chinese localization

### Tech stack

Tauri 2 + React + TypeScript (frontend) / Rust + axum + rusqlite + reqwest (backend proxy).

### License

[MIT](LICENSE)

> This project is not affiliated with Anthropic, Claude, or CC Switch. It's a personal convenience tool.
