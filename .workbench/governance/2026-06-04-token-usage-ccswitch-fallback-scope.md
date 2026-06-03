## 背景

用户反馈 Token 使用信息卡片在打开历史对话时自动统计,调用次数和缓存命中率持续刷新且明显错误。

## 根因

CC Switch 的 `proxy_request_logs` 只有秒级 `created_at`,且同表包含 Claude Desktop 历史加载、状态、标题和其他后台请求。Claude++ 之前按 `sinceMs` 聚合所有成功 `claude-desktop` 行,会把这些后台请求累加到当前卡片,造成调用次数和 token 数膨胀。

前端缓存命中率此前用 `cache_read / input` 计算。CC Switch 记录里的缓存读可能远大于本轮新输入,所以会显示超过 100% 的比例。

## 变更

- CC Switch DB 回退只读取 `sinceMs` 之后最新一条成功代理日志,不再按时间段求和。
- `/claude-plus/token-usage` 不再把 CC Switch DB 回退写回全局 token usage 状态,避免跨轮污染。
- 注入脚本保留 `source` 字段,将 `cc-switch/proxy` 视为弱来源:页面实际捕获到 usage 后会清掉弱来源;卡片最终渲染后忽略弱来源轮询刷新。
- 缓存命中率改为 `cache_read / (input + cache_read + cache_write)`,并封顶到 100%。

## 验证

- `npm run build`
- `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo test --manifest-path src-tauri\Cargo.toml --lib'`

## 回退

回退本次提交即可恢复旧行为。若仅需关闭用户可见影响,可在 Claude++ 页面增强中卸载 Token 使用信息增强,或重新安装上一版本后重启 Claude Desktop。
