# claux analytics

Shows a daily spend chart, project breakdown, model breakdown, and monthly cost forecast.

## Usage

```bash
claux analytics
claux analytics --days 7
claux analytics --days 30
claux analytics --json

# Local on-device usage metrics
claux analytics local
claux analytics local --json
claux analytics local --reset
```

## Output

### Daily chart

```
  Daily spend (last 7 days)

  $3.50 ┤
  $3.00 ┤         █
  $2.50 ┤     █   █
  $2.00 ┤     █   █   █
  $1.50 ┤ █   █   █   █
  $1.00 ┤ █   █   █   █   █
  $0.50 ┤ █   █   █   █   █   █   █
        └───────────────────────────
        Mon Tue Wed Thu Fri Sat Sun

  Total $14.20 · Avg $2.03/day · Peak Thu $3.42
```

A 7-day vertical bar chart is shown by default. A 30-day ASCII sparkline appears below it as a compact trend line.

### By Project

```
  By Project
  /Users/you/bigproject    $8.40  59%
  /Users/you/api           $3.20  23%
  /Users/you/myproject     $2.60  18%
```

### By Model

```
  By Model
  claude-sonnet-4-6    $11.20   89K tok/$
  claude-opus-4-7       $3.00   12K tok/$
```

The `K tok/$` column shows output tokens per dollar — higher means more output for the same spend.

### Monthly forecast

```
  Monthly forecast
  Daily avg (7d)   $2.03   ·   Month to date   $18.70
  Est. end of month   $43.60   ·   Annual proj.   $740.00
```

Projections are based on the 7-day rolling average. The end-of-month estimate accounts for remaining calendar days in the current month.

## Flags

| Flag | Description |
|---|---|
| `--days N` | Number of days to include in the chart and tables. Default: 30. |
| `--json` | Output all data as a JSON object |

## Local analytics

`claux analytics local` shows on-device usage metrics that Claux records about itself:

| Metric | Description |
|---|---|
| Command counts | How many times each `claux` command has been run |
| Failure classes | Categories of errors encountered |
| Empty-state frequency | How often sessions/spend data was empty |
| TUI refresh latency | Distribution of TUI refresh times in millisecond buckets |

This data never leaves your machine.

```bash
claux analytics local --reset   # clear all local metrics
```
