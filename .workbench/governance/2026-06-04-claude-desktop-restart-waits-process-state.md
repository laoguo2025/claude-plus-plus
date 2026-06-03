## 背景

用户反馈 Claude Desktop 已打开时,点击 Claude++ 右上角“重启 Claude Desktop”按钮没有真正退出并重启。

## 根因

旧流程在 `taskkill /IM Claude.exe /T` 返回成功后立即调用启动入口。Windows 上 `taskkill` 返回成功不等于目标进程已经完全退出; 立刻启动可能只聚焦仍在退出中的旧实例,导致用户看到没有完成真正的退出后重启。

## 变更

- 非强制 `taskkill` 成功后继续轮询 `tasklist`,确认 `Claude.exe` 消失后才启动。
- 如果非强制关闭后进程仍存在,再执行 `/F` 强制关闭,并继续等待进程消失。
- 启动 Claude Desktop 后短暂轮询确认 `Claude.exe` 重新出现; 未检测到则返回错误。
- 启动入口仍保持原约定: 优先桌面 `Claude.lnk`,再回退 Store AppUserModelID。

## 验证

- `npm run build`
- `cmd.exe /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cargo test --manifest-path src-tauri\Cargo.toml --lib'`

## 回退

回退本次提交即可恢复旧的即时启动行为。不涉及 Claude Desktop 资源写入。
