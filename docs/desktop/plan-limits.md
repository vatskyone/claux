# Plan Limits

The Plan Limits card in the Dashboard tab shows your Claude subscription usage for the two rolling windows Anthropic enforces on Claude Pro and Max plans.

## Overview

Claude's API enforces rate limits on a **5-hour rolling window** and a **7-day rolling window**. When you approach these limits, Claude Code slows down or pauses until the window resets. The Plan Limits card gives you visibility into how close you are before that happens.

## The two windows

| Window | What it tracks | Reset behavior |
|---|---|---|
| **5-hour** | Token and spend usage in the last 5 hours | Rolls forward continuously; oldest usage falls off as time passes |
| **7-day** | Token and spend usage in the last 7 days | Same rolling behavior |

Each window shows:

- A progress bar (green → yellow → red as usage climbs)
- Used percentage
- Reset timestamp — both a countdown ("resets in 2h 14m") and the exact local time in 24-hour format

## Data source

Plan-limit data comes from the Claux `statusLine` integration — a small Python wrapper that intercepts Claude Code's `statusLine` hook and writes rate-limit data to `~/.claude/claux/rate_limits.json`.

**The Plan Limits card requires the integration to be installed.** Without it, the card shows a diagnostic message explaining why data is unavailable.

See [Claude Integration](claude-integration.md) for installation instructions.

## Diagnostic states

The card provides specific diagnostic messages instead of a generic placeholder:

| State | Message | What it means |
|---|---|---|
| **Integration not installed** | Prompt to install | The statusLine hook hasn't been set up yet |
| **Waiting for first response** | "Waiting for first API response" | The integration is installed but hasn't received any data yet — start a session |
| **Source not running** | "statusLine source not running" | The integration is installed but Claude Code hasn't emitted a status update recently |
| **Unmanaged statusLine** | Warning with detected command | You have a custom `statusLine` command Claux doesn't manage |
| **Stale data** | Timestamp of last known data | Data was received previously but hasn't been updated recently |
| **No data in payload** | "Plan limits unavailable" | Claude Code emitted a statusLine event but it contained no rate-limit fields |

## Auto-refresh

Plan-limit data refreshes automatically:

- Every time the session monitor detects new session data
- Every time you click the refresh button (↻) in the popover header
- Every time you open the popover

## Reset timestamps

Reset times are always shown in your local timezone using 24-hour format:

- **5-hour window**: `HH:mm` (e.g. `14:32`)
- **7-day window**: `DD Mon HH:mm` (e.g. `09 Jun 08:00`)
