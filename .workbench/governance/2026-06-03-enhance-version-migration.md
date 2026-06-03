# 页面增强脚本版本与升级迁移

## 目标

- 7 个页面增强项从 `v0.1` 开始显式声明版本。
- Claude++ 页面增强卡片在标题旁显示版本号。
- 新版本 Claude++ 读取增强状态时，自动把已启用且版本过期的增强脚本升级到当前版本。
- 原本未启用的增强项不因升级自动启用。

## 现场结论

- 页面增强定义的唯一来源是 `src/shared/enhance-features.json`。
- 旧实现只写 `window.<marker>=true`，只能判断开关，无法判断脚本版本。
- `技能`、`对话列表中文化`、`Token使用信息` 还有 preload/main bridge，需要在已启用且过期时同步重装 bridge。

## 变更

- 增强定义增加 `version` 字段，当前 7 项均为 `v0.1`。
- Rust 状态结构增加版本字段，并将写入 payload 改为 `window.<marker>={version:"v0.1"}`。
- 状态读取兼容旧 `window.<marker>=true`，旧 payload 视为已启用但版本未知，需要迁移。
- `claude_enhance_status` 读取真实 Claude Desktop 资源时先执行版本迁移：只升级已启用且版本不一致的增强项。
- `token_usage` 页面网络捕获排除 `/claude-plus/token-usage` 自身轮询，避免把本地轮询误算为对话请求。
- 前端页面增强卡片标题旁显示版本号。

## 验证

- `npm run build` 通过。
- `cargo test claude_enhance::imp::tests --lib` 通过：35 passed, 3 ignored。
- `cargo test --lib` 通过：61 passed, 4 ignored。
- 未运行 ignored 的真实安装验证测试；这些测试会写入本机 Claude Desktop 资源并改变增强状态，不适合作为本轮“保留用户原状态”的自动验收。

## 回退

- 回退本轮本地提交即可撤销源码和 UI 变更。
- 若已通过新版 Claude++ 触发过 Claude Desktop 资源迁移，可使用 Claude++ 页面增强的取消/恢复入口，或恢复 `.claude-plus-enhance-backups` 最近备份。
