# Public release checklist

This repository can be made public only after every unchecked item is closed.

- [x] Root GPL-3.0 license is present for BeatGlint additions and modifications.
- [x] Apache-2.0 and MIT license texts for vendored snapshots are retained.
- [x] User-provided skins, charts, audio, illustrations, AppImages, migration archives and rendered videos are excluded.
- [x] Root and component README files have Chinese instructions or a Chinese companion README.
- [x] Confirm the `prpr-avc` license at the pinned upstream GPL repository and retain its notice.
- [x] Generate and review a locked Cargo dependency license/SBOM report at `provenance/cargo-license-sbom.json`.
- [x] Scan vendored source/docs for private paths, credentials and local-only artifacts; none remain in tracked public files.
- [x] Build from a clean clone using the Chinese instructions in `docs/公开发布与构建.md`.
- [ ] Run `cargo test`, release build, manifest verification and the documented bounded smoke.
- [ ] Replace any local absolute paths in examples with placeholders or environment variables.
- [ ] Only then change the GitHub repository visibility to public.
