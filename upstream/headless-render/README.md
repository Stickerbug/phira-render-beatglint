中文说明：[`README.zh-CN.md`](README.zh-CN.md)。

# Headless renderer prototype

This crate is the first build boundary for the dedicated renderer. It reuses
the saved upstream `render.rs` protocol slice and chart rendering code while
omitting the Tauri application, tray, WebView and task queue. The full Phira
AppImage remains the visual reference; this public tree does not ship the
private full application or its runtime assets.

The crate now uses the `prpr` headless feature: it avoids Tauri, file dialogs,
and live CPAL/ALSA audio while retaining decoded clips and chart timing. It is
still a dependency-isolation step, not a complete bounded renderer. Video
decoding is also omitted in this profile; full-faithful video remains on the
AppImage reference path. The next change will make audio mixing and stateful
effects explicit in the render request before adding frame/segment output.

Run from this directory after the local Rust toolchain and native dependencies
are available:

```text
cargo check
cargo test
```
