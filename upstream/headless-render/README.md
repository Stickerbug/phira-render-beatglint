# Headless renderer prototype

This crate is the first build boundary for the dedicated renderer. It reuses
the upstream `render.rs` protocol and chart rendering code while omitting the
Tauri application, tray, WebView and task queue. The existing full renderer
remains under `upstream/phira-render` and remains the visual reference.

The crate now uses the `prpr` headless feature: it avoids Tauri, file dialogs,
and live CPAL/ALSA audio while retaining decoded clips and chart timing. It is
still a dependency-isolation step, not a complete bounded renderer. The next
change will make audio mixing and stateful effects explicit in the render
request before adding frame/segment output.

Run from this directory after the local Rust toolchain and native dependencies
are available:

```text
cargo check
```
