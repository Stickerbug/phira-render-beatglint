# BeatGlint Phira 优化渲染器

这是一个面向 BeatGlint 的 Phira 渲染器优化版，采用 GPL-3.0-only 发布。项目保留真实 Phira 的谱面、判定线和几何渲染路径，并增加 Tauri-free 的 headless 单帧与短片段输出。当前 `main` 已将 headless 文件加载从 Tokio worker wake 改为逐帧轮询安全的线程 future。

当前支持：

- 单帧 PNG 或 MP4；
- 按视频帧网格输出 bounded segment；
- 显式 replay/state/audio 模式；
- 可选的 source-music-only segment/resident 音频；
- 分块 full 音频混音和可复现的 manifest 校验；
- `release-fast` 性能对照构建。
- 推荐的 `residentWindows` 顺序窗口输出，可在一次场景回放中生成多个 bounded 窗口（可选源音乐）；`tools/render_resident_windows.py` 提供可重复的调用入口。

性能样本和 resident 窗口收益见 [`docs/性能对比-20261027.md`](docs/性能对比-20261027.md)。

当前 headless profile 从时间零开始固定帧回放；`residentWindows` 只复用同一时间轴做有序窗口吞吐优化，不提供随机状态跳转或 checkpoint。它要求 `renderMode=full`、`stateMode=replay`，`audioMode` 可为 `none` 或 `music-only`；后者为每个窗口附加源音乐，不包含判定音效和 ending 音频。视频背景、完整粒子/命中特效和 full-faithful 视觉验收仍应使用完整 Phira/AppImage 参考路径。这个仓库不包含谱面、音频、曲绘、皮肤、AppImage、模型、迁移包或渲染视频；请自行准备有权使用的输入。

## 快速开始

环境要求：Linux、Rust stable、FFmpeg、可用的 Xorg/Mesa 离屏显示，以及合法的 Phira 资源目录。

```bash
git clone https://github.com/Stickerbug/phira-render-beatglint.git
cd phira-render-beatglint

git checkout main

export RUSTUP_HOME="$PWD/.rustup"
export CARGO_HOME="$PWD/.cargo"
export PATH="$CARGO_HOME/bin:$PATH"

cargo test --manifest-path upstream/headless-render/Cargo.toml
cargo build --release --manifest-path upstream/headless-render/Cargo.toml
```

完整中文说明见 [`docs/使用说明-zh.md`](docs/使用说明-zh.md)；异步唤醒修复和验证记录见 [`docs/异步唤醒修复-20261030.md`](docs/异步唤醒修复-20261030.md)；第三方许可证和版权边界见 [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)。

## 许可证

BeatGlint 新增和修改的代码按 GPL-3.0-only 发布，完整文本见 [`LICENSE`](LICENSE)。仓库中的 Phira、原始 phira-render、锁定的 macroquad 依赖和其他依赖仍保留各自的上游许可证与版权声明，详见 [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) 和 [`LICENSES/`](LICENSES/)。
