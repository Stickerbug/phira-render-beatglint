# BeatGlint Phira 优化渲染器

这是一个面向 BeatGlint 的 Phira 渲染器优化版，采用 GPL-3.0-only 发布。项目保留真实 Phira 的谱面、判定线和几何渲染路径，并增加 Tauri-free 的 headless 单帧与短片段输出。

当前支持：

- 单帧 PNG 或 MP4；
- 按视频帧网格输出 bounded segment；
- 显式 replay/state/audio 模式；
- 可选的 source-music-only segment 音频；
- 分块 full 音频混音和可复现的 manifest 校验；
- `release-fast` 性能对照构建。

当前 headless profile 从时间零开始固定帧回放，尚未实现随机状态跳转、checkpoint 或 resident worker。视频背景、完整粒子/命中特效和 full-faithful 视觉验收仍应使用完整 Phira/AppImage 参考路径。这个仓库不包含谱面、音频、曲绘、皮肤、AppImage、模型、迁移包或渲染视频；请自行准备有权使用的输入。

## 快速开始

环境要求：Linux、Rust stable、FFmpeg、可用的 Xorg/Mesa 离屏显示，以及合法的 Phira 资源目录。

```bash
git clone https://github.com/Stickerbug/phira-render-beatglint.git
cd phira-render-beatglint

git checkout public/gpl-release-20261027

export RUSTUP_HOME="$PWD/.rustup"
export CARGO_HOME="$PWD/.cargo"
export PATH="$CARGO_HOME/bin:$PATH"

cargo test --manifest-path upstream/headless-render/Cargo.toml
cargo build --release --manifest-path upstream/headless-render/Cargo.toml
```

完整中文说明见 [`docs/使用说明-zh.md`](docs/使用说明-zh.md)；公开发布和许可证检查见 [`PUBLIC-RELEASE-CHECKLIST.md`](PUBLIC-RELEASE-CHECKLIST.md)。

## 许可证

BeatGlint 新增和修改的代码按 GPL-3.0-only 发布，完整文本见 [`LICENSE`](LICENSE)。仓库中的 Phira、原始 phira-render、macroquad 和其他依赖仍保留各自的上游许可证与版权声明，详见 [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) 和 [`LICENSES/`](LICENSES/)。
