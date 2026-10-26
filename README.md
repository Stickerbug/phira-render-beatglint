# phira-render-beatglint

Private BeatGlint renderer workbench for faithful Phira replay and bounded
frame/segment previews.

This repository currently contains a **source recovery checkpoint**, not a
rebuildable renderer. The saved `upstream/` files are the verified diagnostic
snapshot used by BeatGlint. They prove the contents of the pinned `prpr`
source files, but they do not prove that the current AppImage was linked from
these files.

The current full-song renderer remains the golden reference:

- TeamFlos/phira-render v0.1.0 AppImage;
- the existing high-fidelity v6.4 skin;
- the exact chart, audio, parameters and renderer SHA recorded by BeatGlint.

The first implementation target is a separate `frame`/`segment` path that
reuses Phira chart and line rendering. It must preserve a full-faithful mode,
record warmup/state mode in every manifest, and pass parity checks before it is
used by BeatGlint.

This repository intentionally does not contain charts, audio, skins, models,
AppImage trees, migration archives or rendered videos. Those inputs remain
private and are referenced by SHA in manifests.

## Status

See [`STATUS.md`](STATUS.md) and [`provenance/commit-provenance.json`](provenance/commit-provenance.json).

The source snapshot is not buildable yet: the complete Phira workspace,
Cargo.lock and external dependencies still need to be restored and pinned.
