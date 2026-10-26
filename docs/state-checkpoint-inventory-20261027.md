# Phira state checkpoint and resident worker inventory

This inventory is based on the pinned `prpr` snapshot in this repository. It
defines what a safe warm start would have to preserve; it does not implement
random seeking or enable a resident worker.

## Why changing the clock is insufficient

`TimeManager::seek_to` only changes `start_time` and schedules a short wait
(`upstream/phira/prpr/src/time.rs:108-111`). It does not run the scene update
that would have occurred between the old and new times. `GameScene::update`
then applies several history-dependent systems in order: autoplay/judge,
custom update callbacks, chart and note caches, video decoding, and effects
(`upstream/phira/prpr/src/scene/game.rs:899-950`). A direct clock jump therefore
cannot reconstruct the same frame as replay.

## State inventory

| Component | Evidence in source | State that changes during replay | Safe reconstruction |
|---|---|---|---|
| Scene/time phase | `GameScene::state`, `last_update_time`, `pause_rewind`, `pause_first_time` (`scene/game.rs:139-142`) | Starting/BeforeMusic/Playing/Ending transitions, fade alpha, pause behavior | Reconstruct only by replaying the phase transitions with the same fixed frame clock |
| `TimeManager` | `time.rs:3-11`, `time.rs:88-110` | start/pause/wait/speed and real-time correction | Scalar fields are serializable, but restoring them alone is insufficient |
| Chart line/object animation | `Chart::update` and `JudgeLine::update` (`core/chart.rs:91-107`, `core/line.rs:118-164`) | per-line `Anim` times, control objects, note transforms, above/below draw caches | Pure time functions can be evaluated at a target time, but note cache and judge state must match first |
| Notes | `Note::judge`, `Note::object`, `JudgeLine::cache` (`core/note.rs:28-39`, `core/line.rs:114-163`) | judged/pre-judged/hold fields, dead notes, update-order and above/below indices | Serialize note judge status and cache indices, or rebuild them by replay |
| Autoplay/judge | `Judge` fields and `auto_play_update` (`judge.rs:231-270`, `judge.rs:789-827`) | per-line cursor, flick trackers, combo/counts, judgement event queue, hold timing | Must serialize all fields that affect later `res.judge_line_color`, hit FX and ending result; do not reset only the score |
| Bad-note and hit effects | `GameScene::bad_notes` and `Note::update` (`scene/game.rs:144`, `core/note.rs:132-152`) | fading bad notes, hold particle emission timing and colors | Serialize active bad-note entries and hold emission phase, or replay from before their lifetime |
| Particles | `Resource::emitter` is drawn with frame `dt` (`scene/game.rs:1048-1053`); particle emitter uses random spawn values (`particle.rs:109-140`) | active particle list, lifetimes, positions, velocities, RNG-dependent spawn order | Requires emitter snapshot plus deterministic RNG state; otherwise disable particles explicitly |
| Chart effects | `Effect::t` and uniforms (`core/effect.rs:26-31`, `145-184`) | current shader time/uniform values and render-target ping-pong history | Uniform values can be recalculated, but the source render target may contain prior passes; checkpoint the target or disable effects |
| Video background | `core/video.rs:14-30`, `91-126`; decoder thread in `prpr-avc/src/video.rs:50-90` | decoder position, next frame, ended flag, Y/U/V textures and decode thread state | Must reopen/seek and decode to the same frame, or checkpoint decoded textures; direct clock seek is unsafe |
| Music clock | `GameScene::music`, `audio::Music::seek_to` | clip position, paused state and scene offset | Source clip and absolute scene clock must be recorded; this alone does not restore visual state |
| Scene stack/UI globals | `Main::scenes`, `times`, `BILLBOARD`, `DIALOG` (`scene.rs:342-378`, `381-500`) | overlays, return times, dialog scroll/touches and billboard animation | A resident worker must clear/rebuild the stack and thread-local/global UI state between jobs |

The current headless profile replaces video with a no-op implementation and
audio output with deterministic clip stubs. That makes it useful for bounded
geometry diagnostics, but it is not evidence that a full-profile checkpoint is
complete.

## Minimum safe interfaces

### Replay modes

`replay-from-zero` is the current faithful diagnostic behavior. A future
`replay-warmed` mode may start at an earlier scene time only when the warmup
window is explicit and all history-dependent outputs in the target window have
been compared with full replay. The first implementation should keep particles,
video and effects either fully replayed or explicitly disabled; silently
dropping them changes the evidence class.

### Checkpoint mode

A checkpoint must be versioned and bound to the exact chart package, skin,
renderer binary, config, input audio SHA, scene time, frame index and feature
flags. Its payload must cover at least:

1. `GameScene` phase and timing fields;
2. `TimeManager` scalar state and music position;
3. every note's `JudgeStatus`, judge cursors/trackers, judgement queue and bad-note list;
4. chart line caches/control objects and `Resource` time/alpha/camera state;
5. particle active state and RNG state, or an explicit `particles=disabled` flag;
6. effect uniforms plus render-target ping-pong contents, or `effects=disabled`;
7. video decoder position and decoded textures, or `video=disabled`; and
8. scene stack/UI globals if the snapshot is taken outside `GameScene`.

The checkpoint writer should fail closed when a field is not serializable. A
partial snapshot must never be presented as full-faithful output.

### Resident worker

The first safe resident worker should reuse only process/GL/asset setup while
creating a fresh `GameScene` for every job. Reusing the same scene object is
allowed only after an explicit `reset_for_job` contract clears chart caches,
judge state, bad notes, particles, effect targets, video decoder state, music,
scene stack and thread-local input/UI state. A worker job must report the
renderer commit, state mode, feature flags and a state-cleanliness result.

## Promotion gates

Before enabling a warm or resident mode for BeatGlint previews:

* Render the same bounded window by full replay and by the candidate mode at a
  fixed FPS. Compare frame count, frame time grid, line transforms, note
  visibility/ownership and decoded audio clock.
* Use windows that cross autoplay hits, hold tails, flicks, particle bursts,
  effect range boundaries and video frame changes. A quiet geometry-only window
  is not sufficient.
* Run two consecutive jobs with different charts and compare the second job
  against a fresh process. Any cross-job difference is a hard failure.
* Keep the existing AppImage and v6.4 skin as the final visual reference until
  representative official charts pass the comparison. The current
  `state_mode=stateless` protocol must continue to fail closed until these
  gates are met.

