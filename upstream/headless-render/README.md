中文说明：[`README.zh-CN.md`](README.zh-CN.md)。

# Headless renderer

This crate is the first build boundary for the dedicated renderer. It reuses
the saved upstream `render.rs` protocol slice and chart rendering code while
omitting the Tauri application, tray, WebView and task queue. The full Phira
AppImage remains the visual reference; this public tree does not ship the
private full application or its runtime assets.

The crate uses the `prpr` headless feature: it avoids Tauri, file dialogs, and
live CPAL/ALSA audio while retaining decoded clips and chart timing. It
supports frame, bounded segment, and ordered `residentWindows` output. Video
decoding is omitted in this profile; full-faithful video remains on the
AppImage reference path. The headless IO waker fix is documented in
[`docs/异步唤醒修复-20261030.md`](../../docs/异步唤醒修复-20261030.md).

Run from this directory after the local Rust toolchain and native dependencies
are available:

```text
cargo check
cargo test
```
