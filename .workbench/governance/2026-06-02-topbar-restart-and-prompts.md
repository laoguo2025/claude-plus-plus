# 2026-06-02 顶部重启按钮与页面提示调整

## 原因

用户确认顶部重启按钮应重启 Claude Desktop，而不是重启 Claude++；同时要求移除一键汉化与页面增强页内重复的重启卡片，改为在目标卡片上方显示醒目提示。

## 范围

- 顶部操作区的重启按钮改为调用 Claude Desktop 重启命令，并删除 Claude++ 自重启命令入口。
- CCS 转接说明文字增大行距。
- 当前服务商与模型映射表按 CC Switch 原始顺序展示，列改为 CCS 模型角色、Claude 模型显示名、实际请求模型。
- 一键汉化页在“检测汉化程度”上方显示生效提示，并删除页内重启卡片。
- 页面增强页在“第三方API”上方显示生效提示，并删除页内重启卡片。

## 回滚依据

若顶部按钮语义或页面提示不符合需求，可回退 `src/App.tsx`、`src/App.css`、`src-tauri/src/lib.rs` 本次提交；此前 CCS 转接页第 4 步仍保留 Claude Desktop 重启入口，不依赖被删除的 Claude++ 自重启命令。

## 验证

- `npm run build`：通过。
- `cargo test --manifest-path src-tauri\Cargo.toml --lib`：通过，36 passed，3 ignored。
- `git diff --check`：通过，仅出现 Windows 换行提示。
- `build-release.bat`：通过，生成 release exe 与 NSIS 安装包。
