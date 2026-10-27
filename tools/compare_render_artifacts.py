#!/usr/bin/env python3
"""Compare two already-rendered artifacts without rendering or mutating files.

The report is designed for state-parity checks: it validates the manifest's
frame-grid contract, compares decoded video frame hashes, and compares decoded
44.1 kHz stereo PCM when both artifacts contain audio. It is intentionally
read-only and never starts the Phira renderer.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


def run_json(args: list[str]) -> dict:
    return json.loads(subprocess.check_output(args, text=True))


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def probe(path: Path) -> dict:
    data = run_json(["ffprobe", "-v", "error", "-show_streams", "-show_format", "-of", "json", str(path)])
    streams = data.get("streams", [])
    video = next((stream for stream in streams if stream.get("codec_type") == "video"), None)
    audio = next((stream for stream in streams if stream.get("codec_type") == "audio"), None)
    if video is None:
        raise ValueError(f"no video stream in {path}")
    return {"video": video, "audio": audio, "format": data.get("format", {})}


def frame_hashes(path: Path) -> list[str]:
    output = subprocess.check_output(
        ["ffmpeg", "-v", "error", "-i", str(path), "-map", "0:v:0", "-f", "framemd5", "-an", "-"],
        text=True,
    )
    hashes = []
    for line in output.splitlines():
        if not line or line.startswith("#"):
            continue
        hashes.append(line.rsplit(",", 1)[-1].strip())
    return hashes


def audio_sha(path: Path, has_audio: bool) -> tuple[str | None, int]:
    if not has_audio:
        return None, 0
    process = subprocess.Popen(
        ["ffmpeg", "-v", "error", "-i", str(path), "-map", "0:a:0", "-ac", "2", "-ar", "44100", "-f", "s16le", "-"],
        stdout=subprocess.PIPE,
    )
    digest = hashlib.sha256()
    count = 0
    assert process.stdout is not None
    for chunk in iter(lambda: process.stdout.read(1024 * 1024), b""):
        digest.update(chunk)
        count += len(chunk)
    if process.wait() != 0:
        raise RuntimeError(f"ffmpeg audio decode failed for {path}")
    return digest.hexdigest(), count


def artifact(path: Path) -> dict:
    metadata = probe(path)
    frames = frame_hashes(path)
    audio_digest, audio_bytes = audio_sha(path, metadata["audio"] is not None)
    video = metadata["video"]
    return {
        "path": str(path),
        "file_sha256": sha256(path),
        "width": video.get("width"),
        "height": video.get("height"),
        "fps": video.get("r_frame_rate"),
        "duration_s": float(video.get("duration") or metadata["format"].get("duration") or 0.0),
        "frame_count": len(frames),
        "frame_hashes": frames,
        "video_codec": video.get("codec_name"),
        "audio_codec": metadata["audio"].get("codec_name") if metadata["audio"] else None,
        "audio_sample_rate": int(metadata["audio"]["sample_rate"]) if metadata["audio"] and metadata["audio"].get("sample_rate") else None,
        "audio_pcm_sha256": audio_digest,
        "audio_pcm_bytes": audio_bytes,
    }


def validate_manifest(case: dict, left: dict, right: dict, fps: int) -> dict:
    mode = case.get("mode")
    expected_frames = case.get("output_frames")
    if mode == "segment":
        expected_frames = math.ceil(float(case["end_s"]) * fps) - math.ceil(float(case["start_s"]) * fps)
    checks = {
        "left_frame_grid": left["frame_count"] == expected_frames,
        "right_frame_grid": right["frame_count"] == expected_frames,
    }
    expected_hash = case.get("output_video_sha256") or case.get("output_sha256")
    if expected_hash:
        checks["left_manifest_hash"] = left["file_sha256"] == expected_hash
        checks["right_manifest_hash"] = right["file_sha256"] == expected_hash
    return checks


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--left", type=Path, required=True)
    parser.add_argument("--right", type=Path, required=True)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--case-index", type=int, default=0)
    parser.add_argument("--allow-mismatch", action="store_true")
    args = parser.parse_args()
    left = artifact(args.left)
    right = artifact(args.right)
    frame_equal = left["frame_hashes"] == right["frame_hashes"]
    audio_equal = left["audio_pcm_sha256"] == right["audio_pcm_sha256"]
    report = {
        "format": "beatglint_render_artifact_parity_v1",
        "left": {key: value for key, value in left.items() if key != "frame_hashes"},
        "right": {key: value for key, value in right.items() if key != "frame_hashes"},
        "frame_count_equal": left["frame_count"] == right["frame_count"],
        "frame_hashes_equal": frame_equal,
        "audio_pcm_equal": audio_equal,
        "mismatched_frame_indices": [
            index for index, pair in enumerate(zip(left["frame_hashes"], right["frame_hashes"])) if pair[0] != pair[1]
        ],
    }
    if args.manifest:
        manifest = json.loads(args.manifest.read_text())
        cases = manifest.get("cases") or []
        if not 0 <= args.case_index < len(cases):
            raise SystemExit(f"case index {args.case_index} is outside manifest")
        report["manifest_checks"] = validate_manifest(cases[args.case_index], left, right, int(manifest["config"]["fps"]))
    report["ok"] = all(report.get("manifest_checks", {}).values()) and report["frame_count_equal"] and frame_equal and audio_equal
    print(json.dumps(report, ensure_ascii=False, indent=2))
    if not report["ok"] and not args.allow_mismatch:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
