# 2026-06-03 路由状态修复留痕

## 触发原因

用户指出 CCS 转接页显示的 "CC Switch 路由 已开启" 与 CC Switch 实际总开关关闭不一致。

## 排查结论

前端当前用 `get_mappings` 是否读到当前服务商映射来展示 CC Switch 路由状态。首次修复改读 `settings.json.enableLocalProxy`,但现场再次验证发现该字段会滞后:CC Switch 已启动代理、端口可连通时,它仍可能为 `false`。真实开关源应读 CC Switch SQLite 的 `proxy_config.proxy_enabled`。

## 变更范围

- 新增真实 CC Switch 路由开关状态检测,从 `proxy_config` 读取,不写死或展示固定路由地址。
- CCS 转接页改为四个状态卡: Claude Desktop、CC Switch 路由开关、Claude++ 路由接管、模型服务商配置。
- Claude Desktop 安装状态只在 Claude++ 前端启动时检测一次。
- 明确 Claude++ 接管/断开接管按钮语义。
- 右上角刷新按钮改为 Claude++ 全局状态刷新,覆盖路由、服务商、Claude Desktop、增强和诊断页状态。

## 回退依据

若变更导致状态展示异常,还原本轮涉及的 `src-tauri/src/server.rs`、`src-tauri/src/lib.rs`、`src/App.tsx` 和 `src/App.css` 即可。Claude Desktop 配置写入逻辑不在本轮改变。
