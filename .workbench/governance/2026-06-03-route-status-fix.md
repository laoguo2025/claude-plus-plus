# 2026-06-03 路由状态修复留痕

## 触发原因

用户指出 CCS 转接页显示的 "CC Switch 路由 已开启" 与 CC Switch 实际总开关关闭不一致。

## 排查结论

前端当前用 `get_mappings` 是否读到当前服务商映射来展示 CC Switch 路由状态。现场只读验证显示,CC Switch `settings.json` 中 `enableLocalProxy` 为 `false`,但界面仍因映射存在显示已开启。

## 变更范围

- 新增真实 CC Switch 路由开关状态检测,不写死或展示固定路由地址。
- CCS 转接页改为四个状态卡: Claude Desktop、CC Switch 路由开关、Claude++ 路由接管、模型服务商配置。
- Claude Desktop 安装状态只在 Claude++ 前端启动时检测一次。
- 明确 Claude++ 接管/断开接管按钮语义。

## 回退依据

若变更导致状态展示异常,还原本轮涉及的 `src-tauri/src/server.rs`、`src-tauri/src/lib.rs`、`src/App.tsx` 和 `src/App.css` 即可。Claude Desktop 配置写入逻辑不在本轮改变。
