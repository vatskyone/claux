# Analytics

The Analytics tab lives in the popover and gives you a compact spend overview. For the full-screen analytics window, click **Open full Analytics window** at the bottom of the tab.

## Spend Summary

Three spend cells at the top of the Analytics tab show your cumulative API spend:

| Cell | Period | Trend indicator |
|---|---|---|
| **Today** | Current calendar day | ↑/↓ vs yesterday |
| **This week** | Current 7-day window | ↑/↓ vs prior 7 days |
| **This month** | Current calendar month | ↑/↓ vs prior 30 days |

Trend arrows appear in **orange** (spending more than the prior period) or **green** (spending less). The arrow is hidden when the change is under 5% or there is no prior history to compare against.

### 7-day sparkline

Below the spend cells, a compact bar chart shows your daily costs for the last 7 days. Today's bar is full blue; prior days are dimmed. The sparkline is hidden when there is no spend history.

## Monthly Budget

If you have set a monthly budget in **Settings → General → Monthly budget**, a progress bar appears below the sparkline:

| Fill level | Color | Label |
|---|---|---|
| 0–69% | Green | Remaining amount |
| 70–89% | Yellow | Remaining amount with a warning |
| 90–100% | Red | Over budget / amount over |

The budget tracker resets at the start of each calendar month.

## Full Analytics Window

Clicking **Open full Analytics window** opens a larger window with more detail.

### 30-day daily cost chart

An interactive bar chart (SwiftUI Charts) showing your daily API spend for the last 30 days. Each bar represents one day. Hover over a bar to see the exact date and cost.

You can switch between **7-day** and **30-day** views using the segmented control above the chart. The 7-day view is the default.

### By Project

A table of your top projects by total spend, sorted descending. Each row shows:

- Project path (truncated to the last two path components)
- Total cost for the period

### By Model

A table of spend broken down by Claude model:

- Model name and version
- Total cost for the period
- Efficiency: output tokens per dollar (K tok/$)

The efficiency column lets you compare the relative value of each model for your workload — a higher K tok/$ means more output per dollar spent.

## Cost attribution accuracy

Claux attributes cost per assistant turn, not per session. This means sessions that span midnight are split correctly — the portion of tokens generated after midnight is attributed to the new day. Today's spend figure is always accurate regardless of when a session started.

> **Important:** Cost figures are computed from local JSONL token counts using Claux's built-in pricing table. They are close estimates, not authoritative billing records. Always check your [Anthropic billing dashboard](https://console.anthropic.com) for invoicing.
