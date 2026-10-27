#!/usr/bin/env python3
"""Render ordered preview windows with one headless Phira process.

The renderer's stdin protocol is two JSON lines: a RenderParams object and a
legacy output path. This wrapper keeps the source params intact, injects the
validated ``config.residentWindows`` list, and launches one resident replay.
It is intentionally a thin process wrapper: it does not inspect or mutate
charts, audio, skins, or existing output files. ``--audio-mode music-only``
adds source music to each window without gameplay sound effects.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import subprocess
import sys
from pathlib import Path
from typing import Any


def parse_window(values: list[str]) -> dict[str, Any]:
    if len(values) != 3:
        raise argparse.ArgumentTypeError("--window requires START_S END_S OUTPUT")
    try:
        start = float(values[0])
        end = float(values[1])
    except ValueError as exc:
        raise argparse.ArgumentTypeError("window times must be finite numbers") from exc
    if not math.isfinite(start) or not math.isfinite(end) or start < 0 or end <= start:
        raise argparse.ArgumentTypeError("window must satisfy 0 <= start_s < end_s")
    output = Path(values[2])
    if not values[2] or output.parent == Path(""):
        # Path("foo.mp4").parent is '.', so this branch only guards an empty
        # CLI value; the current working directory is a valid parent.
        output = Path(values[2])
    if output.name == "":
        raise argparse.ArgumentTypeError("window output must be a file path")
    return {"startS": start, "endS": end, "output": str(output)}


def normalise_output(path_text: str) -> Path:
    path = Path(path_text).expanduser()
    if not path.name:
        raise ValueError(f"output is not a file path: {path_text!r}")
    parent = path.parent if path.parent != Path("") else Path(".")
    if not parent.is_dir():
        raise ValueError(f"output parent does not exist: {parent}")
    return path


def validate_windows(windows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    if not windows:
        raise ValueError("at least one --window is required")
    previous_end = 0.0
    seen: set[Path] = set()
    result: list[dict[str, Any]] = []
    for window in windows:
        start = float(window["startS"])
        end = float(window["endS"])
        if not math.isfinite(start) or not math.isfinite(end) or start < previous_end or end <= start:
            raise ValueError("windows must be finite, ordered, non-overlapping, and satisfy start < end")
        output = normalise_output(str(window["output"]))
        key = output.absolute()
        if key in seen:
            raise ValueError(f"resident outputs must be distinct: {output}")
        seen.add(key)
        result.append({"startS": start, "endS": end, "output": str(output)})
        previous_end = end
    return result


def load_params(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text())
    except OSError as exc:
        raise ValueError(f"cannot read params: {path}: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise ValueError(f"params is not valid JSON: {path}: {exc}") from exc
    if not isinstance(data, dict) or not isinstance(data.get("config"), dict):
        raise ValueError("params must be an object containing a config object")
    return data


def build_payload(params: dict[str, Any], windows: list[dict[str, Any]], audio_mode: str) -> dict[str, Any]:
    # Make a shallow copy of the top-level object and a fresh config so callers
    # can safely reuse the source JSON. The renderer itself remains authoritative
    # for the final validation of all fields.
    payload = dict(params)
    config = dict(params["config"])
    config["renderMode"] = "full"
    config["stateMode"] = "replay"
    config["audioMode"] = audio_mode
    config["warmupS"] = 0.0
    config.pop("startS", None)
    config.pop("endS", None)
    config["residentWindows"] = windows
    payload["config"] = config
    return payload


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--params", type=Path, required=True, help="base RenderParams JSON")
    parser.add_argument("--renderer-binary", type=Path, required=True)
    parser.add_argument("--assets", type=Path, required=True, help="Phira resource directory")
    parser.add_argument(
        "--window",
        action="append",
        nargs=3,
        metavar=("START_S", "END_S", "OUTPUT"),
        required=True,
        help="ordered half-open window; repeat for additional outputs",
    )
    parser.add_argument("--payload-out", type=Path, help="optional path to save the generated resident JSON")
    parser.add_argument(
        "--audio-mode",
        choices=("none", "music-only"),
        default="none",
        help="per-window audio mode; music-only adds source music without gameplay SFX",
    )
    args = parser.parse_args()

    try:
        source = load_params(args.params)
        windows = validate_windows([parse_window(values) for values in args.window])
        payload = build_payload(source, windows, args.audio_mode)
        if not args.renderer_binary.is_file():
            raise ValueError(f"renderer binary does not exist: {args.renderer_binary}")
        if not args.assets.is_dir():
            raise ValueError(f"assets directory does not exist: {args.assets}")
        if args.payload_out:
            args.payload_out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n")
    except (OSError, ValueError, argparse.ArgumentTypeError) as exc:
        parser.error(str(exc))

    # The second line is retained for compatibility with old callers. Resident
    # mode ignores it, but use the first output so logs remain intuitive.
    legacy_output = windows[0]["output"]
    request = json.dumps(payload, ensure_ascii=False, separators=(",", ":"))
    command = [str(args.renderer_binary), "render", str(args.assets)]
    try:
        completed = subprocess.run(
            command,
            input=f"{request}\n{json.dumps(legacy_output)}\n",
            text=True,
            check=False,
            env=os.environ.copy(),
        )
    except OSError as exc:
        print(f"resident renderer could not start: {exc}", file=sys.stderr)
        return 127
    if completed.returncode != 0:
        return completed.returncode
    print(json.dumps({"ok": True, "windows": windows}, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
