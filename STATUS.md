# Status

## 2026-10-26 source recovery checkpoint

- `prpr` source snapshot: the pinned source tree from
  `TeamFlos/phira@0020dc45914a07637ea74a11c8a030f3fa3f9d43`.
- `prpr-avc`, locales, shader files, root `Cargo.toml`, root `Cargo.lock` and
  the upstream renderer source are now present.
- The macroquad snapshot is still partial; external git dependencies and the
  exact renderer linkage need to be restored or proven.
- Current state: **not yet build-verified**. The environment does not contain
  `cargo` or `rustc`, so no build has been attempted.
- No renderer binary was rebuilt and no AppImage was replaced.
- No chart, audio, skin, model, training, separation or transcription data is
  included here.
- Local checkpoint commits are `05cdcc3`, `5e5850b` and `76a93d6`.
- The intended GitHub repository is `Stickerbug/phira-render-beatglint`; both
  `gh repo create` and the REST create call were refused with HTTP 403
  (`Resource not accessible by integration`). No remote repository was
  created and no source was pushed.

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
