# State parity harness

`tools/compare_render_artifacts.py` compares two already-rendered artifacts
without starting Phira. It is the first validation layer for future
`replay-warmed` or resident-worker work:

```text
python tools/compare_render_artifacts.py \
  --left fixtures/outputs/profile-release.mp4 \
  --right fixtures/outputs/profile-release-fast.mp4 \
  --manifest fixtures/manifests/headless-smoke-20261027.json \
  --case-index 3
```

The report checks:

- the manifest's half-open frame grid and expected frame count;
- decoded video dimensions, duration and frame count;
- per-frame `framemd5` equality;
- decoded 44.1 kHz stereo PCM equality when both artifacts contain audio; and
- the manifest's expected artifact SHA-256 when one is recorded.

It is read-only and returns a nonzero exit status on mismatch unless
`--allow-mismatch` is supplied. It does not claim that two matching bounded
renders are full-faithful Phira parity: a future state comparison must still
render the same window through the golden AppImage and exercise particles,
judgement effects, holds, flicks and video changes.

The release and `release-fast` Credits segment smoke currently produces an
identical report (`5` frames, no audio, matching per-frame hashes). This is a
tool self-check only; it does not enable random seeking or a resident worker.
