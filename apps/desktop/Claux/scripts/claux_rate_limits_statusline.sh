#!/usr/bin/env bash
set -euo pipefail

payload="$(cat)"

base_dir="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
out_file="${base_dir}/claux/rate_limits.json"
debug_log="${base_dir}/claux/statusline_debug.log"

/usr/bin/python3 - "$payload" "$out_file" "$debug_log" <<'PY'
import datetime as dt
import json
import os
import sys
import time

payload = sys.argv[1]
out_file = sys.argv[2]
debug_log = sys.argv[3]

os.makedirs(os.path.dirname(out_file), exist_ok=True)
now = dt.datetime.now().isoformat(timespec="seconds")

if not payload.strip():
    with open(debug_log, "a", encoding="utf-8") as log:
        log.write(f"{now} empty-stdin\n")
    sys.exit(0)

try:
    obj = json.loads(payload)
except Exception as e:
    with open(debug_log, "a", encoding="utf-8") as log:
        log.write(f"{now} invalid-json error={e}\n")
    sys.exit(0)

rate_limits = obj.get("rate_limits") or {}

def normalize_window(name: str):
    raw = rate_limits.get(name) or {}
    return {
        "used_percentage": raw.get("used_percentage"),
        "resets_at": raw.get("resets_at"),
    }

snapshot = {
    "updated_at": int(time.time()),
    "five_hour": normalize_window("five_hour"),
    "seven_day": normalize_window("seven_day"),
}

with open(out_file, "w", encoding="utf-8") as f:
    json.dump(snapshot, f)

has_limits = bool(rate_limits)
with open(debug_log, "a", encoding="utf-8") as log:
    log.write(
        f"{now} ok has_rate_limits={has_limits} "
        f"five={snapshot['five_hour'].get('used_percentage')} "
        f"week={snapshot['seven_day'].get('used_percentage')}\n"
    )

def pct(window):
    value = window.get("used_percentage")
    if isinstance(value, (int, float)):
        return f"{value:.0f}%"
    return "--"

five = pct(snapshot["five_hour"])
week = pct(snapshot["seven_day"])
print(f"5h {five} · 7d {week}", end="")
PY
