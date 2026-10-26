# Build recovery notes

## 2026-10-27

The original upstream renderer manifest still refers to the deleted
`TeamFlos/phira` `render-lib` branch, so an unmodified Cargo resolution cannot
serve as the build. The repository instead pins the verified Phira source
under `upstream/phira` and provides a Tauri-free `upstream/headless-render`
profile.

The isolated Rust 1.99.1 toolchain lives outside the project. The headless
profile removes Tauri UI, file dialogs, live CPAL/ALSA audio and `prpr-avc`
video decoding while retaining chart timing and geometry. It passes
`cargo check`, five unit tests and a release build. A real Xorg smoke produced
PNG, one-frame MP4 and bounded segment outputs; the output manifest records
input and output hashes.

This profile intentionally does not establish full renderer parity. The
existing AppImage remains the reference for video backgrounds, audio muxing,
particles and all stateful gameplay effects. The first bounded path replays
from t=0 and rejects non-zero warmup, because direct time assignment does not
reconstruct scene, particle, judgement or video state. A future checkpoint or
resident worker must be proved against the reference before promotion.
