# CCS route transfer copy

## Scope

用户要求左侧菜单 `CCS转接` 改为 `CCS路由转接`。

## Change

- 将 `overview` 路由的菜单 label 与页面标题统一改为 `CCS路由转接`。
- 未修改路由 id、转接逻辑、状态检测或后端命令。

## Verification

- `npm run typecheck`: passed。
- `.\build-release.bat`: passed。
- release 产物已复制到 `release/claude-plus-plus.exe` 与 `release/Claude++_1.0.0_x64-setup.exe`。

## Rollback

回退本次提交即可恢复原文案。该变更不写 Claude Desktop、CC Switch 或外部配置。
