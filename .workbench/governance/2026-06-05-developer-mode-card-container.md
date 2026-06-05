# Developer mode card container

## Scope

用户要求 `一键开发/汉化` 页的 `开发者模式` 区域与下方汉化功能区保持同一层级样式: 外层白色大容器,内部灰色小卡片。

## Change

- 将开发者模式区域结构调整为 `panel -> workflowRows -> workflowRow`,与汉化功能区一致。
- 移除开发者模式专用 CSS 覆盖,保留通用 `workflowRow` 灰色卡片样式。
- 未修改开发者模式检测逻辑、开启命令或汉化功能逻辑。

## Verification

- `npm run typecheck`: passed。

## Rollback

回退本次提交即可恢复当前开发者模式卡片 UI。该变更不写 Claude Desktop、CC Switch 或外部配置。
