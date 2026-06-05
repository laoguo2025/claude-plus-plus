# Welcome Claude Desktop status copy

## Scope

用户要求欢迎页 `Claude Desktop` 检测卡片状态恢复为安装语义。

## Change

- 将欢迎页 `Claude Desktop` 卡片的显示文案从 `已定位` / `未定位` 改为 `已安装` / `未安装`。
- 内部检测逻辑保持不变,仍使用 `welcome_status.claude_desktop_found` 和共享 Claude Desktop 安装发现链路。

## Verification

- `npm run typecheck`: passed。

## Rollback

回退本次提交即可恢复旧显示文案。该变更不写 Claude Desktop、CC Switch 或外部配置。
