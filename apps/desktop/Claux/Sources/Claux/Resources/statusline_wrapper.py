#!/usr/bin/env python3

from __future__ import annotations

import datetime as dt
import json
import os
import pathlib
import subprocess
import sys
import time


SCRIPT_PATH = pathlib.Path(__file__).resolve()
CLAUX_DIR = SCRIPT_PATH.parent
BASE_DIR = CLAUX_DIR.parent
BRIDGE_FILE = CLAUX_DIR / "statusline_bridge.json"
OUT_FILE = CLAUX_DIR / "rate_limits.json"
DEBUG_LOG = CLAUX_DIR / "statusline_debug.log"


def log(kind: str, **fields: object) -> None:
    now = dt.datetime.now().isoformat(timespec="seconds")
    parts = [f"{now} {kind}"]
    for key, value in fields.items():
        parts.append(f"{key}={value}")
    CLAUX_DIR.mkdir(parents=True, exist_ok=True)
    with open(DEBUG_LOG, "a", encoding="utf-8") as handle:
        handle.write(" ".join(parts) + "\n")


def read_bridge() -> dict[str, object]:
    try:
        with open(BRIDGE_FILE, "r", encoding="utf-8") as handle:
            raw = json.load(handle)
        if isinstance(raw, dict):
            return raw
    except FileNotFoundError:
        return {}
    except Exception as exc:
        log("bridge-read-error", error=type(exc).__name__)
    return {}


def run_downstream(payload: str, command: str | None) -> str:
    if not command:
        return ""

    try:
        proc = subprocess.run(
            ["/bin/zsh", "-lc", command],
            input=payload,
            text=True,
            capture_output=True,
            check=False,
        )
        if proc.returncode != 0:
            log("downstream-failed", exit=proc.returncode)
        return proc.stdout or ""
    except Exception as exc:
        log("downstream-error", error=type(exc).__name__)
        return ""


def normalize_window(rate_limits: dict[str, object], name: str) -> dict[str, object]:
    raw = rate_limits.get(name) or {}
    if not isinstance(raw, dict):
        raw = {}
    return {
        "used_percentage": raw.get("used_percentage"),
        "resets_at": raw.get("resets_at"),
    }


def pct(window: dict[str, object]) -> str:
    value = window.get("used_percentage")
    if isinstance(value, (int, float)):
        return f"{value:.0f}%"
    return "--"


def main() -> int:
    payload = sys.stdin.read()
    bridge = read_bridge()
    downstream_output = run_downstream(payload, bridge.get("downstream_command"))

    CLAUX_DIR.mkdir(parents=True, exist_ok=True)

    if not payload.strip():
        log("empty-stdin")
        sys.stdout.write(downstream_output)
        return 0

    try:
        obj = json.loads(payload)
    except Exception as exc:
        log("invalid-json", error=type(exc).__name__)
        sys.stdout.write(downstream_output)
        return 0

    if not isinstance(obj, dict):
        log("invalid-json", error="top-level-not-object")
        sys.stdout.write(downstream_output)
        return 0

    rate_limits = obj.get("rate_limits") or {}
    if not isinstance(rate_limits, dict):
        rate_limits = {}

    snapshot = {
        "updated_at": int(time.time()),
        "five_hour": normalize_window(rate_limits, "five_hour"),
        "seven_day": normalize_window(rate_limits, "seven_day"),
    }

    with open(OUT_FILE, "w", encoding="utf-8") as handle:
        json.dump(snapshot, handle)

    has_limits = bool(rate_limits)
    log(
        "ok",
        has_rate_limits=has_limits,
        five=snapshot["five_hour"].get("used_percentage"),
        week=snapshot["seven_day"].get("used_percentage"),
    )

    if downstream_output:
        sys.stdout.write(downstream_output)
    else:
        sys.stdout.write(
            f"5h {pct(snapshot['five_hour'])} · 7d {pct(snapshot['seven_day'])}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
