# Dashboard Tab

The Dashboard tab is the default view in `claux tui`. It is split into two columns that update every 5 seconds.

## Left column

The left column contains two panels stacked vertically.

### Tokens panel

A horizontal bar chart showing the token breakdown for the active session:

```
  Tokens ─────────────────────────────────────────────────
  Input    ███████████░░░░░░░  42,100   45%
  Output   ██████░░░░░░░░░░░░  18,400   20%
  Cache R  ████░░░░░░░░░░░░░░  12,000   13%
  Cache W  ██░░░░░░░░░░░░░░░░   6,200    7%
  Thinking █░░░░░░░░░░░░░░░░░   2,000    2%
  ──────────────────────────────────────────────────────────
  Total 93,200 tokens · Cache hit 68%  Grade A
```

Each row shows a proportional `█░` bar, the raw token count, and its percentage share of the total.

The summary line shows total tokens and the cache hit rate with a color-coded grade:

| Grade | Cache hit rate | Color |
|---|---|---|
| A | ≥ 60% | Green |
| B | 30–59% | Yellow |
| C | < 30% | Red |

When no session is active, this panel shows **Lifetime stats**:

```
  Lifetime stats ─────────────────────────────────────────
  Total sessions       142
  Lifetime spend       $284.50
  Avg cost/session     $2.00
  Total output tokens  4.2M
  Overall cache hit    58%  Grade B
  Best cache session   94%  /Users/you/api
```

### Usage panel

Shows context window fill, budget bars, and credit status:

```
  Usage ──────────────────────────────────────────────────
  Context window  ████████░░░░  67%  134k / 200k
  This week       ██░░░░░░░░░░  $3.42 / $50.00  resets Mon 09 Jun
  Credit          enabled  $18.70 of $200.00 used  this month
```

Each bar only appears when the relevant config key is set:

| Bar | Required config | Description |
|---|---|---|
| Context window | Always shown | Context fill for the active session |
| This week | `weekly-budget` | Spend vs weekly budget |
| 5h limit | `plan-5h-limit` | Rolling 5-hour plan window usage |
| Credit | `monthly-credit` | Monthly credit spend vs cap |

The weekly bar includes a reset timestamp showing the next Monday in `Mon DD Mon` format.

## Right column — Insights panel

The Insights panel shows context-aware live recommendations, updating every 5 seconds:

```
  Insights ───────────────────────────────────────────────
  Cache efficiency    A  68%  — optimal, no action needed
  Context health      ✓ Healthy  67% full
  Cost projection     $0.84 so far · $0.68/hr · est. $1.52 EOD
  Model               claude-sonnet-4-6
  Thinking            8% of output tokens
  Efficiency          9.4K tok/$
  CLAUDE.md           82/100  ██████░░  Good
  Context quality     A  cache 68% · fill 67%
```

### Insights fields

| Field | Description |
|---|---|
| **Cache efficiency** | Grade A–D with actionable tip when below 50%. "Reuse system prompts" / "Add persistent system prompt" |
| **Context health** | Three thresholds: ✓ Healthy (< 75%), ↑ Consider /compact (75–90%), ⚠ Run /compact now! (> 90%) |
| **Cost projection** | Current cost · burn rate · estimated spend by end of day |
| **Model** | Active model name, color-coded: Opus = magenta, Sonnet = blue, Haiku = green |
| **Thinking** | Extended thinking tokens as a percentage of output (hidden when 0) |
| **Efficiency** | Output tokens per dollar (K tok/$) — higher means more output per dollar |
| **CLAUDE.md** | Quality score 0–100 with a proportional bar and label |
| **Context quality** | Composite grade (A–D) combining cache hit rate and context fill |

### Context quality grades

| Grade | Condition |
|---|---|
| A | Cache ≥ 60% AND fill < 75% |
| B | Cache ≥ 30% OR fill < 75% |
| C | Neither condition met |
| D | Fill ≥ 90% — critical |
