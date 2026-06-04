# 2026-06-04 CC Switch 路由开关字段修复留痕

## 触发原因

用户反馈 CC Switch 路由明明未开启,Claude++ 却显示路由开关已开启。

## 根因

Claude++ 状态读取 `proxy_config.proxy_enabled` 作为路由开关。现场 CC Switch 当前表结构里同时存在 `proxy_enabled` 和 `enabled`; 当 UI 总开关关闭时,`enabled=0`,但 `proxy_enabled` 仍可能为 `1`,导致 Claude++ 误判。

## 变更

- `load_proxy_config` 在当前 schema 存在 `enabled` 列时优先读取 `enabled`。
- 旧版 schema 没有 `enabled` 列时继续读取 `proxy_enabled`,保持兼容。
- 补回归测试覆盖 `proxy_enabled=1, enabled=0` 应显示关闭,以及旧 schema 回退开启。
- 更新项目地图,明确 `enabled` 是当前 UI 总开关字段。

## 验证

- 先跑新增定向测试,确认当前实现会失败。
- 修复后跑定向测试、Rust 库测试、前端构建和 diff 检查。

## 回退

如状态读取出现兼容性问题,回退本次提交即可。该变更只读 CC Switch 数据库,不写入 CC Switch、Claude Desktop 或用户配置。
