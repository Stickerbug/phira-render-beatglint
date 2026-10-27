# Headless 渲染器（中文）

这是 BeatGlint 专用渲染器的无 Tauri 构建边界。它复用 `prpr` 的谱面、判定线和几何渲染代码，移除 Tauri、文件对话框、实时 CPAL/ALSA 音频和视频解码。

当前支持单帧 PNG、bounded segment、replay/state/audio 显式协议和可选 source-music-only 音频。它从时间零开始固定帧回放，不支持随机 seek、checkpoint 或 resident worker；完整 Phira/AppImage 仍是视觉参考。

## 构建

在仓库根目录执行：

```bash
cargo check --manifest-path upstream/headless-render/Cargo.toml
cargo test --manifest-path upstream/headless-render/Cargo.toml
cargo build --release --manifest-path upstream/headless-render/Cargo.toml
```

中文公开发布和运行方法见仓库根目录的 `docs/使用说明-zh.md`。

English version: [`README.md`](README.md)。
