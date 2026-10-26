# Build recovery notes

## 2026-10-26

The restored upstream renderer manifest still declares:

```toml
prpr = { git = "https://github.com/TeamFlos/phira", package = "prpr", branch = "render-lib" }
```

That branch no longer exists. A first `cargo check --locked` therefore fails
before compiling any renderer code with `failed to find branch `render-lib``.

For a local baseline attempt, a temporary copy of the renderer manifest was
made outside this repository with `prpr` pointed at the verified local
`upstream/phira/prpr` source. Rust 1.99.1 was installed in the isolated paths
`/workspace/phira-render-rustup` and `/workspace/phira-render-cargo`; no main
project Python environment was changed. Cargo metadata resolved successfully
and compilation reached native dependencies.

The temporary build then stopped at `glib-sys 0.15.10` because the execution
environment has no `glib-2.0.pc` or GTK development headers. No renderer
binary was produced and no AppImage was modified.

The next buildable path should be a headless renderer profile that removes
Tauri UI, file-dialog and live audio dependencies from the build graph while
keeping the existing full-faithful AppImage as the visual reference. Any such
profile must first pass the unmodified smoke parity check before adding frame
or segment optimizations.

That first dependency-isolation step is now present under
`upstream/headless-render`: it disables `rfd` and live `sasa/cpal` through a
`prpr` headless feature, supplies deterministic no-device audio types, pins the
old macroquad/miniquad commits and compiles successfully with `cargo check`.
It currently only proves source compilation; it does not yet render a frame,
and bounded modes remain fail-closed until warmup and output are implemented.

The headless branch also carries a small compatibility adjustment in the
renderer entry for the pinned `prpr 0.4.0` API (`BasicPlayer`, `TextPainter`
and `LoadingScene` signatures). This is a local source compatibility layer,
not an assertion that the archived AppImage was linked from that exact API.
