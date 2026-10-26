# Headless renderer prototype

This crate is the first build boundary for the dedicated renderer. It reuses
the upstream `render.rs` protocol and chart rendering code while omitting the
Tauri application, tray, WebView and task queue. The existing full renderer
remains under `upstream/phira-render` and remains the visual reference.

The crate currently still inherits `prpr`'s audio and video dependencies. It
is therefore a dependency-isolation step, not yet the final no-audio preview
profile. The next change will make audio mixing and stateful effects explicit
in the render request before adding frame/segment output.

Run from this directory after the local Rust toolchain and native dependencies
are available:

```text
cargo check
```
