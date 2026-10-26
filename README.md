# phira-render-beatglint

Private BeatGlint renderer workbench for faithful Phira replay and bounded
frame/segment previews.

The existing full-song Phira AppImage, with the high-fidelity v6.4 skin, is
still the visual and gameplay reference. This repository is an isolated
optimization boundary: it does not replace that reference and it does not
contain charts, audio, skins, models, migration archives or rendered videos.
Those inputs remain private and are referenced by SHA-256 in manifests.

The `work/headless-frame-segment` branch contains the first buildable
Tauri-free diagnostic profile. It reuses Phira chart and line rendering and
supports a single PNG/frame or a bounded segment. Segment audio is opt-in
`audio_mode=music-only` and contains source music only; the default remains
silent. Bounded requests replay deterministically from time zero; they do not
yet seek through a saved scene state. Video backgrounds, full-faithful audio
mixing and state checkpointing therefore remain on the existing AppImage path.

The branch is pushed to the private GitHub repository
`Stickerbug/phira-render-beatglint`. See [`STATUS.md`](STATUS.md) for current
evidence and [`docs/render-modes.md`](docs/render-modes.md) for the mode
contracts. Build provenance is recorded in
[`provenance/commit-provenance.json`](provenance/commit-provenance.json).
