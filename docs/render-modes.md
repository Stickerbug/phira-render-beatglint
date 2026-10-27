# Render modes

The renderer exposes four conceptual modes. The first three have bounded or
golden runtime evidence; names are deliberately explicit so a fast diagnostic
frame cannot be mistaken for a faithful gameplay render.

| Mode | Purpose | Current evidence |
|---|---|---|
| `full-faithful` | Existing full-song Phira replay | Golden reference (AppImage) |
| `frame-replay` | One PNG or one-frame MP4 at a selected time | Implemented as deterministic replay from t=0; no checkpoint |
| `segment-replay` | Bounded no-audio MP4 on the video frame grid | Implemented; replay from t=0, video backgrounds omitted |
| `resident-worker` | Ordered windows in one loaded scene/process | Experimental opt-in; bounded silent replay only |

Every output manifest must record renderer commit, binary SHA, skin SHA,
chart/audio input SHAs, mode, state mode, time window, warmup, frame rate,
and whether particles, effects, video and audio were enabled. A bounded
request currently uses `state_mode=replay`; `warmup_s` must remain
zero. A resident request is supplied through `config.residentWindows`, an
ordered array of `{startS, endS, output}` windows. The renderer replays from
time zero once to the largest `endS`, routes each frame range to its own
FFmpeg sink, and keeps the ordinary two-line stdin protocol. The second
output-path line is still required for protocol compatibility and is ignored
for this mode. Windows must be non-overlapping and use `renderMode=full`,
`stateMode=replay`, `audioMode=none`. This is an ordered throughput
optimization, not random seeking or a state checkpoint; it remains opt-in
until more charts and full-faithful audio/video parity are verified.

The frame protocol rounds a requested frame to the nearest video frame for
single-frame output. Segment windows use a half-open `[ceil(start·fps),
ceil(end·fps))` range, so off-grid requests have a recorded, reproducible
frame count.

Bounded requests also carry an explicit `stateMode` (`replay` or
`stateless`). `replay` is the only executable mode today: the scene advances
from time zero at fixed frame steps, so a frame request is deterministic but
does not provide a random-access state checkpoint. `stateless` is reserved and
fails closed until direct seeking has an explicit contract for judge state,
particles, effects, and video. This prevents a caller from treating a future
fast path as equivalent to Phira's full replay before parity is demonstrated.

Segment requests may opt into `audio_mode=music-only`. This writes the source
music sampled on the same absolute frame grid as the emitted segment and muxes
it as AAC; it deliberately excludes click/drag/flick SFX and ending audio.
Frame requests and the default `audio_mode=none` remain silent. The complete
audio and video mix still belongs to `full-faithful` until its clock terms have
been checked against the golden AppImage.

Callers converting nominal gameplay time should use
`tools/convert_chart_window.py` and record its JSON in the artifact manifest.
The reset and readback-lag fields are profile inputs: the full AppImage's
historical PBO path uses a two-frame lag at 30 fps, while bounded direct
readback uses zero lag. Neither profile should be replaced by a global magic
offset.
