# Render modes

The renderer exposes four conceptual modes. Only the first two currently have
runtime evidence; names are deliberately explicit so a fast diagnostic frame
cannot be mistaken for a faithful gameplay render.

| Mode | Purpose | Current evidence |
|---|---|---|
| `full-faithful` | Existing full-song Phira replay | Golden reference (AppImage) |
| `frame-stateless` | One PNG or one-frame MP4 at a selected time | Implemented as deterministic replay from t=0; no checkpoint |
| `segment-replay` | Bounded no-audio MP4 on the video frame grid | Implemented; replay from t=0, video backgrounds omitted |
| `resident-worker` | Multiple windows in one cleaned process | Design target; not implemented |

Every output manifest must record renderer commit, binary SHA, skin SHA,
chart/audio input SHAs, mode, state mode, time window, warmup, frame rate,
and whether particles, effects, video and audio were enabled. A bounded
request currently uses `state_mode=replay-from-zero`; `warmup_s` must remain
zero. A future checkpoint or resident worker must be introduced as a separate
state mode and pass parity tests before it is enabled.

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
