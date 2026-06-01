# Claude Desktop 页面增强黑屏修复

## 现象
- 页面增强注入后，Claude Desktop 主窗口黑屏。
- 日志出现 `Main webview became unresponsive`。

## 根因
- 第一版注入脚本在全局 `MutationObserver` 回调里反复重写增强入口 `innerHTML`。
- 该写入会再次触发 observer，在 Claude Desktop 的 React 动态导航中可形成自触发循环，导致主 WebView 卡死。

## 修复
- 回滚现场注入后重启 Claude Desktop，黑屏解除。
- 注入脚本改为幂等防抖：
  - observer 回调合并为 250ms 定时处理。
  - 已存在且版本匹配的增强入口不再重写 DOM。
  - 只在入口缺失、顺序变化或版本变化时重建软入口。

## 菜单入口纠偏
- 旧实现问题：
  - 侧边栏入口使用自定义 `.claude-plus-enhance-nav` 样式，字体、颜色、行高和 Claude Desktop 原生导航项不一致。
  - 点击后先进入“自定义/开发者”页面再按文字查找目标，属于模拟查找，不是跳转到 Claude Desktop 原有页面。
- 现场核验：
  - Claude Desktop 前端路由包含 `/customize/plugins/new`、`/customize/connectors`、`/setup-desktop-3p`。
  - `app.asar` 主进程开发者菜单 `Configure Third-Party Inference…` 打开的窗口 URL 为 `${YeA}/setup-desktop-3p`。
- 本轮修正：
  - 入口行改为克隆同一侧边栏里的原生导航项 DOM/classes/styles，只替换图标、中文标签、目标路径和点击处理。
  - 删除自定义按钮 CSS 注入，启动时只清理旧版样式节点。
  - `第三方API` 打开 `/setup-desktop-3p`；`插件与技能` 打开 `/customize/plugins/new?marketplace&plugin`；`MCP与扩展` 打开 `/customize/connectors`。
  - v3.6 保留克隆原生“计划任务”行来继承字体、颜色、间距和 hover 状态，但先替换文本，再以目标文本为锚点清理文字前方旧图标节点，插入 Claude++ 自己的 16px 图标，避免残留加号或计划任务时钟图标。
  - 保留 250ms 防抖和版本幂等，避免重新引入 observer 自触发卡死。

## 验证
- `npm run build` 通过。
- `cargo check` 通过，仅保留既有 `server.rs` dead_code warning。
- 重新写入 3 个菜单增强后重启 Claude Desktop，35 秒内未再出现 `Main webview is unresponsive` 或 fatal boundary。
- 当前仅菜单增强已接入；Markdown 导出与时间线仍未接入真实功能。
- v3.6 写入现场 Claude Desktop 后截图确认：新增三项侧栏样式与原生菜单行对齐，未再出现 `+`，未再复制“计划任务”的时钟图标；原生“计划任务”自己的时钟图标保留。
- 逐项真实点击验证：
  - `第三方API` 通过 Claude Desktop preload 暴露的 `window["claude.settings"].Custom3pSetup.openSetupWindow()` 打开原生“配置第三方推理”独立窗口，不再在主窗口内跳转 `/setup-desktop-3p`。
  - `插件与技能` 打开 Claude Desktop 原生 `Customize` 页面，包含连接应用、创建 Skills、浏览插件。
  - `MCP与扩展` 打开 Claude Desktop 原生 `Customize` 连接器页。
- v3.6 重启及点击验证后日志未出现 `Main webview is unresponsive` 或 fatal boundary。
- v3.7 修正 `第三方API` 的打开方式后，现场点击已弹出 900x720 的 Claude 原生第三方推理配置窗口，日志未出现 `Main webview is unresponsive` 或 fatal boundary。
