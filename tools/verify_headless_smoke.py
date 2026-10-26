#!/usr/bin/env python3
"""Validate timing and stream invariants of a headless smoke manifest.

This verifier does not render, download, or delete artifacts. It checks the
recorded frame-grid math and audio stream claims, and can optionally verify a
local renderer binary SHA-256.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
from pathlib import Path


SHA256 = re.compile(r"^[0-9a-f]{64}$")


def fail(message: str) -> None:
    raise SystemExit(f"manifest verification failed: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def verify_case(case: dict, fps: int) -> None:
    mode = case.get("mode")
    require(mode in {"frame", "segment"}, f"unsupported case mode: {mode!r}")
    frames = case.get("output_frames")
    require(isinstance(frames, int) and frames > 0, "case output_frames must be positive")
    duration = case.get("output_duration_s")
    if duration is None and mode == "frame" and case.get("output_format") == "png":
        duration = 1 / fps
    require(isinstance(duration, (int, float)), "case output_duration_s is missing")
    require(math.isclose(float(duration), frames / fps, abs_tol=1e-9), "case duration is off the fps grid")
    if mode == "frame":
        require(frames == 1, "frame case must emit one frame")
        require("start_s" in case and "end_s" not in case, "frame case window is malformed")
    else:
        start, end = case.get("start_s"), case.get("end_s")
        require(isinstance(start, (int, float)) and isinstance(end, (int, float)) and end > start,
                "segment case window is malformed")
        expected = math.ceil(float(end) * fps) - math.ceil(float(start) * fps)
        require(frames == expected, "segment frame count does not match ceil frame grid")
    if case.get("audio_mode") == "music-only":
        require(mode == "segment", "music-only audio is only valid for segments")
        streams = case.get("streams") or {}
        audio = streams.get("audio") or {}
        video = streams.get("video") or {}
        require(audio.get("codec") == "aac", "music-only audio must be AAC")
        require(audio.get("sample_rate") == 44100, "music-only audio must be 44.1 kHz")
        audio_duration = audio.get("duration_s")
        require(isinstance(audio_duration, (int, float)), "music-only audio duration is missing")
        require(math.isclose(float(audio_duration), float(duration), abs_tol=1 / 44100),
                "music-only audio duration is off the segment duration")
        require(video.get("frames") == frames, "audio case video frame count mismatch")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--renderer-binary", type=Path)
    args = parser.parse_args()
    data = json.loads(args.manifest.read_text())
    require(data.get("format") == "beatglint_headless_renderer_smoke_v1", "unexpected manifest format")
    require(isinstance(data.get("renderer_commit"), str) and re.fullmatch(r"[0-9a-f]{40}", data["renderer_commit"]),
            "renderer_commit must be a full commit SHA")
    binary_sha = data.get("renderer_binary_sha256")
    require(isinstance(binary_sha, str) and SHA256.fullmatch(binary_sha), "renderer_binary_sha256 is malformed")
    fps = data.get("config", {}).get("fps")
    require(isinstance(fps, int) and fps > 0, "config.fps must be positive")
    cases = data.get("cases")
    require(isinstance(cases, list) and cases, "manifest has no cases")
    for case in cases:
        verify_case(case, fps)
    parity = data.get("full_audio_parity")
    if parity is not None:
        require(parity.get("decoded_pcm_exact") is True, "full audio parity is not exact")
        require(parity.get("max_abs_sample_difference") == 0, "full audio parity has non-zero sample error")
    if args.renderer_binary:
        actual = hashlib.sha256(args.renderer_binary.read_bytes()).hexdigest()
        require(actual == binary_sha, f"renderer binary SHA mismatch: {actual}")
    print(json.dumps({"ok": True, "cases": len(cases), "fps": fps, "renderer_commit": data["renderer_commit"]}))


if __name__ == "__main__":
    main()
