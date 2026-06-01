# Claude Desktop 页面增强菜单文案瘦身

## 变更
- 页面增强项 `插件与技能` 改为 `技能`。
- 页面增强项 `MCP与扩展` 改为 `MCP`。
- 同步更新管理页说明和注入到 Claude Desktop 的侧边栏标签。

## 验证
- `rustfmt --check src-tauri/src/claude_enhance.rs` 通过。
- `npm run build` 通过。
- 相关源码中已无旧文案 `插件与技能`、`MCP与扩展`。

## 回退
- 回退本轮提交即可恢复旧文案。
