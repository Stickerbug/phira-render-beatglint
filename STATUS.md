# Status

## 2026-10-26 source recovery checkpoint

- `prpr` source snapshot: 43 Rust source files from
  `TeamFlos/phira@0020dc45914a07637ea74a11c8a030f3fa3f9d43`.
- Renderer entry snapshot: the restored `renderer-preview/render.rs`.
- Macroquad snapshot: only the diagnostic `Cargo.toml` and `src/lib.rs`.
- Current state: **not buildable**. Root workspace metadata, complete
  dependencies, Cargo.lock and license files are not present in this
  checkpoint.
- No renderer binary was rebuilt and no AppImage was replaced.
- No chart, audio, skin, model, training, separation or transcription data is
  included here.

## Promotion gates

The repository may gain a buildable baseline only after:

1. complete upstream source and dependency provenance is recorded;
2. licenses are checked for every vendored or patched dependency;
3. an unmodified build reproduces the smoke output dimensions, frame count,
   clock mapping and decode result;
4. BeatGlint records the resulting binary SHA and keeps the existing AppImage
   as the full-faithful fallback.

Optimized frame/segment code must be introduced after the baseline, with each
change carrying a parity report against the baseline.
