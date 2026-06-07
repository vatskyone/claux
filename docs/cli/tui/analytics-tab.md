# Analytics Tab

The Analytics tab shows spend history, project and model breakdowns, and a monthly cost forecast. All data is derived from local session logs.

## 7-day bar chart

The primary chart shows your daily spend for the last 7 days as vertical proportional bars:

```
  Daily spend — last 7 days
  ──────────────────────────────────────────────────
  $3.50 ┤
  $3.00 ┤               █
  $2.50 ┤           █   █
  $2.00 ┤           █   █   █
  $1.50 ┤   █       █   █   █
  $1.00 ┤   █   █   █   █   █
  $0.50 ┤   █   █   █   █   █   █   █
        └────────────────────────────────
        Mon Tue Wed Thu Fri Sat Sun

  Total $14.20 · Avg $2.03/day · Peak Thu $3.42
```

- Today's column is highlighted in blue
- The header line shows period total, daily average, and the peak day
- Days with no spend show a zero-height bar

## 30-day sparkline

A compact secondary trend line below the 7-day chart:

```
  30-day trend
  ▁▁▂▃▁▂▄▇█▅▃▂▁▁▃▄▅▃▂▁▁▂▃▄▅▆▇█▅▃▂
```

## By Project and By Model tables

Both tables are rendered side by side to use horizontal space efficiently:

```
  By Project                          By Model
  ─────────────────────────────       ────────────────────────────────────
  /you/bigproject   $8.40   59%       claude-sonnet-4-6   $11.20   89K tok/$
  /you/api          $3.20   23%       claude-opus-4-7      $3.00   12K tok/$
  /you/myproject    $2.60   18%
```

### By Project columns

| Column | Description |
|---|---|
| Project | Abbreviated project path |
| Cost | Total spend for the period |
| % | Share of total spend |

### By Model columns

| Column | Description |
|---|---|
| Model | Model name and version |
| Cost | Total spend for the period |
| K tok/$ | Output tokens per dollar (thousands). Higher = more output per dollar. |

The efficiency column (`K tok/$`) lets you compare the relative value of each model for your actual workload.

## Monthly forecast

```
  Monthly forecast
  ─────────────────────────────────────────────────────────
  Daily avg (7d)     $2.03    ·    Month to date   $18.70
  Est. end of month  $43.60   ·    Annual proj.   $740.00
```

| Figure | Calculation |
|---|---|
| Daily avg (7d) | Total spend in the last 7 days ÷ 7 |
| Month to date | Actual spend so far this calendar month |
| Est. end of month | Month-to-date + (daily avg × remaining days in month) |
| Annual proj. | Daily avg × 365 |

Forecasts are based on the 7-day rolling average, not the full month average. They update every 5 seconds as the active session accumulates cost.

## Navigation

The Analytics tab does not have cursor navigation — it is a read-only display. Press `r` to force a refresh, or wait for the 5-second auto-refresh.
