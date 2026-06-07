# Agents Tab

The Agents tab monitors every sub-agent spawned by Claude Code in the active session. A green `●` dot on the tab label means at least one agent is still running.

## Agent list

The top 38% of the screen shows a list of all agents in the current session:

```
  Agents (3)  ●
  ────────────────────────────────────────────────────────────────────────
  ● Explore    Lv3 [████░░░░]  Find API endpoints in src/   ★★★★☆  $0.02  14s
  ✓ Plan       Lv2 [██░░░░░░]  Design auth refactor          ★★★★★  $0.08  42s
  ✗ general    Lv1 [░░░░░░░░]  Search for test helpers       ★★☆☆☆  $0.01   8s
```

### Agent list columns

| Column | Description |
|---|---|
| **Status** | `●` green = running, `✓` grey = completed, `✗` red = failed |
| **Type** | Agent type (`Explore`, `Plan`, `general-purpose`, `claude-code-guide`, etc.) |
| **Level** | XP level indicator (`Lv1`–`Lv5`) with a visual XP bar |
| **Task** | One-line task description from the agent's `input.description` |
| **Stars** | Quality score (1–5 stars) based on output completeness and length |
| **Cost** | API cost for this agent run |
| **Duration** | Elapsed time (running) or total time (completed/failed) |

### Navigation

| Key | Action |
|---|---|
| `↑` / `↓` | Move cursor to select an agent |
| `r` | Force refresh of the agent list |

## Agent detail panel

The bottom 62% of the screen shows the detail for the selected agent, updating as you move the cursor:

```
  Agent detail — Explore  Lv3
  ────────────────────────────────────────────────────────────────────────
  Task         Find API endpoints in src/
  Status       completed  in 14s
  Model        claude-sonnet-4-6
  Cost         $0.02

  Token breakdown (% of parent session)
  Input    ██████░░░░░░░░░░  8,200    9%
  Output   ████░░░░░░░░░░░░  4,100    5%
  Cache R  ██░░░░░░░░░░░░░░  2,800    3%

  Output preview
  Found 12 API endpoints in src/api/routes.rs and src/api/handlers.rs.
  The main router is defined at line 45 of routes.rs. Key endpoints: ...

  Quality  ★★★★☆  Good — rich output, no error keywords
```

### Detail panel fields

| Field | Description |
|---|---|
| **Task** | Full task description passed to the agent |
| **Status** | Running / completed / failed with duration |
| **Model** | Model the agent used |
| **Cost** | Agent's own API cost |
| **Token breakdown** | Per-category token counts and their share of the parent session's total |
| **Output preview** | First 250 characters of the agent's result |
| **Quality** | Stars rating with a label explaining the score |

The token breakdown shows each category as a percentage of the parent session total, making it easy to see how much of the session cost was attributable to each sub-agent.

If the agent's own JSONL file is not available (e.g. the session has been cleaned up), the panel shows a graceful fallback with the data available from the parent JSONL.

## XP and level system

Agent levels are computed from cumulative invocation counts across all sessions:

| Level | Tasks completed |
|---|---|
| Lv.1 | 1–4 |
| Lv.2 | 5–14 |
| Lv.3 | 15–29 |
| Lv.4 | 30–59 |
| Lv.5 | 60+ |

The XP bar (`[████░░░░]`) shows progress toward the next level. Counts are global across all projects and sessions.

## Quality star system

Quality stars are computed from the agent's output:

| Stars | Condition |
|---|---|
| ★★★★★ | Output ≥ 500 characters, no error keywords |
| ★★★★☆ | Good output, minor issues |
| ★★★☆☆ | Moderate output (200–499 chars) |
| ★★☆☆☆ | Minimal output or contains error keywords |
| ★☆☆☆☆ | Did not complete (no output or empty result) |

Error keywords checked: `error`, `failed`, `exception`, `traceback`, `panic`, `not found`.

## How agent data is parsed

Claux uses a two-pass approach on the session JSONL:
1. First pass: collects all `tool_use` events where `name = "Agent"`
2. Second pass: matches each to its `tool_result` by `tool_use_id`

The agent's own JSONL file (in the `subagents/` companion directory) is then parsed separately for the per-token breakdown. This is all done locally with no network calls.
