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
