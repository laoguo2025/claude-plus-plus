# 页面增强 bridge 安装速度优化

## 目标

用户反馈页面增强点击“增强”时等待数秒，尤其是 `技能`、`对话列表中文化`、`Token使用信息` 三项。

## 根因

- 这三项不仅写前端 marker，还要写 Claude Desktop `app.asar` 内的 main/preload bridge。
- 旧链路对 main 和 preload 分别执行一次 `app.asar` 读取、header 解析、备份、内容替换、完整性同步和写回。
- 因此一次增强会重复两次重型 ASAR 写入。

## 变更

- 新增批量 bridge patch 路径：一次读取 `app.asar`，一次解析 header，一次备份，一次写回，一次同步 Claude.exe 内嵌完整性。
- `技能`、`对话列表中文化`、`Token使用信息` 的 main/preload bridge 写入统一走批量路径。
- no-op 情况下不写回 `app.asar`，避免重复点击当前版本增强时做无效写入。
- 删除旧的单文件 bridge ASAR patch 路径，避免后续误用。

## 验证

- `npm run build` 通过。
- `cargo test claude_enhance::imp::tests --lib` 通过：36 passed, 3 ignored。
- `cargo test --lib` 通过：62 passed, 4 ignored。
- `rustfmt --check src\claude_enhance.rs` 通过。

## 回退

回退本轮本地提交即可恢复旧的逐文件 ASAR 写入路径。若已写入 Claude Desktop 资源，可用页面增强取消/恢复入口或 `.claude-plus-enhance-backups` 最近备份回退。
