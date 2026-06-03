# Claude++ 竖屏宣传视频

## 目标

按用户确认的“深色科技 + Claude 橙色”风格，制作 9:16、20 秒、带背景音乐的 Claude++ 宣传视频。

## 变更

- 新增 HyperFrames 视频工程：`docs/videos/claude-plus-plus-promo/`。
- 复用现有 Claude++ 应用截图与图标，生成本地合成背景音乐。
- 输出成片：`docs/videos/claude-plus-plus-promo/claude-plus-plus-promo.mp4`。

## 验证

- `npm run check`：0 error，0 layout issues；仅存在重复使用同一 logo 的媒体发现警告，不影响渲染。
- `npm run render -- --output claude-plus-plus-promo.mp4 --quality standard --fps 30 --workers 2`：成功生成 MP4。
- `ffprobe`：新版视频 1080x1920、30.000000 秒；音频 30.016000 秒。
- `volumedetect`：新版音频 mean_volume 约 -16.5 dB，max_volume 约 -5.9 dB。
- 抽取 2 秒、8 秒、28.5 秒帧和 6 段 contact sheet，首屏、Claude++ 主画面、Claude Desktop 局部浮层和结尾画面均正常显示。

## 回退

删除 `docs/videos/claude-plus-plus-promo/` 并回退本次提交即可移除视频工程和成片，不影响应用运行代码。
