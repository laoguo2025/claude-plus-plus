# 2026-06-02 诊断日志页双栏增强

## 原因

用户要求借鉴 Codex++ 的“诊断”和“日志”页面，让 Claude++ 的“诊断日志”页左侧展示诊断报告，右侧展示日志列表。

参考来源：

- `BigPizzaV3/CodexPlusPlus`
- `apps/codex-plus-manager/src/App.tsx` 中 `DiagnosticsPanel` 与 `LogsPanel`
- `crates/codex-plus-core/src/diagnostic_log.rs` 中 JSONL 诊断日志方式

## 范围

- 新增 Claude++ 本地诊断日志文件：`%USERPROFILE%\.claude-plus-plus\claude-plus-plus.log`。
- 后端新增最近日志读取和诊断报告生成命令。
- 管理页“诊断日志”改为左右双栏：左侧可复制 JSON 诊断报告，右侧最近日志行号列表。
- 关键管理动作写入轻量 JSONL 诊断事件。

## 回滚依据

若新页面或日志写入引入问题，可回退本次提交；业务路由、汉化、页面增强能力不依赖诊断日志文件。

## 验证

- `npm run build`：通过。
- `cargo test --manifest-path src-tauri\Cargo.toml --lib`：通过，36 passed，3 ignored。
- `git diff --check`：通过，仅出现 Windows 换行提示。
- 页面预览：通过，诊断页左右双栏、报告、日志、复制/刷新按钮可见；页面无横向溢出，双栏填满内容区。
- Release 打包：通过，生成 release exe 与 NSIS 安装包。
