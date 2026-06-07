# claux sessions

Shows a table of your recent Claude Code sessions, sorted by most recent first.

## Usage

```bash
claux sessions
claux sessions -n 50
claux sessions --json
```

## Output

```
 ●  just now   1h 14m  sonnet  /Users/you/myproject    $0.84  [refactor]
 ○  2h ago     42m     sonnet  /Users/you/api           $0.31
 ○  yesterday  3h 02m  opus    /Users/you/bigproject    $4.17  [v2 arch]
 ○  2d ago     18m     haiku   /Users/you/scripts       $0.04
```

## Columns

| Column | Description |
|---|---|
| **Status dot** | `●` green = active, `○` grey = ended |
| **Time** | Relative time since the session ended (e.g. "just now", "2h ago", "yesterday") |
| **Duration** | Total session duration |
| **Model** | Abbreviated model name (`sonnet`, `opus`, `haiku`) |
| **Project** | Abbreviated project path (last two components) |
| **Cost** | Total session cost in USD |
| **Tag** | Session tag in brackets if one has been set with `claux tag` |

## Flags

| Flag | Description |
|---|---|
| `-n N` | Show the last N sessions. Default: 20. |
| `--json` | Output as a JSON array |

## JSON output

```json
[
  {
    "id": "abc123def456",
    "project_path": "/Users/you/myproject",
    "start_time": "2026-06-07T10:00:00Z",
    "end_time": "2026-06-07T11:14:00Z",
    "duration_secs": 4440,
    "cost_usd": 0.84,
    "model": "claude-sonnet-4-6",
    "is_active": false,
    "title": "Refactor auth middleware",
    "tag": "refactor",
    "input_tokens": 42100,
    "output_tokens": 18400,
    "cache_read_tokens": 12000,
    "cache_write_tokens": 6200,
    "thinking_tokens": 2000
  }
]
```

## Session IDs

Session IDs are UUIDs derived from the JSONL filename. The first 8–12 characters are usually enough to identify a session uniquely for use with `claux tag`:

```bash
claux tag abc123 "refactor"
```
