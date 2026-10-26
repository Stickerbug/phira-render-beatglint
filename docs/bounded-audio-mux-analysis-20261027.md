# Bounded audio output: clock and mux analysis

This note records the safe boundary for adding audio to the diagnostic
`frame`/`segment` renderer. It does not change the current output contract:
bounded output is still video-only and `full` remains the audio-bearing
reference path.

## What the current renderer does

The bounded loop advances the Phira scene from time zero in fixed steps and
emits only the requested frames. The first emitted segment frame is therefore
at output timestamp zero even though its scene time is the requested absolute
time. For an `fps` of 10 and a request `[3.51, 4.0)`, the emitted range is
frames `[36, 40)`, so the video is 0.4 seconds long and represents scene times
3.6 through 3.9 seconds.

The bounded path already decodes the chart music into `AudioClip`, but it does
not write that clip to the output. This is deliberate: simply passing the
whole music file as a second FFmpeg input would put the song at time zero and
would be wrong for every window after the intro. A frame output must remain
silent; attaching a one-frame audio stream is both wasteful and misleading.

## Safe first audio mode

The first bounded audio mode should be an explicit, opt-in
`music-only-segment` mode with these properties:

* It is accepted only for `mode=segment` and `state_mode=replay`.
* The output audio duration is exactly the number of emitted video frames
  divided by `fps`.
* Audio samples are generated from the decoded `AudioClip` for the same
  absolute scene-time window as the emitted frames. Samples before the music
  start or after the track end are zero padded.
* The music volume follows `volumeMusic`. Click, drag, flick, ending and other
  SFX are not included in this first mode. The manifest must say
  `audio_output=music_only` rather than implying full Phira audio parity.
* The generated temporary audio is encoded by FFmpeg and muxed with the
  bounded video using `-shortest`; frame output stays video-only.

The segment window must be taken from `RenderPlan::segment_frame_range`, not
from the unrounded request values. Define:

```text
start_frame, end_frame = segment_frame_range(fps)
scene_start = start_frame / fps
scene_end   = end_frame / fps
duration    = (end_frame - start_frame) / fps
```

The renderer's existing audio placement starts music at the same intro clock
used by full rendering:

```text
music_start_scene = O - min(chart.offset, 0)
O = LoadingScene::TOTAL_TIME + GameScene::BEFORE_TIME
```

For each output sample at local time `u`, read the decoded clip at
`u + scene_start - music_start_scene`. This preserves the negative-offset
padding used by the full mix. The `ChartInfo.offset` and configurable offset
must be measured against a real full-render smoke before being folded into
this equation; the current full mixer uses `chart.offset` and is therefore the
compatibility baseline, not an assumption to silently expand.

## Why full game audio is a later mode

The full path mixes music, SFX and the ending into a single 44.1 kHz buffer
before invoking FFmpeg. Reusing that code for a short segment is possible, but
it must first prove the following against the golden AppImage:

1. note-event times include the same chart/info/config offset terms as the
   scene;
2. fake notes do not generate SFX;
3. simultaneous notes have the same clipping and summing behavior;
4. the ending is excluded unless the requested scene window reaches it; and
5. FFmpeg encoding delay does not move the audio cue relative to the first
   emitted frame.

Until those checks exist, `game-mix-segment` should fail closed rather than
silently claiming parity. The music-only path is sufficient for checking
music-linked animation timing and avoids creating a second source of gameplay
state.

## Mux and verification requirements

The temporary audio should be kept alive until the video process exits. AAC in
the final MP4 is preferable to copying MP3 into MP4 because decoder support is
more predictable; the manifest should record the codec and sample rate. A
bounded run with audio must verify with `ffprobe` that:

* there is exactly one video and one audio stream;
* video frame count equals `end_frame - start_frame`;
* audio duration is within one sample of that frame-grid duration; and
* the audio stream starts at PTS zero in the segment container.

A deterministic smoke should use a short known package and record the input
package SHA, renderer commit, binary SHA, `start_s`, `end_s`, `fps`, emitted
frame range, audio mode, output stream metadata and output SHA. The current
video-only smoke cases remain unchanged and are the regression guard for the
default path.

## Performance implications

The extra work for a short music-only segment is linear in the segment sample
count and uses a temporary encoded file. It does not train, separate, or
transcribe audio. The larger existing cost is the from-zero scene replay; a
resident worker or checkpoint can only be added after stateful particle,
effect, judge and video state have an explicit serialization contract.

For long segments, the audio writer should stream decoded samples to FFmpeg
instead of allocating a whole-song buffer. The current full-faithful mixer
still retains its existing behavior until a separately verified streaming
replacement is available.

