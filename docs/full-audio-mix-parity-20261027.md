# Full audio-mix parity smoke

The chunked full-render mixer was checked against the existing AppImage on the
same Credits prefix package and the same 320×180, 10 fps parameters. The new
renderer still produced 100 frames and a 10.0 s MP4 with the same 44.1 kHz
stereo MP3 stream duration (`9.900998 s`).

After decoding both MP4 audio streams to signed 16-bit stereo PCM with FFmpeg,
the common arrays had 873,310 samples and were byte-identical:

* mean absolute sample difference: `0`
* maximum absolute sample difference: `0`
* exact sample fraction: `1.0`
* decoded PCM SHA-256: `42e84d3205036853ebf2a8e41b8557f062bfbbdf402d9b189e7e0f5dc363d996`

The new video container SHA differs from the AppImage smoke because this
repository is the diagnostic source build and does not claim full visual
parity. The important result here is that replacing the whole-song float
buffer with 8192-frame chunks preserved the full audio mix exactly for the
reference input while bounding the mixer working set. The observed child
maximum RSS was 178,224 KiB for this small smoke (including GL and runtime
resources), and the render completed in about 1.27 s. No training, separation,
or transcription was run.
