# 2026-06-02 Logo 与 Cowork 风格配色

## 原因

用户要求把 Claude++ 全部 logo 图标替换为 `C:\Users\Administrator\Desktop\bot2.png`，并参考 `C:\Users\Administrator\Desktop\Claude Cowork 设计文档.md` 优化整体配色。

## 范围

- 使用 `bot2.png` 生成并替换 `src-tauri/icons/` 下现有桌面端 PNG、ICO、ICNS、Windows Store logo。
- 侧栏品牌图标由文字 `C++` 改为同源 bot 图标。
- 配色 token 调整为 Cowork 风格暖白/炭黑层级，保留 Clay 品牌橙作为选中态与强调色。
- 新增轻量语义变量：品牌柔和底、状态底、状态边框、面板阴影。

## 非变化范围

- 不修改功能入口、路由、页面结构和 Tauri 打包配置。
- `tauri icon` 生成的移动端图标目录已清理，避免纳入当前 Windows/NSIS 范围。

## 回滚依据

如需恢复旧视觉，可回退本次提交；图标替换不影响路由、汉化、页面增强和诊断日志功能。

## 验证

- `npm run build`：通过。
- 浏览器预览：亮/暗主题 logo 加载正常，暗色层级为 `#0d0d0d / #171716 / #1f1f1e / #2c2c2a`。
- `git diff --check`：通过，仅出现 Windows 换行提示。
- `cargo test --manifest-path src-tauri\Cargo.toml --lib`：通过，36 passed，3 ignored。
- Release 打包：通过，已生成 `src-tauri\target\release\claude-plus-plus.exe` 和 `src-tauri\target\release\bundle\nsis\Claude++_0.1.0_x64-setup.exe`。
