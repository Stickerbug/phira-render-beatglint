#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
printf '%s\n' "renderer repository: ${repo_root}"

if command -v cargo >/dev/null 2>&1; then
  printf 'cargo: '; cargo --version
else
  echo 'cargo: MISSING (build is intentionally blocked until the pinned Rust toolchain is installed)' >&2
  missing=1
fi

if command -v rustc >/dev/null 2>&1; then
  printf 'rustc: '; rustc --version
else
  echo 'rustc: MISSING' >&2
  missing=1
fi

for required in \
  "$repo_root/upstream/headless-render/Cargo.toml" \
  "$repo_root/upstream/headless-render/Cargo.lock" \
  "$repo_root/upstream/headless-render/src/main.rs" \
  "$repo_root/upstream/phira/prpr/Cargo.toml" \
  "$repo_root/upstream/phira-render/src-tauri/src/render.rs"; do
  test -f "$required" || { echo "missing: $required" >&2; missing=1; }
done

if [[ "${missing:-0}" != 0 ]]; then
  exit 2
fi

cargo metadata \
  --manifest-path "$repo_root/upstream/headless-render/Cargo.toml" \
  --no-deps --format-version 1 >/dev/null
echo 'metadata: OK'
