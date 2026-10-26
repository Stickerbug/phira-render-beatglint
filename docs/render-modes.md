# Render modes

The repository will expose four explicit modes:

| Mode | Purpose | Evidence status |
|---|---|---|
| `full-faithful` | Existing full-song Phira replay | Golden reference |
| `frame-stateless` | Fast diagnostic PNG at selected times | Not a full-state replay |
| `segment-warmed` | Bounded clip after fixed-step warmup | Candidate native evidence after parity |
| `resident-worker` | Multiple windows in one cleaned process | Allowed only after state-isolation tests |

Every output must record renderer commit, binary SHA, skin SHA, chart/audio
input SHAs, time window, warmup and whether particles/effects/audio were
enabled. A fast output must never be silently presented as a full-faithful
render.
