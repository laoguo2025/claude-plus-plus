# 2026-06-05 自包含诊断报告增强

## 原因

用户要求诊断日志能够覆盖 Claude++ 在用户电脑上的常见问题，拿到报告后可以直接判断问题位置，减少二次追问。

## 范围

- 后端诊断报告升级为 schemaVersion 2，增加 summary、findings、checks、paths。
- 诊断自采集 Claude++ 设置、网关端口和 token 格式、CC Switch 数据库/路由/同 entry 网关配置、Claude Desktop configLibrary、Claude Desktop 安装和 app.asar 可读性、开发者模式候选文件、最近诊断日志摘要。
- 仅做只读采集；不写 Claude Desktop、CC Switch、ASAR 或外部配置。
- 敏感值只保留存在性、长度、格式或脱敏 URL 结构，不输出 raw API key、gateway token、Authorization、query secret 或完整敏感 JSON。
- 前端诊断页说明和浏览器预览报告同步新结构。

## 回滚依据

若报告生成或诊断页出现回归，可回退本次提交。诊断报告不参与代理转发、汉化安装、页面增强安装或外部配置写入。

## 验证

- `cmd.exe /c build.bat test --lib`：通过，131 passed，5 ignored。
- `npm run typecheck`：通过。
- `cmd.exe /c build.bat fmt --check`：返回成功；仍打印既有 `src-tauri/src/lib.rs` import 顺序建议，本轮未改该文件。
- `git diff --check`：通过，仅有 Windows 换行提示。
