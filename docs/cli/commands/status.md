# claux status

Shows a detailed card for the currently active Claude Code session. If no session is active, shows a brief idle state with the most recent session summary.

## Usage

```bash
claux status
claux            # same as claux status
claux status --json
```

## Output

```
  ● Active session
  Project    /Users/you/myproject
  Model      claude-sonnet-4-6
  Duration   1h 14m
  Cost       $0.84
  Burn rate  $0.68/hr   → est. $1.52 by EOD
  Context    ████████░░░░  67%  134k / 200k
  Cache hit  68%  (Grade A)
  Tokens     Input 42k · Output 18k · Cache R 12k · Cache W 6k · Thinking 2k
  CLAUDE.md  82/100  Good
```

## Fields

| Field | Description |
|---|---|
| **Project** | Absolute path to the project directory |
| **Model** | Claude model name and version |
| **Duration** | Elapsed time since the session started |
| **Cost** | Total API cost so far, calculated from token counts |
| **Burn rate** | Average cost per hour based on the session's token spend rate |
| **Projection** | Estimated cost by end of day based on the current burn rate |
| **Context** | Context window fill percentage and token count. Color-coded: green < 70%, yellow 70–90%, red > 90% |
| **Cache hit** | Percentage of input tokens served from the prompt cache. Grade: A ≥ 60%, B 30–59%, C < 30% |
| **Tokens** | Breakdown of every token category: input, output, cache read, cache write, thinking |
| **CLAUDE.md** | Quality score (0–100) for the project's CLAUDE.md file, with a label (Excellent / Good / Fair / Weak) |

## Flags

| Flag | Description |
|---|---|
| `--json` | Output all fields as a JSON object |

## JSON output

```json
{
  "active": true,
  "project_path": "/Users/you/myproject",
  "model": "claude-sonnet-4-6",
  "duration_secs": 4440,
  "cost_usd": 0.84,
  "burn_rate_per_hour": 0.68,
  "context_fill_pct": 67.0,
  "context_tokens": 134000,
  "context_window": 200000,
  "cache_hit_rate": 0.68,
  "input_tokens": 42100,
  "output_tokens": 18400,
  "cache_read_tokens": 12000,
  "cache_write_tokens": 6200,
  "thinking_tokens": 2000,
  "claudemd_score": 82
}
```

## Active session detection

Claux combines two signals to determine whether a session is active:

1. A `~/.claude/sessions/<pid>.json` file exists whose `sessionId` matches the JSONL filename.
2. The JSONL file's modification time is less than 90 seconds ago (fallback for sessions that didn't write a PID file).

Both signals are combined with OR, so crashed sessions that left no PID file still show as active for a short window.
