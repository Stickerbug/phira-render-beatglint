# Native PGR parsing and particle evidence

2026-10-22 (Asia/Shanghai). Read-only public-source investigation; no model,
training, feature production, native build or renderer modification.

The restored renderer archive's Cargo.toml names TeamFlos/phira, branch
render-lib. Its Cargo.lock prpr entry has no git source, so it does not itself
prove an exact prpr commit. Historical project documents identify 0020dc4.
GitHub's commit endpoint resolves that prefix to
`0020dc45914a07637ea74a11c8a030f3fa3f9d43` (2023-09-03, fix path on Windows).
This source is the documented pin, not a newly rebuilt attestation of the
current renderer binary's linkage.

All 43 prpr/src Rust files at that commit were fetched using the GitHub
connector `fetch_file` with the full commit ref. Each file's actual Git blob
SHA-1, including Git blob header and byte length, matches GitHub's tree entry.
`files-manifest.json` also records SHA-256 and byte lengths. Network curl in
the default sandbox failed to connect to the proxy; the connector read the
public source successfully. No network settings or credentials were changed.

## Parsing contracts

References below use local source line numbers. The online equivalent prefix
is `https://github.com/TeamFlos/phira/blob/0020dc45914a07637ea74a11c8a030f3fa3f9d43/`.

- `prpr/src/parse/pgr.rs:18-69`: all numeric time, BPM, speed, floor and pose
  fields deserialize into f32 before the parser arithmetic.
- `pgr.rs:73-91`: validation removes rows whose f32 start is greater than end,
  then requires exact adjacent end/start equality in original array order.
  It does not sort curves or speed events. Empty arrays are not handled safely
  by this macro; an independent diagnostic should reject them explicitly.
- `pgr.rs:94-110`: speed begins at raw start 0 (assertion, not negative clamp).
  For all but the last row, f32 prefix position is accumulated in source order
  as `pos += (raw_end - raw_start) * r * value`. The last row extends to the
  chart-global `max_time`, regardless of its raw end. Height keyframe values
  are divided by f32 HEIGHT_RATIO (0.83175).
- `pgr.rs:113-150`: scalar and move channels can drop a redundant start
  keyframe when the last value equals the next row start value. Starts are
  clamped to 0 after multiplication by r; final end keyframes are popped.
  Therefore a literal source row list is not identical to the native keyframe
  topology, especially around jumps/ties or the final row.
- `pgr.rs:153-194`: each side's notes are sorted by raw f32 time. Hit is
  `raw_time * r`; Hold end is `(raw_time + raw_hold_time) * r`, then height is
  queried at that end. Hold travel speed is raw speed divided by speed at the
  native hit. Note floor is the raw floor divided by HEIGHT_RATIO.
- `pgr.rs:197-198`: r is evaluated as `60. / bpm / 32.` in f32, rather than
  treating a precomputed double `1.875 / bpm` as equivalent.
- `pgr.rs:227-243`: max_time is the chart-global max of each line's maximum
  raw head time multiplied by that line's f32 r, plus f32 1. It is not based
  on the last Hold end or audio duration.
- `prpr/src/core/anim.rs:92-125`: set_time advances while the next keyframe
  time is <= the query (last tie wins), can rewind, and interpolation uses
  `(query-t0)/(t1-t0)` without a general clip in Anim itself. Tween ID 2 is
  linear; ID 0 is step, per core/tween.rs.
- `prpr/src/core/tween.rs:293-296`: f32 interpolation is evaluated as
  `x + (y - x) * t`, with the corresponding intermediate operation order.
- `prpr/src/parse.rs:13-70`: process_lines marks multiple_hint using exact
  equality between native f32 hit times, not a microsecond grouping tolerance.
  A time conversion can change which notes use the multi-note texture even
  when both timings land in the same displayed frame.
- `prpr/src/core/line.rs:59-69`: JudgeLineCache stable-sorts notes by
  `(plain, !above, normalized_speed, (height + localY) * normalized_speed)`.
  `judge.rs:237-244` subsequently stable-sorts those indices by native hit.
  Changes in normalized Hold speed can therefore change same-time iteration
  order and which random values later apply to which simultaneous note.

The pinned PgrChart struct also does not read formatVersion or implement an
obvious packed-v1 move decoding branch. This investigation covers the v3
sources used in the current replay comparison. Python v1 support does not
establish native v1 support; upstream preprocessing would need separate proof.

These operations differ from both converting event boundaries into double
seconds and from the existing target's separately converted Hold hit+duration.
Preserving a double geometry target therefore cannot by itself prove native
f32 parse equivalence after BPM normalization.

## Particle causality: evidence and limits

No `srand` or `seed` call occurs in the fetched 43 prpr/src Rust files or the
seven Rust files from the restored phira-render source archive. This does not
attest the entire linked dependency/binary state. It does rule out asserting
a chart/BPM/time-derived seed from these source files.

`prpr/src/particle.rs` imports macroquad's prelude at line 12. Lines 197-205
and 567-615 call `rand::gen_range` to obtain emission position, size, rotation,
velocity and lifetime variation. The restored renderer Cargo.lock pins
Mivik/prpr-macroquad to `19119d6e3ec231502f6d64ff077114ffe1fcb04a`; its
`src/lib.rs:130-131` reexports `quad_rand::*`. Cargo.lock records quad-rand
0.2.1. Two corresponding macroquad files are retained with verified blobs.

`prpr/src/core/resource.rs:242-306` makes separate hit-fx and square emitters;
emit_at emits one hit sprite plus four square particles when not hidden.
`resource.rs:506-517` skips emission when config.particle is false.

`prpr/src/judge.rs:787-837` iterates autoplay judgments in line/note order;
native note.time > current frame time postpones a note, and Hold end <= current
time closes it. It emits the effect after committing. Separately,
`prpr/src/core/note.rs:132-150` emits during an active Hold when res.time > at,
then advances at by HOLD_PARTICLE_INTERVAL / speed.

Thus native timing/active-state changes can alter which calls consume the
shared PRNG stream at which update, without any need to reseed by chart.
That is a source-backed mechanism, not proof that it explains the observed
Distorted differences. The parent agent's controlled variants are required
to establish which changes affect those frames. Nor does this source read
prove pixel equivalence, manual playability, AP or musical appropriateness.
