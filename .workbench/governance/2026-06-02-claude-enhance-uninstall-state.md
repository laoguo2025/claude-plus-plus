# Claude Desktop 页面增强取消状态修复

## 现象
- 页面增强页点击“取消”后会重新弹出 Claude Desktop。
- 取消后页面仍把对应增强项显示为已启用，“增强”按钮未恢复可点状态。

## 根因
- 页面增强安装与取消命令在写入资源后无条件调用 Claude Desktop 启动逻辑。
- 状态判断只查找 marker 字符串；增强脚本本体也包含 marker，导致 payload 已移除后仍被误判为已启用。

## 修复
- 页面增强安装与取消只负责停止 Claude Desktop、写入资源和备份，不再自动启动 Claude Desktop。
- 启用状态只认 `;window.<marker>=true;` payload，不再把脚本内 marker 常量当作启用。
- 增加单元测试覆盖“脚本存在但 payload 不存在时未启用”和“payload 控制启用状态”。

## 验证
- `npm run build` 通过。
- `rustfmt --check src-tauri/src/claude_enhance.rs` 通过。
- `cargo test claude_enhance::imp::tests --lib` 通过；仅剩既有 `server.rs` dead_code warning。
- `cargo check` 通过；仅剩既有 `server.rs` dead_code warning。
- 复查仅剩一个既有 release 版 Claude++ 进程，已关闭本轮临时启动的 debug 实例。

## 回退
- 回退本轮提交即可恢复旧行为。
