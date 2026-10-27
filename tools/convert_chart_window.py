#!/usr/bin/env python3
"""Convert gameplay-chart seconds to an explicit renderer video-time window.

The renderer accepts scene/video seconds. This tool keeps the conversion
profile explicit instead of scattering a magic ``+3.6`` through callers.
It does not render or read chart/audio assets.
"""

from __future__ import annotations

import argparse
import json
import math


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--start-chart-s", type=float, required=True)
    parser.add_argument("--end-chart-s", type=float, required=True)
    parser.add_argument("--fps", type=int, required=True)
    parser.add_argument("--game-clock-reset-s", type=float, default=3.5333335399627686)
    parser.add_argument("--readback-lag-frames", type=int, default=0)
    args = parser.parse_args()
    if not (math.isfinite(args.start_chart_s) and math.isfinite(args.end_chart_s)
            and 0 <= args.start_chart_s < args.end_chart_s):
        parser.error("require finite 0 <= start-chart-s < end-chart-s")
    if args.fps <= 0 or args.readback_lag_frames < 0 or not math.isfinite(args.game_clock_reset_s):
        parser.error("fps must be positive and timing parameters finite")
    start_frame = math.ceil((args.start_chart_s + args.game_clock_reset_s) * args.fps)
    end_frame = math.ceil((args.end_chart_s + args.game_clock_reset_s) * args.fps)
    start_frame += args.readback_lag_frames
    end_frame += args.readback_lag_frames
    result = {
        "format": "beatglint_chart_window_conversion_v1",
        "input_time_domain": "gameplay-chart-s",
        "output_time_domain": "renderer-video-s",
        "start_chart_s": args.start_chart_s,
        "end_chart_s": args.end_chart_s,
        "fps": args.fps,
        "game_clock_reset_s": args.game_clock_reset_s,
        "readback_lag_frames": args.readback_lag_frames,
        "start_frame": start_frame,
        "end_frame": end_frame,
        "start_video_s": start_frame / args.fps,
        "end_video_s": end_frame / args.fps,
        "frame_count": end_frame - start_frame,
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
