# Dashboard

The Dashboard tab is the first thing you see when you open the Claux popover. It shows the active session at a glance and your Claude subscription plan-limit usage.

## Active Session Card

When a Claude Code session is running, the active session card appears at the top of the Dashboard.

### Header row

| Element | Description |
|---|---|
| **ACTIVE SESSION** badge | Confirms a session is live |
| **Model badge** | The Claude model in use (e.g. `Sonnet 4.6`, `Opus 4.8`) |
| **Elapsed time** | How long the current session has been running |

### Cost and burn rate

| Field | Description |
|---|---|
| **Cost** | Total API cost of the session so far, calculated from local JSONL token counts using current pricing |
| **Burn rate** | Cost per hour, computed from the session's average token spend rate |
| **Projection** | Estimated cost by end of day based on the current burn rate |

### Context health bar

The context health bar shows how full the model's context window is, using the most recent assistant message's token counts (not a cumulative total across turns).

| Fill level | Color | Meaning |
|---|---|---|
| 0–69% | Green | Healthy — no action needed |
| 70–89% | Yellow / amber | Consider running `/compact` |
| 90–100% | Red | Run `/compact` now to avoid degraded responses |

The bar displays the raw fill percentage and token count (e.g. `67% · 134k / 200k`).

### Token breakdown

A row-by-row breakdown of every token category in the session:

| Row | Description |
|---|---|
| **Input** | Tokens you sent to the model |
| **Output** | Tokens the model generated |
| **Cache Read** | Tokens served from the prompt cache (billed at ~10× discount) |
| **Cache Write** | Tokens written to the prompt cache |
| **Thinking** | Extended thinking tokens (estimated from thinking content blocks) |

### Session quality score

The session quality score is derived from the session's JSONL and reflects how productive the session has been.

| Metric | Description |
|---|---|
| **Accepted edits** | File changes accepted by the user |
| **Rejected actions** | Tool calls or edits that were rejected |
| **Agent outcomes** | Completed vs failed sub-agent runs |
| **Touched files** | Number of distinct files modified |

The score is color-coded: green ≥ 70, yellow 40–69, red < 40.

### CLAUDE.md quality

If the active session's project contains a `CLAUDE.md` file, Claux scores it on a 0–100 scale across three dimensions:

- **Length** (0–30 pts) — whether the file is substantive
- **Structure** (0–30 pts) — headers, code blocks, bullet lists
- **Content coverage** (0–40 pts) — presence of 8 key topic categories (build, tests, run, conventions, etc.)

Color coding: green ≥ 70, yellow 40–69, red < 40. A score below the configured threshold triggers a warning indicator.

## No active session

When no Claude Code session is running, the card is replaced with a brief idle state showing when the last session ended and today's total spend.

## Plan Limits Card

Below the active session card (or the idle state), the Dashboard shows your Claude subscription plan-limit usage.

See [Plan Limits](plan-limits.md) for full documentation.

## Refresh

Claux refreshes session data automatically every 10 seconds (configurable in Settings). It also refreshes immediately when:

- The popover is opened
- You click the refresh button (↻) in the popover header

The header refresh button updates both session data and plan-limit data.
