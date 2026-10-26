# Status

## 2026-10-27 bounded renderer checkpoint

- The private GitHub repository `Stickerbug/phira-render-beatglint` exists;
  branch `work/headless-frame-segment` is pushed and tracks its remote.
- The pinned Phira source is `TeamFlos/phira@0020dc45914a07637ea74a11c8a030f3fa3f9d43`.
  The restored source manifest and archive provenance are recorded under
  `upstream/` and `provenance/`.
- The Tauri-free `upstream/headless-render` profile passes `cargo check`, five
  render-plan unit tests, and a release build with the isolated Rust 1.99.1
  toolchain. No BeatGlint Python environment was changed.
- A real Xorg smoke used the private Credits prefix package
  `59bad2003abb894bc7d2b2267f4c0282f213de5dae3ce1ee6d5d64c9cf6bc4ef`,
  320×180 at 10 fps, with the v6.4 skin. The recorded cases include a one
  frame MP4, a direct 320×180 PNG, an aligned 3.5–4.0 s segment (5 frames),
  and an off-grid 3.51–4.0 s segment (4 frames). Their hashes and commands
  are in `fixtures/manifests/headless-smoke-20261027.json`.
- The release binary exits successfully and the generated outputs decode with
  `ffprobe`. The runtime still prints macroquad async wake warnings on stderr;
  these are tracked as a runtime cleanup item, not treated as parity evidence.

## Explicit limitations

- This is a diagnostic headless profile, not full-faithful evidence. It omits
  video decoding and does not mux bounded audio.
- Bounded rendering replays from t=0 with a fixed frame step. Non-zero warmup
  is currently rejected until scene state can be checkpointed safely.
- The full AppImage remains the golden reference for particles, video
  backgrounds, audio timing, and gameplay-visible state. No AppImage or model
  weights were replaced, and the BeatGlint main pipeline was not changed.

## Promotion gates

Before this profile is used for production previews, it must have:

1. an explicit state mode in every request and manifest;
2. a parity report against the existing full renderer for representative
   chart windows, including line ownership, note visibility and clock mapping;
3. a safe stateful path (resident worker or validated checkpoint) before
   claiming random-access or warm-start fidelity;
4. a deliberate audio/video mux path with independently verified timestamps;
5. binary, input and skin SHA-256 records retained alongside every artifact.
