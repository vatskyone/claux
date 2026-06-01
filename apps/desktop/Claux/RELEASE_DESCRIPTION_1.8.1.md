# Claux v1.8.1 — Release Description

Claux v1.8.1 improves plan-limit visibility and makes blank states explicit, so users immediately understand whether usage bars are active, waiting, unavailable, or stale.

## Highlights

- Added **Dashboard plan-limit metrics** for:
  - `5-hour` usage window
  - `7-day` usage window
- Added progress bars, percentage labels, and reset countdowns.
- Added **blank-state diagnostics** when bars are empty:
  - Waiting for first API response
  - statusLine source not running
  - Plan limits unavailable in current session payload
  - Data stale
- Added local statusline collector script:
  - `apps/desktop/Claux/scripts/claux_rate_limits_statusline.sh`

## What Changed

- New model types for plan-limit windows and diagnostics.
- New `RateLimitMonitor` to parse and monitor `~/.claude/claux/rate_limits.json`.
- `AppStore` now publishes plan-limit snapshots + diagnostics to UI.
- New `PlanLimitsCard` in Dashboard with actionable UX messages.

## Setup (for plan-limit data)

1. Configure Claude Code `statusLine` command to run the collector script.
2. Ensure the script writes to:
   - `~/.claude/claux/rate_limits.json`
3. Open Claux Dashboard and refresh once.

## Known Behavior

- If your current Claude runtime/session does not expose `rate_limits`, the card now clearly reports that state instead of showing silent empty bars.
- Data source is local-only; no backend required.

## Files Included In This Release

- `apps/desktop/Claux/Sources/Claux/RateLimitMonitor.swift`
- `apps/desktop/Claux/Sources/Claux/Views/PlanLimitsCard.swift`
- `apps/desktop/Claux/Sources/Claux/AppStore.swift`
- `apps/desktop/Claux/Sources/Claux/Models.swift`
- `apps/desktop/Claux/Sources/Claux/Views/PopoverView.swift`
- `apps/desktop/Claux/scripts/claux_rate_limits_statusline.sh`
- `apps/desktop/Claux/CHANGELOG.md`
