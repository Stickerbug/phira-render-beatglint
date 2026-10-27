# Third-party source and license notices

This repository contains a source workbench and a Tauri-free renderer derived
from the following upstream components. Their original copyright and license
terms remain in force.

| Component | Pinned/source location | License | Notice |
|---|---|---|---|
| Phira / `prpr` | `upstream/phira`, TeamFlos/phira commit `0020dc45914a07637ea74a11c8a030f3fa3f9d43` | GPL-3.0-only | See `LICENSE` and `upstream/phira/LICENSE`. |
| Original `phira-render` source | `upstream/phira-render` | Apache-2.0 | See `LICENSES/Apache-2.0.txt` and `upstream/phira-render/LICENSE`. Modified renderer files retain the upstream attribution. |
| `macroquad` | Locked Git dependency recorded in `upstream/headless-render/Cargo.lock` | MIT OR Apache-2.0 | The incomplete local source snapshot is intentionally not vendored; see `LICENSES/MIT.txt` and `LICENSES/Apache-2.0-macroquad.txt`. |
| `prpr-avc` snapshot | `upstream/phira/prpr-avc` | GPL-3.0-only, as part of the pinned TeamFlos/phira repository | The snapshot's Cargo manifest now records the upstream GPL license; do not remove or replace upstream headers. |
| Other Cargo dependencies | `upstream/*/Cargo.toml`, lockfiles | Their respective package licenses | Check each dependency license before redistribution. |

The root GPL-3.0-only notice applies to BeatGlint's new and modified source as
an independently copyrightable contribution and to the combined distribution
where GPL compatibility permits it. It does not erase or relicense separable
upstream Apache, MIT, or other dependency material. User-provided skins,
charts, audio, illustrations, AppImages, migration archives, and rendered
artifacts are intentionally excluded from this repository.
