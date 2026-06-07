# claux spend

Shows your Claude Code API spend for today, this week, and this month, with trend indicators comparing each period to its equivalent prior period.

## Usage

```bash
claux spend
claux spend --json
```

## Output

```
  Today       $0.84   ↑ from $0.21 yesterday
  This week   $3.42   ↓ from $5.11 last week
  This month  $18.70
```

## Trend indicators

| Indicator | Color | Meaning |
|---|---|---|
| `↑` | Orange | Spending more than the prior period |
| `↓` | Green | Spending less than the prior period |
| _(none)_ | — | Change < 5% (noise threshold) or no prior data |

| Period | Compared against |
|---|---|
| Today | Yesterday |
| This week | Prior 7 days |
| This month | Prior 30 days |

## Flags

| Flag | Description |
|---|---|
| `--json` | Output as a JSON object |

## JSON output

```json
{
  "today": 0.84,
  "yesterday": 0.21,
  "this_week": 3.42,
  "prev_week": 5.11,
  "this_month": 18.70,
  "prev_month": 0.0
}
```

## Cost attribution

Spend figures use per-turn cost attribution — the cost of each assistant response is attributed to the calendar day when that response was generated, not the day the session started. This means sessions that span midnight are split correctly across days.

> **Note:** These figures are estimates computed from local token counts and Claux's built-in pricing table. Always refer to the [Anthropic console](https://console.anthropic.com) for authoritative billing.
